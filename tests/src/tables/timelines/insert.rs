use crate::{consts::PGRST_HOST, session::JWT};

use common::Identifier;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertTimeline {
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertedTimeline {
    id: Identifier,
}

pub async fn insert(
    jwt: &JWT,
    client: &reqwest::Client,
) -> Result<Identifier, String> {
    let x = InsertTimeline {};

    let res = client.post(format!("{}/timelines?select=id", *PGRST_HOST))
        .json(&x)
        .header("Authorization", jwt.to_string())
        .header("Prefer", "return=representation")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        let resp = res.json::<Vec<InsertedTimeline>>()
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
