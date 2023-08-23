use common::{MPQ, Identifier, consts::{REST_DATABASE_HOST, REST_DATABASE_HOST_HEADER}};

use num_rational::BigRational;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct SelectTimePointsAndSummaries {
    session_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct TimePointOrSummaryRow {
    time_point_id: Option<Identifier>,
    time_point_value: Option<MPQ>,
    time_point_timeline: Option<Identifier>,
    summary_min: Option<MPQ>,
    summary_max: Option<MPQ>,
    summary_count: Option<Identifier>,
    summary_visible: Option<Vec<Identifier>>,
    summary_id: Option<Identifier>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TimePointOrSummary {
    TimePoint {
        id: Identifier,
        value: BigRational,
    },
    GeneralSummary {
        min: BigRational,
        max: BigRational,
        count: Identifier,
    },
    Summary {
        min: BigRational,
        max: BigRational,
        count: Identifier,
        visible: Vec<Identifier>,
        id: Identifier,
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
    session_id: String,
) -> Result<Vec<TimePointOrSummary>, String> {
    let params = SelectTimePointsAndSummaries {
        session_id,
    };
    let res = client.post(format!("{}/rpc/select_time_points_and_summaries", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
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
            Ok(body)
        }
        e => Err(format!("Bad return: {:?}", e)),
    }
}
