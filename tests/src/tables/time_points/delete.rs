use crate::{consts::PGRST_HOST, session::JWT};

use common::Identifier;

pub async fn delete(
    jwt: &JWT,
    client: &reqwest::Client,
    id: Identifier,
) -> Result<(), String> {

    let res = client.delete(format!("{}/time_points?id=eq.{}", *PGRST_HOST, id))
        .header("Authorization", jwt.to_string())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        Ok(())
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
