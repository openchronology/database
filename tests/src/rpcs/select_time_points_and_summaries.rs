use crate::{
    bounds::MonotonicBounds,
    consts::PGRST_HOST,
};

use common::MPQ;

use num_rational::BigRational;
use num_traits::FromPrimitive;
use serde::{Serialize, Deserialize};


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

pub async fn select_time_points_and_summaries(
    client: &reqwest::Client,
    xs: MonotonicBounds
) -> Result<usize, String> {
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