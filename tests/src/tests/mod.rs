pub mod select_time_points_and_summaries;

use select_time_points_and_summaries::verify_select_time_points_and_summaries;

use quickcheck::Gen;

pub async fn run_tests(client: &reqwest::Client, g: &mut Gen) {
    let result = verify_select_time_points_and_summaries(client, g).await;

    if let Err(e) = result {
        panic!("{}", e);
    }
}
