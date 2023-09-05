use anyhow::{Result, ensure, Context, bail};
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
    summary_next_threshold: Option<MPQ>,
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
        next_threshold: BigRational,
    },
    Summary {
        min: BigRational,
        max: BigRational,
        count: Identifier,
        next_threshold: BigRational,
        visible: Vec<Identifier>,
        id: Identifier,
    },
}

impl TimePointOrSummary {
    fn from_row(row: TimePointOrSummaryRow) -> Result<Self> {
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
                summary_next_threshold: Some(MPQ(next_threshold)),
                summary_visible: Some(visible),
                summary_id: Some(id),
                ..
            } => Ok(TimePointOrSummary::Summary { min, max, count, next_threshold, visible, id }),
            TimePointOrSummaryRow {
                summary_min: Some(MPQ(min)),
                summary_max: Some(MPQ(max)),
                summary_count: Some(count),
                summary_next_threshold: Some(MPQ(next_threshold)),
                summary_visible: None,
                summary_id: None,
                ..
            } => Ok(TimePointOrSummary::GeneralSummary { min, max, count, next_threshold }),
            _ => bail!("Not enough fields: {row:?}"),
        }
    }
}

pub async fn select_time_points_and_summaries(
    client: &reqwest::Client,
    session_id: String,
) -> Result<Vec<TimePointOrSummary>> {
    let params = SelectTimePointsAndSummaries {
        session_id,
    };
    let resp = client.post(format!("{}/rpc/select_time_points_and_summaries", *REST_DATABASE_HOST))
        .header("Host", (*REST_DATABASE_HOST_HEADER).clone())
        .json(&params)
        .send()
        .await?;
    ensure!(resp.status().as_u16() / 100 == 2, "Not a 2xx response code");
    let value: serde_json::Value = resp
        .json()
        .await?;
    let body: Vec<TimePointOrSummaryRow> =
        serde_json::from_value(value.clone())
        .context("Json decoding error - value: {value:?}")?;
    let body: Vec<TimePointOrSummary> = body
        .into_iter()
        .map(|x| TimePointOrSummary::from_row(x).context("original value: {value:?}"))
        .collect::<Result<Vec<TimePointOrSummary>>>()?;
    Ok(body)
}
