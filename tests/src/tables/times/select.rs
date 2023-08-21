use common::{consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, MPQ};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Time {
    value: MPQ,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<Time>, String> {
    let res = client.get(format!("{}/times", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Vec<Time>>()
        .await
        .map_err(|e| e.to_string())
}
