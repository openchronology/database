pub mod select_time_points_and_summaries;
pub mod select_timeline_after_insert;

use select_time_points_and_summaries::verify_select_time_points_and_summaries;
use select_timeline_after_insert::verify_select_timeline_after_insert;

use quickcheck::Gen;
use color_print::cprintln;

pub async fn run_tests(client: &reqwest::Client, g: &mut Gen) {
    let results = vec![
        verify_select_time_points_and_summaries(client, g).await,
        verify_select_timeline_after_insert(client).await,
    ];
    let result: Result<Vec<()>, String> = results.into_iter().collect();

    if let Err(e) = result {
        cprintln!("<red>{}</red>", e);
        panic!();
    }
}
