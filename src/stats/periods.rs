use std::collections::BTreeMap;
use std::fmt;

use chrono::{Datelike, NaiveDate};

use super::Segment;

pub const PERIOD_LIMIT: usize = 12;

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct PeriodRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PeriodSeries {
    pub points: Vec<PeriodPoint>,
    /// Calendar periods in the complete source span, including gaps.
    pub source_count: usize,
    pub source_range: Option<PeriodRange>,
    pub displayed_range: Option<PeriodRange>,
    pub omitted_older_periods: bool,
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

/// Invalid calendar input or nonfinite arithmetic; adapters can translate this
/// into their safe infrastructure error without exposing stored data.
#[derive(Debug, PartialEq, Eq)]
pub struct CalculationError;

impl fmt::Display for CalculationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Invalid statistics calendar or arithmetic input")
    }
}

impl std::error::Error for CalculationError {}

fn label(date: NaiveDate, period: Period) -> String {
    date.format(if period == Period::Month {
        "%Y-%m"
    } else {
        "%Y"
    })
    .to_string()
}

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
    // Match existing helpers: monthly gaps are zero-filled, yearly data is sparse.
    // Latest-window/year gap filling is intentionally left to the bounding layer.
    if period == Period::Month
        && let (Some((&first, _)), Some((&last, _))) = (map.first_key_value(), map.last_key_value())
    {
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
/// `displayed_segment`). Monthly gaps are filled; yearly output is sparse.
///
/// # Errors
/// Returns an error for invalid ending dates or nonfinite cost totals.
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

/// Allocate converted, ROUND2 segment distances by inclusive civil calendar
/// days, then integer-round period totals. Includes invalid efficiency segments.
/// Yearly allocation is direct, never a sum of rounded monthly allocations.
/// Monthly gaps are zero-filled; yearly output matches the sparse frontend helper.
///
/// # Errors
/// Returns an error for invalid/reversed dates, calendar boundary overflow,
/// or nonfinite distances or totals.
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

fn previous_period(date: NaiveDate, period: Period) -> Result<NaiveDate, CalculationError> {
    let (year, month) = if period == Period::Month && date.month() > 1 {
        (date.year(), date.month() - 1)
    } else {
        (
            date.year().checked_sub(1).ok_or(CalculationError)?,
            if period == Period::Month { 12 } else { 1 },
        )
    };
    NaiveDate::from_ymd_opt(year, month, 1).ok_or(CalculationError)
}

fn source_bounds(
    segments: &[Segment],
    period: Period,
    distance: bool,
) -> Result<Option<(NaiveDate, NaiveDate)>, CalculationError> {
    let mut bounds: Option<(NaiveDate, NaiveDate)> = None;
    for segment in segments {
        let end = parse_date(&segment.end_date)?;
        let start = if distance {
            parse_date(&segment.start_date)?
        } else {
            end
        };
        if start > end
            || (distance && (!segment.distance.is_finite() || segment.distance < 0.0))
            || (!distance && !segment.cost.is_finite())
        {
            return Err(CalculationError);
        }
        let first = period_start(start, period)?;
        let last = period_start(end, period)?;
        bounds = Some(bounds.map_or((first, last), |(low, high)| {
            (low.min(first), high.max(last))
        }));
    }
    Ok(bounds)
}

fn bounded_periods(
    segments: &[Segment],
    period: Period,
    distance: bool,
) -> Result<PeriodSeries, CalculationError> {
    let Some((first, last)) = source_bounds(segments, period, distance)? else {
        return Ok(PeriodSeries {
            points: Vec::new(),
            source_count: 0,
            source_range: None,
            displayed_range: None,
            omitted_older_periods: false,
        });
    };
    let mut map = BTreeMap::new();
    let mut visible_start = last;
    map.insert(last, 0.0);
    for _ in 1..PERIOD_LIMIT {
        if visible_start <= first {
            break;
        }
        visible_start = previous_period(visible_start, period)?;
        map.insert(visible_start, 0.0);
    }
    for segment in segments {
        let end = parse_date(&segment.end_date)?;
        if !distance {
            let key = period_start(end, period)?;
            if key >= visible_start {
                add(&mut map, key, segment.cost)?;
            }
            continue;
        }
        let start = parse_date(&segment.start_date)?;
        let end_excl = end.succ_opt().ok_or(CalculationError)?;
        let days =
            f64::from(i32::try_from((end_excl - start).num_days()).map_err(|_| CalculationError)?);
        // Iterate only the visible window, but keep the FULL interval denominator.
        for (key, total) in &mut map {
            let slice_start = start.max(*key);
            let slice_end = end_excl.min(next_period(*key, period)?);
            if slice_start < slice_end {
                let slice_days = f64::from(
                    i32::try_from((slice_end - slice_start).num_days())
                        .map_err(|_| CalculationError)?,
                );
                *total += segment.distance * (slice_days / days);
                if !total.is_finite() {
                    return Err(CalculationError);
                }
            }
        }
    }
    let years = last.year() - first.year();
    let span = if period == Period::Month {
        years * 12 + i32::try_from(last.month()).map_err(|_| CalculationError)?
            - i32::try_from(first.month()).map_err(|_| CalculationError)?
    } else {
        years
    };
    Ok(PeriodSeries {
        points: points(map, period, distance)?,
        source_count: usize::try_from(span + 1).map_err(|_| CalculationError)?,
        source_range: Some(PeriodRange {
            start: label(first, period),
            end: label(last, period),
        }),
        displayed_range: Some(PeriodRange {
            start: label(visible_start, period),
            end: label(last, period),
        }),
        omitted_older_periods: visible_start > first,
    })
}

/// Sum ending-period costs into the latest data-anchored 12 calendar periods.
/// Zero-fill only inside the displayed window, without padding short histories.
/// Input conversion/rounding and invalid-efficiency eligibility match `period_costs`.
/// No complete zero-filled historical series is materialized.
///
/// # Errors
/// Returns an error for invalid ending dates, calendar overflow or nonfinite costs/totals.
pub fn bounded_period_costs(
    segments: &[Segment],
    period: Period,
) -> Result<PeriodSeries, CalculationError> {
    bounded_periods(segments, period, false)
}

/// Allocate distances into the latest data-anchored 12 calendar periods using
/// each FULL segment's inclusive calendar-day denominator. Integer-round only
/// final period totals; zero-fill gaps without padding before the source start.
/// Input conversion and invalid-efficiency eligibility match `period_distances`.
///
/// # Errors
/// Returns an error for invalid/reversed dates, calendar overflow or nonfinite distances/totals.
pub fn bounded_period_distances(
    segments: &[Segment],
    period: Period,
) -> Result<PeriodSeries, CalculationError> {
    bounded_periods(segments, period, true)
}
