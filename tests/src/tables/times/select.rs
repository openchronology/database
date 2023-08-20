use common::{consts::PGRST_HOST, MPQ};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Time {
    value: MPQ,
}

pub async fn select_all(client: &reqwest::Client) -> Result<Vec<Time>, String> {
    let res = client.get(format!("{}/times", *PGRST_HOST))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    res.json::<Vec<Time>>()
        .await
        .map_err(|e| e.to_string())
}
