#[macro_use]
extern crate dotenv_codegen;
#[macro_use]
extern crate lazy_static;
#[cfg(test)]

use common::MPQ;
use num_rational::BigRational;
use num_bigint::BigInt;
use serde::{Serialize, Deserialize};
use quickcheck::{Arbitrary, Gen};
use num_traits::{FromPrimitive, Zero};
use std::time::{Instant, Duration};
use statistical::{mean, median, mode};

lazy_static! {
    static ref PGRST_HOST: &'static str = dotenv!("PGRST_HOST");
    static ref PGRST_JWT_SECRET: &'static str = dotenv!("PGRST_JWT_SECRET");
    static ref PGRST_JWT_AUD: &'static str = dotenv!("PGRST_JWT_AUD");
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct InsertTimePoint {
    value: MPQ,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct SelectTimePointsAndSummaries {
    left_window: MPQ,
    right_window: MPQ,
    threshold: MPQ,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct TimePointOrSummaryRow {
    time_point_id: Option<i32>,
    time_point_value: Option<MPQ>,
    summary_min: Option<MPQ>,
    summary_max: Option<MPQ>,
    summary_count: Option<i32>,
    summary_visible: Option<Vec<i32>>,
    summary_id: Option<i32>,
}

#[derive(Clone, PartialEq, Eq)]
enum TimePointOrSummary {
    TimePoint {
        id: i32,
        value: BigRational,
    },
    GeneralSummary {
        min: BigRational,
        max: BigRational,
        count: i32,
    },
    Summary {
        min: BigRational,
        max: BigRational,
        count: i32,
        visible: Vec<i32>,
        id: i32,
    },
}

impl TimePointOrSummary {
    fn from_row(row: TimePointOrSummaryRow) -> Result<Self, String> {
        match row {
            TimePointOrSummaryRow {
                time_point_id: Some(id),
                time_point_value: Some(MPQ(value)),
                ..
            } => Ok(TimePointOrSummary::TimePoint {id, value}),
            TimePointOrSummaryRow {
                summary_min: Some(MPQ(min)),
                summary_max: Some(MPQ(max)),
                summary_count: Some(count),
                summary_visible: Some(visible),
                summary_id: Some(id),
                ..
            } => Ok(TimePointOrSummary::Summary { min, max, count, visible, id }),
            TimePointOrSummaryRow {
                summary_min: Some(MPQ(min)),
                summary_max: Some(MPQ(max)),
                summary_count: Some(count),
                summary_visible: None,
                summary_id: None,
                ..
            } => Ok(TimePointOrSummary::GeneralSummary { min, max, count }),
            _ => Err(format!("Not enough fields: {:?}", row)),
        }
    }
}

#[derive(Clone, Debug)]
struct MonotonicBounds {
    left: BigRational,
    right: BigRational,
}

impl Arbitrary for MonotonicBounds {
    fn arbitrary(g: &mut Gen) -> Self {
        let n1 = BigInt::arbitrary(g);
        let d1 = BigInt::arbitrary(g);
        let d1 = if d1.is_zero() {
            d1 + BigInt::from_u8(1_u8).unwrap()
        } else {d1};
        let n2 = BigInt::arbitrary(g);
        let d2 = BigInt::arbitrary(g);
        let d2 = if d2.is_zero() {
            d2 + BigInt::from_u8(1_u8).unwrap()
        } else {d2};
        let x = BigRational::new_raw(n1, d1);
        let y = BigRational::new_raw(n2, d2);
        if x < y {
            MonotonicBounds{left: x, right: y}
        } else if y < x {
            MonotonicBounds{left: y, right: x}
        } else {
            MonotonicBounds{left: x, right: y + BigRational::from_u8(1u8).unwrap()}
        }
    }
}

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let mut g = Gen::new(10);

    for _i in 0..1000 {
        let x = InsertTimePoint { value: MPQ::arbitrary(&mut g) };

        let res = client.post(format!("{}/time_points", *PGRST_HOST))
            .json(&x)
            .send()
            .await;

        match res {
            reqwest::Result::Ok(resp) if resp.status().as_u16() / 100 == 2 => {
            }
            e => {
                panic!("Couldn't insert random time point: {:?}", e);
            }
        }
    }

    let mut times: Vec<Duration> = vec![];
    let mut lengths: Vec<usize> = vec![];

    for i in 0..100 {
        let sample = MonotonicBounds::arbitrary(&mut g);
        let now = Instant::now();
        let result = test_select_time_points_and_summaries(sample.clone()).await;
        match result {
            Ok(l) => {
                times.push(now.elapsed());
                lengths.push(l);
            }
            Err(e) => panic!("Returned test case is an error: {:?}\nSample: {:?}\nIteration: {:?}", e, sample, i),
        }
    }

    let max_times = times.clone().into_iter().max();
    let min_times = times.clone().into_iter().min();
    let mean_times = Duration::from_secs_f64(
        mean(&times.clone().into_iter().map(|x| x.as_secs_f64()).collect::<Vec<f64>>())
    );
    let median_times = Duration::from_secs_f64(
        median(&times.clone().into_iter().map(|x| x.as_secs_f64()).collect::<Vec<f64>>())
    );
    let mode_times = mode(&times.clone());

    let max_lengths = lengths.clone().into_iter().max();
    let min_lengths = lengths.clone().into_iter().min();
    let mean_lengths = mean(
        &lengths.clone().into_iter().map(|x| x as f64).collect::<Vec<f64>>()
    );
    let median_lengths = median(
        &lengths.clone().into_iter().map(|x| x as f64).collect::<Vec<f64>>()
    );
    let mode_lengths = mode(&lengths.clone());

    println!("Tests succeeded");
    println!(
        "Time Taken to Process Selection\n\tMax: {:?}\n\tMin: {:?}\n\tMean: {:?}\n\tMedian: {:?}\n\tMode: {:?}",
        max_times,
        min_times,
        mean_times,
        median_times,
        mode_times
    );
    println!(
        "Points Inside Window\n\tMax: {:?}\n\tMin: {:?}\n\tMean: {:?}\n\tMedian: {:?}\n\tMode: {:?}",
        max_lengths,
        min_lengths,
        mean_lengths,
        median_lengths,
        mode_lengths
    );
    let num_times = client.get(format!("{}/times", *PGRST_HOST))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap()
        .len();
    let num_time_points = client.get(format!("{}/time_points", *PGRST_HOST))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap()
        .len();
    println!("Size of `times` table: {:?}, Size of `time_points` table: {:?}", num_times, num_time_points);
}


async fn test_select_time_points_and_summaries(xs: MonotonicBounds) -> Result<usize, String> {
    let client = reqwest::Client::new();
    let params = SelectTimePointsAndSummaries {
        left_window: MPQ(xs.left.clone()),
        right_window: MPQ(xs.right.clone()),
        threshold: {
            let t = (xs.right.clone() - xs.left.clone())
                / BigRational::from_u8(10u8).unwrap();
            MPQ(t)
        },
    };
    let res = client.post(format!("{}/rpc/select_time_points_and_summaries", *PGRST_HOST))
        .json(&params)
        .send()
        .await;
    match res {
        reqwest::Result::Ok(resp) if resp.status().as_u16() / 100 == 2 => {
            let value: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Json decoding to value error: {:?}", e))?;
            let body: Vec<TimePointOrSummaryRow> = serde_json::from_value(value.clone())
                .map_err(|e| format!("Json decoding from value error: {:?} - value: {:?}", e, value))?;
            let body: Vec<TimePointOrSummary> = body
                .into_iter()
                .map(|x| TimePointOrSummary::from_row(x).map_err(|e| {
                    format!("{}, original value: {:?}", e, value)
                }))
                .collect::<Result<Vec<TimePointOrSummary>, String>>()?;
            if body.iter().all(|t| match t {
                TimePointOrSummary::TimePoint {value, ..} =>
                    value <= &xs.right && value >= &xs.left,
                TimePointOrSummary::GeneralSummary {min, max, ..} =>
                    (min <= &xs.right && min >= &xs.left)
                    || (max <= &xs.right && max >= &xs.left),
                TimePointOrSummary::Summary {min, max, ..} =>
                    (min <= &xs.right && min >= &xs.left)
                    || (max <= &xs.right && max >= &xs.left),
            }) {
                Ok(body.len())
            } else {
                Err("Outside of window".to_owned())
            }
        }
        e => Err(format!("Bad return: {:?}", e)),
    }
}
