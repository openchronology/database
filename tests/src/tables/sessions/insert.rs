use anyhow::{Result, ensure, bail};
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
) -> Result<String> {
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
        .await?;

    ensure!(res.status().as_u16() / 100 == 2, "Bad response code: {res:?}");
    let resp = res.json::<Vec<InsertedSession>>()
        .await?;
    match resp.get(0) {
        None => bail!("No results after insertion".to_owned()),
        Some(x) => Ok(x.id.clone()),
    }
}
