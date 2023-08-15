use crate::consts::PGRST_HOST;

use common::{MPQ, Identifier};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePoint {
    id: Identifier,
    value: MPQ,
    timeline: Identifier,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<TimePoint>, String> {
    let res = client.get(format!("{}/time_points", *PGRST_HOST))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Vec<TimePoint>>()
        .await
        .map_err(|e| e.to_string())
}
