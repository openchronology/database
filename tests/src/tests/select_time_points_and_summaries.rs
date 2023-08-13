use crate::{
    stats::Stats,
    bounds::MonotonicBounds,
    rpcs::select_time_points_and_summaries::select_time_points_and_summaries,
};

use std::time::{Instant, Duration};
use quickcheck::{Arbitrary, Gen};

pub async fn verify_select_time_points_and_summaries(
    client: &reqwest::Client,
    g: &mut Gen,
) -> Result<(), String> {
    println!("Verify `select_time_points_and_summaries`...");

    let mut times: Vec<Duration> = vec![];
    let mut lengths: Vec<usize> = vec![];

    for i in 0..100 {
        let sample = MonotonicBounds::arbitrary(g);
        let now = Instant::now();
        let result = select_time_points_and_summaries(&client, sample.clone()).await;
        match result {
            Ok(l) => {
                times.push(now.elapsed());
                lengths.push(l);
            }
            Err(e) => return Err(
                format!(
                    "Test case returned an error: {:?}\nSample: {:?}\nIteration: {:?}",
                    e,
                    sample,
                    i
                )
            ),
        }
    }

    let times_stats = Stats::new(&times, Duration::as_secs_f64)
        .resolve(Duration::from_secs_f64);
    let lengths_stats = Stats::new(&lengths, |x| *x as f64);

    println!("Success");
    println!(
        "Time Taken to Process Selection\n\t{}",
        times_stats.print_stats()
    );
    println!(
        "Points Inside Window\n\t{}",
        lengths_stats.print_stats()
    );

    Ok(())
}
