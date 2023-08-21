use common::{Identifier, consts::{REST_DATABASE_HOST_HEADER, REST_DATABASE_HOST}, session::JWT};

pub async fn delete(
    jwt: &JWT,
    client: &reqwest::Client,
    id: Identifier,
) -> Result<(), String> {

    let res = client.delete(format!("{}/time_points?id=eq.{}", *REST_DATABASE_HOST, id))
        .header("Authorization", jwt.to_string())
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().as_u16() / 100 == 2 {
        Ok(())
    } else {
        Err(format!("Bad response code: {:?}", res))
    }
}
