#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate lazy_static;

pub mod consts;
pub mod tables;
pub mod rpcs;
pub mod stats;
pub mod bounds;
pub mod tests;
pub mod session;

use crate::tables::{
    time_points::{insert::insert_time_point, select as time_points},
    times::select as times,
};

use common::MPQ;
use quickcheck::{Arbitrary, Gen};


const GENERATED_ROWS: usize = 1000;


pub async fn prepare_db(client: &reqwest::Client, g: &mut Gen) {
    for _i in 0..GENERATED_ROWS {
        if let Err(e) = insert_time_point(&client, MPQ::arbitrary(g)).await {
            panic!("Couldn't insert random time point: {:?}", e);
        }
    }

    let num_times = times::select_all(&client).await.unwrap().len();
    let num_time_points = time_points::select_all(&client).await.unwrap().len();
    println!(
        "Size of `times` table: {:?}, Size of `time_points` table: {:?}",
        num_times,
        num_time_points
    );
}

