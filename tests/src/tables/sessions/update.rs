use common::{consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, MPQ};
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
    // jwt: &JWT,
    client: &reqwest::Client,
    id: String,
    session: UpdateSession,
) -> Result<(), String> {

    let res = client.patch(format!("{}/sessions?id=eq.{}", *REST_DATABASE_HOST, id))
        // .header("Authorization", jwt.to_string())
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .json(&session)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        Ok(())
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
