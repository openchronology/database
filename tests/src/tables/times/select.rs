use anyhow::Result;
use common::{consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, MPQ};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Time {
    value: MPQ,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<Time>> {
    let res = client.get(format!("{}/times", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await?;
    let xs = res.json::<Vec<Time>>()
        .await?;
    Ok(xs)
}
