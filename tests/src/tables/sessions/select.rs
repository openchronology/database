use anyhow::{Result, bail};
use common::{
    Identifier,
    consts::{
        REST_DATABASE_HOST_HEADER,
        REST_DATABASE_HOST
    },
    MPQ, session::JWT,
};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct SelectedThreshold {
    threshold: MPQ,
}

pub async fn select_threshold(
    jwt: Option<&JWT>,
    client: &reqwest::Client,
    id: String,
) -> Result<MPQ> {
    let req = client.get(format!("{}/sessions_precomputed?select=threshold&id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone());
    let req = match jwt {
        None => req,
        Some(jwt) => req
            .header("Authorization", jwt.to_string()),
    };
    let res = req.send()
        .await?;
    let resp = res.json::<Vec<SelectedThreshold>>()
        .await?;
    match resp.get(0) {
        None => bail!("No timeline present".to_owned()),
        Some(x) => Ok(x.threshold.clone()),
    }
}
