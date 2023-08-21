use common::{MPQ, Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    pub id: Identifier,
    pub value: MPQ,
    pub timeline: Identifier,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<TimePoint>, String> {
    let res = client.get(format!("{}/time_points", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Vec<TimePoint>>()
        .await
        .map_err(|e| e.to_string())
}

pub async fn select(client: &reqwest::Client, id: Identifier) -> Result<TimePoint, String> {
    let res = client.get(format!("{}/time_points?id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let resp = res.json::<Vec<TimePoint>>()
        .await
        .map_err(|e| e.to_string())?;
    match resp.get(0) {
        None => Err("No time point with that id".to_owned()),
        Some(t) => Ok(t.clone()),
    }
}
