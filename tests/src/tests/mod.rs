pub mod select_time_points_and_summaries;
pub mod insert_then_delete_timeline;
pub mod insert_then_delete_time_point;

use anyhow::Result;
use select_time_points_and_summaries::verify_select_time_points_and_summaries;
use insert_then_delete_timeline::verify_insert_then_delete_timeline;
use insert_then_delete_time_point::verify_insert_then_delete_time_point;

use quickcheck::Gen;
use color_print::cprintln;

pub async fn run_tests(client: &reqwest::Client, g: &mut Gen) {
    let results = vec![
        verify_insert_then_delete_timeline(client).await,
        verify_insert_then_delete_time_point(client, g).await,
        verify_select_time_points_and_summaries(client, g).await,
    ];
    let result: Result<Vec<()>> = results.into_iter().collect();

    if let Err(e) = result {
        cprintln!("<red>{}</red>", e);
        panic!();
    }
}
