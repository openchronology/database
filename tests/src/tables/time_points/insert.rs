use crate::consts::PGRST_HOST;
use common::MPQ;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertTimePoint {
    value: MPQ,
}

pub async fn insert_time_point(client: &reqwest::Client, value: MPQ) -> Result<(), String> {
    let x = InsertTimePoint { value };

    let res = client.post(format!("{}/time_points", *PGRST_HOST))
        .json(&x)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        Ok(())
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
