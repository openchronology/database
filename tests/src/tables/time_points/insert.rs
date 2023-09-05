use anyhow::{Result, ensure, bail};
use common::{MPQ, Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, session::JWT};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertTimePoint {
    value: MPQ,
    timeline: Identifier,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertedTimePoint {
    id: Identifier,
}

pub async fn insert(
    jwt: &JWT,
    client: &reqwest::Client,
    value: MPQ,
    timeline: Identifier
) -> Result<Identifier> {
    let x = InsertTimePoint { value, timeline };

    let res = client.post(format!("{}/time_points?select=id", *REST_DATABASE_HOST))
        .json(&x)
        .header("Authorization", jwt.to_string())
        .header("Prefer", "return=representation")
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;

    ensure!(res.status().as_u16() / 100 == 2, "Bad response code: {res:?}");
    let resp = res.json::<Vec<InsertedTimePoint>>()
        .await?;
    match resp.get(0) {
        None => bail!("No results after insertion".to_owned()),
        Some(x) => Ok(x.id),
    }
}
