use crate::{
    stats::Stats,
    bounds::MonotonicBounds,
    tables::sessions,
    rpcs::select_time_points_and_summaries::{select_time_points_and_summaries, TimePointOrSummary},
};

use std::time::{Instant, Duration};
use common::MPQ;
use quickcheck::{Arbitrary, Gen};
use color_print::cprintln;
use num_rational::BigRational;
use num_traits::FromPrimitive;


const NUM_TESTS: usize = 100;

pub async fn verify_select_time_points_and_summaries(
    client: &reqwest::Client,
    g: &mut Gen,
) -> Result<(), String> {
    print!("Verify `select_time_points_and_summaries`... ");

    let mut times: Vec<Duration> = vec![];
    let mut lengths: Vec<usize> = vec![];

    for i in 0..NUM_TESTS {
        let now = Instant::now();
        let bounds = MonotonicBounds::arbitrary(g);
        let zoom = (bounds.right.clone() - bounds.left.clone()) / BigRational::from_u8(2).unwrap();
        let pos = bounds.left.clone() + zoom.clone();
        let left = bounds.left;
        let right = bounds.right;
        let session = sessions::insert::insert(client).await?;

        sessions::update::update(client, session.clone(), sessions::update::UpdateSession {
            pos: Some(MPQ(pos.clone())),
            zoom: Some(MPQ(zoom.clone())),
            ..sessions::update::UpdateSession::default()
        }).await.map_err(|e| format!("Couldn't update session - {e:?}\npos: {pos:?}\nzoom: {zoom:?}"))?;

        let result = select_time_points_and_summaries(
            &client,
            // MPQ(pos.clone()),
            // MPQ(zoom.clone()),
            session
        ).await;
        match result {
            Ok(xs) => {
                if xs.iter().all(|t| match t {
                    TimePointOrSummary::TimePoint {value, ..} =>
                        value <= &right && value >= &left,
                    TimePointOrSummary::GeneralSummary {min, max, ..} =>
                        (min <= &right && min >= &left)
                        || (max <= &right && max >= &left),
                    TimePointOrSummary::Summary {min, max, ..} =>
                        (min <= &right && min >= &left)
                        || (max <= &right && max >= &left),
                }) {
                    times.push(now.elapsed());
                    lengths.push(xs.len());
                } else {
                    return Err(
                        format!(
                            "Outside of window - test case {i} - xs: {xs:?} - left: {left} - right: {right}"
                        )
                    );
                }
            }
            Err(e) => return Err(
                format!(
                    "Test case returned an error: {e:?}\nSample: {pos:?} - {zoom:?}\nIteration: {i}",
                )
            ),
        }
    }

    let times_stats = Stats::new(&times, Duration::as_secs_f64)
        .resolve(Duration::from_secs_f64);
    let lengths_stats = Stats::new(&lengths, |x| *x as f64);

    cprintln!("<green>Success</green>");
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
