#[macro_use] extern crate log;

use common::consts::{PGRST_SERVER_PORT, PGRST_HOST, FILTER_PORT};

use awc::{Client, ClientRequest};
use actix_web::{
    web::{self, Json, Data, Path},
    route,
    http::{Method, header},
    middleware::Logger,
    HttpResponse,
    App, Route,
    HttpServer, HttpRequest, FromRequest,
};
use actix_proxy::{IntoHttpResponse, SendRequestError};



fn build_url(req: &HttpRequest, path: String) -> String {
    // apply the method to the client
    let url = format!(
        "{}{}",
        path,
        if req.query_string().is_empty() {
            "".to_owned()
        } else {
            format!("?{}", req.query_string())
        }
    );
    format!("http://{}:{}/{url}", *PGRST_HOST, *PGRST_SERVER_PORT)
}

fn include_headers(req: &HttpRequest, client: ClientRequest) -> ClientRequest {
    // transfer headers that we care about
    let headers = req.headers();
    let c = if let Some(auth) = headers.get(header::AUTHORIZATION) {
        client.insert_header(("Authorization", auth))
    } else {
        client
    };
    if let Some(prefer) = headers.get("Prefer") {
        c.insert_header(("Prefer", prefer))
    } else {
        c
    }
}

#[route("/{url:.*}", method="GET", method="DELETE")]
async fn proxy_no_body(
    req: HttpRequest,
    path: Path<(String,)>,
    client: Data<Client>,
) -> Result<HttpResponse, SendRequestError> {
    // FIXME block whole requests to /sessions
    let (path,) = path.into_inner();
    let url = build_url(&req, path);
    info!("Built url: {url:?}");
    let method = req.method();
    let c = if method == Method::GET {
        client.get(&url)
    } else if method == Method::DELETE {
        client.delete(&url)
    } else {
        error!("Somehow have impossible method: {method:?}");
        panic!("Somehow have impossible method: {method:?}");
    };
    let c = include_headers(&req, c);
    c.send().await?.into_wrapped_http_response()
}

#[route("/{url:.*}", method="POST", method="PATCH")]
async fn proxy_body(
    req: HttpRequest,
    path: Path<(String,)>,
    client: Data<Client>,
    payload: Json<serde_json::Value>,
) -> Result<HttpResponse, SendRequestError> {
    let (path,) = path.into_inner();
    let url = build_url(&req, path);
    let method = req.method();
    let c = if method == Method::POST {
        client.post(&url)
    } else if method == Method::PATCH {
        client.patch(&url)
    } else {
        error!("Somehow have impossible method: {method:?}");
        panic!("Somehow have impossible method: {method:?}");
    };
    let c = include_headers(&req, c);
    c.send_json(&(*payload)).await?.into_wrapped_http_response()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("Starting Filter server");
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default().log_target("filter"))
            .app_data(web::Data::new(Client::default()))
            .service(proxy_body)
            .service(proxy_no_body)
    })
        .bind(("0.0.0.0", (*FILTER_PORT).parse::<u16>().unwrap()))?
        .run()
        .await
}
