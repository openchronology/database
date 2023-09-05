use anyhow::{Result, bail};
use common::{Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub id: Identifier,
    pub author: String,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<Timeline>> {
    let res = client.get(format!("{}/timelines", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;
    let xs = res.json::<Vec<Timeline>>()
        .await?;
    Ok(xs)
}

pub async fn select(client: &reqwest::Client, id: Identifier) -> Result<Timeline> {
    let res = client.get(format!("{}/timelines?id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;
    let resp = res.json::<Vec<Timeline>>()
        .await?;
    match resp.get(0) {
        None => bail!("No timeline present".to_owned()),
        Some(x) => Ok(x.clone()),
    }
}
