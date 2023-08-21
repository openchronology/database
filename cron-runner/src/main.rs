#[macro_use] extern crate log;
extern crate pretty_env_logger;

use chrono::{Utc, Duration};
use tokio::time::sleep;
use common::{consts::{CRON_USER, PGRST_SERVER_PORT}, session::gen_jwt};
use std::env::var;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let client = reqwest::Client::new();

    loop {
        let jwt = gen_jwt(CRON_USER);
        let t = Utc::now() - Duration::minutes(10);
        let t = t.format("%F %T %Z");
        let m_res = client.delete(format!(
            "http://{}:{}/sessions?last_interaction=lt.{t}",
            var("PGRST_HOST").unwrap(),
            *PGRST_SERVER_PORT,
        ))
            .header("Authorization", jwt.to_string())
            .send()
            .await;
        match m_res {
            Ok(res) if res.status().as_u16() / 100 == 2 => {
                info!("Pruned old sessions");
            }
            _ => {
                warn!("Bad response: {:?} - using jwt: {}", m_res, jwt.to_string());
            }
        }
        sleep(tokio::time::Duration::from_millis(1000 * 60 * 2)).await; // check every 2 minutes
    }
}
