use crate::{
    stats::Stats,
    bounds::MonotonicBounds,
    tables::sessions::{self, select::select_threshold},
    rpcs::select_time_points_and_summaries::{select_time_points_and_summaries, TimePointOrSummary},
};

use std::time::{Instant, Duration};
use common::{
    MPQ,
    consts::DEFAULT_FIELD,
    window::{
        get_pos_and_zoom,
        SessionWindow,
        adjust_zoom,
        get_threshold,
    },
};
use quickcheck::{Arbitrary, Gen};
use color_print::cprintln;
use num_rational::BigRational;
use num_traits::FromPrimitive;


const NUM_TESTS: usize = 1000;

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
        let (pos, zoom) = get_pos_and_zoom(bounds.left.clone(), bounds.right.clone());

        let session = sessions::insert::insert(None, client).await?;
        sessions::update::update(None, client, session.clone(), sessions::update::UpdateSession {
            pos: Some(MPQ(pos.clone())),
            zoom: Some(MPQ(zoom.clone())),
            ..sessions::update::UpdateSession::default()
        }).await.map_err(|e| format!("Couldn't update session - {e:?}\npos: {pos:?}\nzoom: {zoom:?}"))?;

        let expected_threshold = get_threshold(zoom.clone(), (*DEFAULT_FIELD).clone());
        let actual_threshold = select_threshold(None, client, session.clone()).await?;
        assert_eq!(expected_threshold, actual_threshold.0);


        let result = select_time_points_and_summaries(
            &client,
            session.clone()
        ).await;
        match result {
            Ok(xs) => {
                for x in xs.iter() {
                    // guard test case to be within window
                    assert!(
                        match x {
                            TimePointOrSummary::TimePoint {value, ..} =>
                                value <= &bounds.right && value >= &bounds.left,
                            TimePointOrSummary::GeneralSummary {min, max, ..}
                            | TimePointOrSummary::Summary {min, max, ..} =>
                                (min <= &bounds.right && min >= &bounds.left)
                                || (max <= &bounds.right && max >= &bounds.left)
                        },
                        "Outside of window - test case {i} - x: {x:?} - left: {} - right: {}",
                        bounds.left,
                        bounds.right,
                    );
                    match x {
                        TimePointOrSummary::GeneralSummary { next_threshold, min, max, .. }
                        | TimePointOrSummary::Summary { next_threshold, min, max, .. } => {
                            let current_window = SessionWindow {
                                pos: pos.clone(),
                                zoom: zoom.clone(),
                                field: (*DEFAULT_FIELD).clone(),
                            };
                            match adjust_zoom(current_window.clone(), next_threshold.clone()) {
                                None => {
                                    return Err(
                                        format!(
                                            "Next threshold isn't smaller than current one: {next_threshold:?} - {current_window:?} - iteration: {i}"
                                        )
                                    )
                                }
                                Some(new_zoom) => {
                                    let (new_pos, _) = get_pos_and_zoom(min.clone(), max.clone());

                                    sessions::update::update(
                                        None,
                                        client,
                                        session.clone(),
                                        sessions::update::UpdateSession {
                                            pos: Some(MPQ(new_pos.clone())),
                                            zoom: Some(MPQ(new_zoom.clone())),
                                            ..sessions::update::UpdateSession::default()
                                        }
                                    ).await.map_err(|e| format!(
                                        "Couldn't update session - {e:?}\npos: {pos:?}\nzoom: {zoom:?}"
                                    )).unwrap();

                                    let new_threshold =
                                        select_threshold(None, client, session.clone())
                                        .await.unwrap();

                                    assert_eq!(
                                        *next_threshold,
                                        new_threshold.0,
                                        "next_threshold: {next_threshold:?} != new_threshold: {new_threshold:?}"
                                    );

                                    // TODO re-query and verify summary doesn't exist

                                    sessions::update::update(
                                        None,
                                        client,
                                        session.clone(),
                                        sessions::update::UpdateSession {
                                            pos: Some(MPQ(pos.clone())),
                                            zoom: Some(MPQ(zoom.clone())),
                                            ..sessions::update::UpdateSession::default()
                                        }
                                    ).await.map_err(|e| format!(
                                        "Couldn't update session - {e:?}\npos: {pos:?}\nzoom: {zoom:?}"
                                    )).unwrap();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                times.push(now.elapsed());
                lengths.push(xs.len());
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
