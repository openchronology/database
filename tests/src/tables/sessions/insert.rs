use common::{consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, session::JWT};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertSession {
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertedSession {
    id: String,
}

pub async fn insert(
    jwt: Option<&JWT>,
    client: &reqwest::Client,
) -> Result<String, String> {
    let x = InsertSession {};

    let req = client.post(format!("{}/sessions?select=id", *REST_DATABASE_HOST))
        .json(&x)
        .header("Prefer", "return=representation")
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone());
    let req = match jwt {
        None => req,
        Some(jwt) => req
            .header("Authorization", jwt.to_string()),
    };

    let res = req.send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        let resp = res.json::<Vec<InsertedSession>>()
            .await
            .map_err(|e| e.to_string())?;
        match resp.get(0) {
            None => Err("No results after insertion".to_owned()),
            Some(x) => Ok(x.id.clone()),
        }
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
