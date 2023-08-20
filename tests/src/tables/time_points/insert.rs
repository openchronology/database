use common::{MPQ, Identifier, consts::PGRST_HOST, session::JWT};

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
) -> Result<Identifier, String> {
    let x = InsertTimePoint { value, timeline };

    let res = client.post(format!("{}/time_points?select=id", *PGRST_HOST))
        .json(&x)
        .header("Authorization", jwt.to_string())
        .header("Prefer", "return=representation")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        let resp = res.json::<Vec<InsertedTimePoint>>()
            .await
            .map_err(|e| e.to_string())?;
        match resp.get(0) {
            None => Err("No results after insertion".to_owned()),
            Some(x) => Ok(x.id),
        }
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
