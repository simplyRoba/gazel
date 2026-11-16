use std::collections::BTreeMap;
use std::fmt;

use chrono::{Datelike, NaiveDate};

use super::Segment;

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PeriodSeries {
    pub points: Vec<PeriodPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Period {
    Month,
    Year,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PeriodPoint {
    pub period: String,
    pub value: f64,
}

/// Invalid calendar input or nonfinite arithmetic.
#[derive(Debug, PartialEq, Eq)]
pub struct CalculationError;

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid statistics calendar or arithmetic input")
    }
}

impl std::error::Error for CalculationError {}

fn parse_date(value: &str) -> Result<NaiveDate, CalculationError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| CalculationError)
}

fn period_start(date: NaiveDate, period: Period) -> Result<NaiveDate, CalculationError> {
    NaiveDate::from_ymd_opt(
        date.year(),
        if period == Period::Month {
            date.month()
        } else {
            1
        },
        1,
    )
    .ok_or(CalculationError)
}

fn next_period(date: NaiveDate, period: Period) -> Result<NaiveDate, CalculationError> {
    let (year, month) = if period == Period::Month && date.month() < 12 {
        (date.year(), date.month() + 1)
    } else {
        (date.year().checked_add(1).ok_or(CalculationError)?, 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).ok_or(CalculationError)
}

fn add(
    map: &mut BTreeMap<NaiveDate, f64>,
    key: NaiveDate,
    value: f64,
) -> Result<(), CalculationError> {
    let sum = map.entry(key).or_default();
    *sum += value;
    if sum.is_finite() {
        Ok(())
    } else {
        Err(CalculationError)
    }
}

fn points(
    mut map: BTreeMap<NaiveDate, f64>,
    period: Period,
    round: bool,
) -> Result<Vec<PeriodPoint>, CalculationError> {
    if let (Some((&first, _)), Some((&last, _))) = (map.first_key_value(), map.last_key_value()) {
        let mut cursor = first;
        while cursor < last {
            map.entry(cursor).or_default();
            cursor = next_period(cursor, period)?;
        }
    }
    Ok(map
        .into_iter()
        .map(|(date, value)| PeriodPoint {
            period: date
                .format(if period == Period::Month {
                    "%Y-%m"
                } else {
                    "%Y"
                })
                .to_string(),
            value: if round { value.round() } else { value },
        })
        .collect())
}

/// Sum costs into each segment's ending period, including invalid efficiency
/// segments. Input must contain converted, ROUND2 history fields (see
/// `displayed_segment`). Both modes zero-fill gaps only within the source span.
///
/// # Errors
/// Returns an error for invalid dates, calendar overflow or nonfinite totals.
pub fn period_costs(
    segments: &[Segment],
    period: Period,
) -> Result<Vec<PeriodPoint>, CalculationError> {
    let mut map = BTreeMap::new();
    for segment in segments {
        add(
            &mut map,
            period_start(parse_date(&segment.end_date)?, period)?,
            segment.cost,
        )?;
    }
    points(map, period, false)
}

/// Allocate converted, rounded distances by inclusive civil days. Round final
/// period totals to integers; yearly allocation is direct, not summed months.
/// Both modes include zero-filled gaps and invalid efficiency segments.
///
/// # Errors
/// Returns an error for invalid/reversed dates, calendar overflow or nonfinite totals.
pub fn period_distances(
    segments: &[Segment],
    period: Period,
) -> Result<Vec<PeriodPoint>, CalculationError> {
    let mut map = BTreeMap::new();
    for segment in segments {
        let start = parse_date(&segment.start_date)?;
        let end = parse_date(&segment.end_date)?;
        if end < start || !segment.distance.is_finite() || segment.distance < 0.0 {
            return Err(CalculationError);
        }
        let end_excl = end.succ_opt().ok_or(CalculationError)?;
        let days =
            f64::from(i32::try_from((end_excl - start).num_days()).map_err(|_| CalculationError)?);
        let mut cursor = start;
        while cursor < end_excl {
            let key = period_start(cursor, period)?;
            let next = next_period(key, period)?;
            let slice_end = next.min(end_excl);
            let slice_days = f64::from(
                i32::try_from((slice_end - cursor).num_days()).map_err(|_| CalculationError)?,
            );
            add(&mut map, key, segment.distance * (slice_days / days))?;
            cursor = slice_end;
        }
    }
    points(map, period, true)
}
