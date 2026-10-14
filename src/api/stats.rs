use std::collections::BTreeMap;

use axum::Json;
use axum::extract::{RawQuery, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::error::{ApiError, db_error};
use crate::stats::{self, AggregatedStats, Fillup, Segment};

mod response;
use response::{SelectedStats, StatsResponse, Units, VehicleSummary};

/// Optional positive vehicle selection. Unknown parameters are ignored.
#[derive(Debug, Default)]
pub struct StatsQuery {
    pub vehicle_id: Option<i64>,
}

impl StatsQuery {
    fn parse(raw: Option<&str>) -> Result<Self, ApiError> {
        let mut query = Self::default();
        for (key, value) in url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes()) {
            if key == "vehicle_id" {
                if query.vehicle_id.is_some() {
                    return Err(ApiError::BadRequest("STATS_INVALID_VEHICLE_ID"));
                }
                query.vehicle_id = Some(
                    value
                        .parse::<i64>()
                        .ok()
                        .filter(|id| *id > 0)
                        .ok_or(ApiError::BadRequest("STATS_INVALID_VEHICLE_ID"))?,
                );
            }
        }
        Ok(query)
    }
}

#[derive(sqlx::FromRow)]
struct StatsRow {
    vehicle_id: i64,
    id: i64,
    date: String,
    odometer: Option<f64>,
    fuel_amount: f64,
    fuel_unit: String,
    cost: Option<f64>,
    is_full_tank: bool,
    is_missed: bool,
}

impl From<StatsRow> for Fillup {
    fn from(row: StatsRow) -> Self {
        Self {
            id: row.id,
            date: row.date,
            odometer: row.odometer,
            fuel_amount: row.fuel_amount,
            fuel_unit: row.fuel_unit,
            cost: row.cost,
            is_full_tank: row.is_full_tank,
            is_missed: row.is_missed,
        }
    }
}

/// The only full-history read for a statistics request. Include null and zero
/// odometers: price eligibility differs from segment/summary eligibility.
async fn fetch_history(tx: &mut Transaction<'_, Sqlite>) -> Result<Vec<StatsRow>, ApiError> {
    tracing::debug!(target: "gazel::stats", event = "history_fetch", "Reading shared statistics history");
    sqlx::query_as::<_, StatsRow>(
        "SELECT vehicle_id, id, date, odometer, fuel_amount, fuel_unit, cost, \
         is_full_tank, is_missed FROM fillups ORDER BY vehicle_id, date, id",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(db_error)
}

fn calculation_error(error: &stats::CalculationError) -> ApiError {
    tracing::error!("Statistics calculation failed: {error}");
    ApiError::InternalError("INTERNAL_ERROR")
}

fn finite(values: impl IntoIterator<Item = f64>) -> Result<(), stats::CalculationError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(stats::CalculationError)
    }
}

fn validate_rows(rows: &[Fillup]) -> Result<(), stats::CalculationError> {
    for row in rows {
        NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").map_err(|_| stats::CalculationError)?;
        finite(
            [row.fuel_amount]
                .into_iter()
                .chain(row.odometer)
                .chain(row.cost),
        )?;
    }
    Ok(())
}

fn validate_summary(summary: &AggregatedStats) -> Result<(), stats::CalculationError> {
    finite(
        [
            summary.total_distance,
            summary.total_fuel,
            summary.total_cost,
        ]
        .into_iter()
        .chain(summary.average_efficiency)
        .chain(summary.average_cost_per_distance),
    )
}

fn vehicle_calculation(
    rows: &[Fillup],
    units: &Units,
) -> Result<(AggregatedStats, Vec<Segment>), stats::CalculationError> {
    validate_rows(rows)?;
    let segments = stats::compute_segments(rows);
    let mut summary = stats::displayed_summary(
        &stats::aggregate_stats(rows, &segments),
        &units.distance_unit,
        &units.volume_unit,
    );
    summary.average_efficiency =
        stats::displayed_average_efficiency(&segments, &units.distance_unit, &units.volume_unit);
    validate_summary(&summary)?;
    let segments: Vec<_> = segments
        .iter()
        .map(|s| stats::displayed_segment(s, &units.distance_unit, &units.volume_unit))
        .collect();
    for s in &segments {
        finite([
            s.distance,
            s.fuel,
            s.cost,
            s.efficiency,
            s.cost_per_distance,
            s.start_odometer,
            s.end_odometer,
        ])?;
    }
    Ok((summary, segments))
}

async fn build(pool: &SqlitePool, query: StatsQuery) -> Result<StatsResponse, ApiError> {
    let mut tx = pool.begin().await.map_err(db_error)?;
    let ids = sqlx::query_scalar::<_, i64>("SELECT id FROM vehicles ORDER BY id")
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?;
    if query.vehicle_id.is_some_and(|id| !ids.contains(&id)) {
        return Err(ApiError::NotFound("VEHICLE_NOT_FOUND"));
    }
    let units = sqlx::query_as::<_, Units>(
        "SELECT distance_unit, volume_unit, currency FROM settings WHERE id = 1",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_error)?;
    let history = fetch_history(&mut tx).await?;
    tx.commit().await.map_err(db_error)?;
    let mut grouped: BTreeMap<i64, Vec<Fillup>> = BTreeMap::new();
    for row in history {
        grouped.entry(row.vehicle_id).or_default().push(row.into());
    }
    calculate(&ids, &grouped, units, query.vehicle_id).map_err(|error| calculation_error(&error))
}

fn calculate(
    ids: &[i64],
    grouped: &BTreeMap<i64, Vec<Fillup>>,
    units: Units,
    selected_id: Option<i64>,
) -> Result<StatsResponse, stats::CalculationError> {
    let mut summaries = Vec::with_capacity(ids.len());
    let mut vehicles = Vec::with_capacity(ids.len());
    let mut fleet_segments = Vec::new();
    let mut selected = None;
    for &id in ids {
        let rows = grouped.get(&id).map_or(&[][..], Vec::as_slice);
        let (summary, segments) = vehicle_calculation(rows, &units)?;
        let vehicle = VehicleSummary::new(id, &summary);
        if Some(id) == selected_id {
            selected = Some(SelectedStats::new(
                vehicle.clone(),
                rows,
                &segments,
                &units,
            )?);
        }
        fleet_segments.extend(segments.into_iter().filter(|s| s.is_valid).map(|s| (id, s)));
        summaries.push(summary);
        vehicles.push(vehicle);
    }
    fleet_segments.sort_by(|(id_a, a), (id_b, b)| {
        a.end_date
            .cmp(&b.end_date)
            .then(id_a.cmp(id_b))
            .then(a.end_fillup_id.cmp(&b.end_fillup_id))
    });
    let native_values: Vec<_> = fleet_segments.iter().map(|(_, s)| s.efficiency).collect();
    let average_efficiency = if native_values.is_empty() {
        None
    } else {
        let count =
            f64::from(u32::try_from(native_values.len()).map_err(|_| stats::CalculationError)?);
        Some(native_values.iter().sum::<f64>() / count)
    };
    let fleet = response::Fleet::new(&stats::fleet_summary(&summaries), average_efficiency);
    finite(
        [fleet.total_distance, fleet.total_fuel, fleet.total_cost]
            .into_iter()
            .chain(fleet.average_efficiency)
            .chain(fleet.cost_per_distance)
            .chain(fleet.cost_per_volume),
    )?;
    let efficiency_values: Vec<_> = native_values
        .into_iter()
        .map(|v| {
            if units.distance_unit == "km" && units.volume_unit == "l" && v > 0.0 {
                100.0 / v
            } else {
                v
            }
        })
        .collect();
    Ok(StatsResponse {
        units,
        fleet,
        vehicles,
        selected,
        fleet_efficiency: stats::sample_sparkline(&efficiency_values)?,
    })
}

/// Combined exact all-time summaries and bounded selected-vehicle charts.
///
/// # Errors
/// Responds with safe JSON errors for invalid selections (400), missing vehicles
/// (404), and database or calculation failures (500). All handler responses,
/// including errors, prohibit HTTP caching.
pub async fn get_stats(State(pool): State<SqlitePool>, RawQuery(raw): RawQuery) -> Response {
    let result = match StatsQuery::parse(raw.as_deref()) {
        Ok(query) => build(&pool, query).await,
        Err(error) => Err(error),
    };
    let mut response = match result {
        Ok(stats) => Json(stats).into_response(),
        Err(error) => error.into_response(),
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
