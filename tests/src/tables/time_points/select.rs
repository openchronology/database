use anyhow::{Result, bail};
use common::{MPQ, Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    pub id: Identifier,
    pub value: MPQ,
    pub timeline: Identifier,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<TimePoint>> {
    let res = client.get(format!("{}/time_points", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;
    let xs = res.json::<Vec<TimePoint>>()
        .await?;
    Ok(xs)
}

pub async fn select(client: &reqwest::Client, id: Identifier) -> Result<TimePoint> {
    let res = client.get(format!("{}/time_points?id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;
    let resp = res.json::<Vec<TimePoint>>()
        .await?;
    match resp.get(0) {
        None => bail!("No time point with that id".to_owned()),
        Some(t) => Ok(t.clone()),
    }
}
