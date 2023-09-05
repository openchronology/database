use anyhow::{Result, ensure};
use common::{consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, MPQ, session::JWT};
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pos: Option<MPQ>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub zoom: Option<MPQ>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub field: Option<MPQ>,
}


pub async fn update(
    jwt: Option<&JWT>,
    client: &reqwest::Client,
    id: String,
    session: UpdateSession,
) -> Result<()> {

    let req = client.patch(format!("{}/sessions?id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .json(&session);
    let req = match jwt {
        None => req,
        Some(jwt) => req
            .header("Authorization", jwt.to_string()),
    };

    let res = req.send()
        .await?;

    ensure!(res.status().as_u16() / 100 == 2, "Bad response code: {res:?}");
    Ok(())
}
