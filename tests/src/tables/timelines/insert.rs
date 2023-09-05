use anyhow::{Result, ensure, bail};
use common::{Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, session::JWT};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertTimeline {
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertedTimeline {
    id: Identifier,
}

pub async fn insert(
    jwt: &JWT,
    client: &reqwest::Client,
) -> Result<Identifier> {
    let x = InsertTimeline {};

    let res = client.post(format!("{}/timelines?select=id", *REST_DATABASE_HOST))
        .json(&x)
        .header("Authorization", jwt.to_string())
        .header("Prefer", "return=representation")
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;

    ensure!(res.status().as_u16() / 100 == 2, "Bad response code: {res:?}");
    let resp = res.json::<Vec<InsertedTimeline>>()
        .await?;
    match resp.get(0) {
        None => bail!("No results after insertion".to_owned()),
        Some(x) => Ok(x.id),
    }
}
