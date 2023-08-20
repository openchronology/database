pub mod tables;
pub mod rpcs;
pub mod stats;
pub mod bounds;
pub mod tests;

use crate::tables::{
    time_points,
    times::select as times, timelines::insert as timeline,
};

use common::{MPQ, consts::TEST_USER_USER, session::gen_jwt};
use quickcheck::{Arbitrary, Gen};


const GENERATED_ROWS: usize = 1000;
const GENERATED_TIMELINES: usize = 100;


pub async fn prepare_db(client: &reqwest::Client, g: &mut Gen) {
    let mut timelines = vec![];
    let jwt = &gen_jwt(TEST_USER_USER);
    println!("Using JWT to generate timelines: {:?}", jwt);
    for _i in 0..GENERATED_TIMELINES {
        match timeline::insert(jwt, client).await {
            Err(e) => panic!("Couldn't insert initial timeline: {:?}", e),
            Ok(timeline_id) => {
                timelines.push(timeline_id);
            }
        }
    }

    for _i in 0..GENERATED_ROWS {
        let timeline = timelines[usize::arbitrary(g) % timelines.len()];
        if let Err(e) = time_points::insert::insert(
            jwt,
            &client,
            MPQ::arbitrary(g),
            timeline
        ).await {
            panic!("Couldn't insert random time point: {:?}", e);
        }
    }

    let num_times = times::select_all(&client).await.unwrap().len();
    let num_time_points = time_points::select::select_all(&client).await.unwrap().len();
    println!(
        "Size of `times` table: {:?}, Size of `time_points` table: {:?}",
        num_times,
        num_time_points
    );
}

