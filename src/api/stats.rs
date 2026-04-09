use axum::Json;
use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, db_error};
use super::fillups::ensure_vehicle_exists;

// ── Conversion constants ─────────────────────────────────

const KM_PER_MI: f64 = 1.609_344;
const L_PER_GAL: f64 = 3.785_411_784;

// ── Database row type ────────────────────────────────────

/// Lightweight fillup row for stats calculations.
#[derive(sqlx::FromRow)]
struct StatsRow {
    date: String,
    odometer: Option<f64>,
    fuel_amount: f64,
    fuel_unit: String,
    cost: Option<f64>,
    is_full_tank: i32,
    is_missed: i32,
}

// ── Query parameters ─────────────────────────────────────

/// Optional date-range filter for stats endpoints.
#[derive(Deserialize)]
pub struct StatsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

// ── Response types ───────────────────────────────────────

/// Aggregate vehicle statistics.
#[derive(Serialize)]
pub struct VehicleStats {
    pub total_distance: f64,
    pub total_fuel: f64,
    pub total_cost: f64,
    pub fill_up_count: i64,
    pub average_efficiency: Option<f64>,
    pub average_cost_per_distance: Option<f64>,
    pub distance_unit: String,
    pub volume_unit: String,
    pub currency: String,
}

/// A single efficiency segment for the history endpoint.
#[derive(Serialize)]
pub struct SegmentHistory {
    pub start_date: String,
    pub end_date: String,
    pub start_odometer: f64,
    pub end_odometer: f64,
    pub distance: f64,
    pub fuel: f64,
    pub cost: f64,
    pub efficiency: f64,
    pub cost_per_distance: f64,
    pub is_valid: bool,
    pub distance_unit: String,
    pub volume_unit: String,
    pub currency: String,
}

// ── Internal types ───────────────────────────────────────

/// Internal segment computed from raw fillup data (in stored units).
struct Segment {
    start_date: String,
    end_date: String,
    start_odometer: f64,
    end_odometer: f64,
    distance: f64,
    fuel: f64,
    cost: f64,
    efficiency: f64,
    cost_per_distance: f64,
    is_valid: bool,
}

/// Settings fields needed for stats unit conversion.
#[derive(sqlx::FromRow)]
struct StatsSettings {
    distance_unit: String,
    volume_unit: String,
    currency: String,
}

// ── Validation ───────────────────────────────────────────

/// Validate that a date string looks like an ISO date (YYYY-MM-DD).
///
/// # Errors
///
/// Returns `ApiError::BadRequest` if the string is not a valid date format.
fn validate_date_filter(value: &str) -> Result<(), ApiError> {
    // Accept YYYY-MM-DD format
    if value.len() == 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value[0..4].chars().all(|c| c.is_ascii_digit())
        && value[5..7].chars().all(|c| c.is_ascii_digit())
        && value[8..10].chars().all(|c| c.is_ascii_digit())
    {
        return Ok(());
    }
    Err(ApiError::BadRequest("STATS_INVALID_DATE_FILTER"))
}

// ── Unit conversion helpers ──────────────────────────────

/// Normalize a fuel amount to liters based on its stored unit.
fn normalize_fuel_to_liters(amount: f64, fuel_unit: &str) -> f64 {
    match fuel_unit {
        "gal" => amount * L_PER_GAL,
        _ => amount, // "l" or unknown → treat as liters
    }
}

/// Convert a distance value between units.
fn convert_distance(value: f64, to: &str) -> f64 {
    match to {
        "mi" => value / KM_PER_MI,
        _ => value, // assume stored as km
    }
}

/// Convert a volume value from liters to the target unit.
fn convert_volume(value: f64, to: &str) -> f64 {
    match to {
        "gal" => value / L_PER_GAL,
        _ => value, // already liters
    }
}

/// Round a float to two decimal places.
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// ── Data fetching ────────────────────────────────────────

/// Fetch fillups for stats, ordered chronologically, filtered by date range,
/// excluding rows with zero/negative odometer.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failures.
async fn fetch_fillups_for_stats(
    pool: &SqlitePool,
    vehicle_id: i64,
    query: &StatsQuery,
) -> Result<Vec<StatsRow>, ApiError> {
    let mut sql = String::from(
        "SELECT date, odometer, fuel_amount, fuel_unit, cost, \
         is_full_tank, is_missed FROM fillups \
         WHERE vehicle_id = ? AND (odometer IS NULL OR odometer > 0)",
    );

    if query.from.is_some() {
        sql.push_str(" AND date >= ?");
    }
    if query.to.is_some() {
        sql.push_str(" AND date <= ?");
    }

    sql.push_str(" ORDER BY date ASC, id ASC");

    let mut q = sqlx::query_as::<_, StatsRow>(&sql).bind(vehicle_id);

    if let Some(ref from) = query.from {
        q = q.bind(from);
    }
    if let Some(ref to) = query.to {
        q = q.bind(to);
    }

    q.fetch_all(pool).await.map_err(db_error)
}

/// Read settings needed for stats unit conversion.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failures.
async fn read_stats_settings(pool: &SqlitePool) -> Result<StatsSettings, ApiError> {
    sqlx::query_as::<_, StatsSettings>(
        "SELECT distance_unit, volume_unit, currency FROM settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

// ── Calculation engine ───────────────────────────────────

/// Compute efficiency segments from an ordered list of fillups.
///
/// Uses the tank-to-tank method: segments span between consecutive full-tank
/// fills, accumulating fuel and cost from all intermediate fills. Segments
/// containing missed fills are flagged as invalid.
fn compute_segments(rows: &[StatsRow]) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut segment_start: Option<usize> = None;

    for (i, row) in rows.iter().enumerate() {
        // Skip rows with no valid odometer
        let odo = row.odometer.unwrap_or(0.0);
        if odo <= 0.0 {
            continue;
        }

        if row.is_full_tank != 0 {
            if let Some(start_idx) = segment_start {
                let start_row = &rows[start_idx];
                let start_odo = start_row.odometer.unwrap_or(0.0);
                let distance = odo - start_odo;

                if distance > 0.0 {
                    // Accumulate fuel and cost from all fills after start through
                    // end (inclusive of end, exclusive of start).
                    let mut total_fuel = 0.0;
                    let mut total_cost = 0.0;
                    let mut has_missed = false;

                    for fill in &rows[(start_idx + 1)..=i] {
                        total_fuel += normalize_fuel_to_liters(fill.fuel_amount, &fill.fuel_unit);
                        total_cost += fill.cost.unwrap_or(0.0);
                        if fill.is_missed != 0 {
                            has_missed = true;
                        }
                    }

                    if total_fuel > 0.0 {
                        segments.push(Segment {
                            start_date: start_row.date.clone(),
                            end_date: row.date.clone(),
                            start_odometer: start_odo,
                            end_odometer: odo,
                            distance,
                            fuel: total_fuel,
                            cost: total_cost,
                            efficiency: distance / total_fuel,
                            cost_per_distance: total_cost / distance,
                            is_valid: !has_missed,
                        });
                    }
                }
            }
            // This full-tank fill becomes the start of the next potential segment
            segment_start = Some(i);
        }
    }

    segments
}

/// Aggregate stats from segments and the full fillup list.
#[allow(clippy::cast_possible_wrap)]
fn aggregate_stats(rows: &[StatsRow], segments: &[Segment]) -> AggregatedStats {
    // Total fuel, cost, fill-up count from all rows
    let fill_up_count = rows.len() as i64;
    let total_fuel: f64 = rows
        .iter()
        .map(|r| normalize_fuel_to_liters(r.fuel_amount, &r.fuel_unit))
        .sum();
    let total_cost: f64 = rows.iter().map(|r| r.cost.unwrap_or(0.0)).sum();

    // Total distance: from first to last valid odometer
    let valid_odometers: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.odometer)
        .filter(|&o| o > 0.0)
        .collect();
    let total_distance = if valid_odometers.len() >= 2 {
        valid_odometers.last().unwrap() - valid_odometers.first().unwrap()
    } else {
        0.0
    };

    // Average efficiency and cost_per_distance from valid segments only
    let valid_segments: Vec<&Segment> = segments.iter().filter(|s| s.is_valid).collect();

    let valid_count = valid_segments.len();
    let average_efficiency = if valid_segments.is_empty() {
        None
    } else {
        let sum: f64 = valid_segments.iter().map(|s| s.efficiency).sum();
        #[allow(clippy::cast_precision_loss)]
        let avg = sum / valid_count as f64;
        Some(avg)
    };

    let average_cost_per_distance = if valid_segments.is_empty() {
        None
    } else {
        let sum: f64 = valid_segments.iter().map(|s| s.cost_per_distance).sum();
        #[allow(clippy::cast_precision_loss)]
        let avg = sum / valid_count as f64;
        Some(avg)
    };

    AggregatedStats {
        total_distance,
        total_fuel,
        total_cost,
        fill_up_count,
        average_efficiency,
        average_cost_per_distance,
    }
}

struct AggregatedStats {
    total_distance: f64,
    total_fuel: f64,
    total_cost: f64,
    fill_up_count: i64,
    average_efficiency: Option<f64>,
    average_cost_per_distance: Option<f64>,
}

// ── Handlers ─────────────────────────────────────────────

/// Vehicle summary stats.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::BadRequest` for invalid date filters.
pub async fn summary(
    State(pool): State<SqlitePool>,
    Path(vehicle_id): Path<i64>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<VehicleStats>, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;
    validate_query_dates(&query)?;

    let rows = fetch_fillups_for_stats(&pool, vehicle_id, &query).await?;
    let segments = compute_segments(&rows);
    let agg = aggregate_stats(&rows, &segments);
    let settings = read_stats_settings(&pool).await?;

    Ok(Json(VehicleStats {
        total_distance: round2(convert_distance(
            agg.total_distance,
            &settings.distance_unit,
        )),
        total_fuel: round2(convert_volume(agg.total_fuel, &settings.volume_unit)),
        total_cost: round2(agg.total_cost),
        fill_up_count: agg.fill_up_count,
        average_efficiency: agg.average_efficiency.map(|e| {
            // Efficiency is distance_unit / volume_unit.
            // Raw is km/L. Convert to target units.
            let dist = convert_distance(1.0, &settings.distance_unit);
            let vol = convert_volume(1.0, &settings.volume_unit);
            round2(e * dist / vol)
        }),
        average_cost_per_distance: agg.average_cost_per_distance.map(|c| {
            // Raw is cost/km. Convert to cost/distance_unit.
            let dist = convert_distance(1.0, &settings.distance_unit);
            round2(c / dist)
        }),
        distance_unit: settings.distance_unit,
        volume_unit: settings.volume_unit,
        currency: settings.currency,
    }))
}

/// Vehicle stats history (per-segment data points).
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::BadRequest` for invalid date filters.
pub async fn history(
    State(pool): State<SqlitePool>,
    Path(vehicle_id): Path<i64>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<Vec<SegmentHistory>>, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;
    validate_query_dates(&query)?;

    let rows = fetch_fillups_for_stats(&pool, vehicle_id, &query).await?;
    let segments = compute_segments(&rows);
    let settings = read_stats_settings(&pool).await?;

    let dist_factor = convert_distance(1.0, &settings.distance_unit);
    let vol_factor = convert_volume(1.0, &settings.volume_unit);

    let history: Vec<SegmentHistory> = segments
        .into_iter()
        .map(|s| SegmentHistory {
            start_date: s.start_date,
            end_date: s.end_date,
            start_odometer: round2(convert_distance(s.start_odometer, &settings.distance_unit)),
            end_odometer: round2(convert_distance(s.end_odometer, &settings.distance_unit)),
            distance: round2(convert_distance(s.distance, &settings.distance_unit)),
            fuel: round2(convert_volume(s.fuel, &settings.volume_unit)),
            cost: round2(s.cost),
            efficiency: round2(s.efficiency * dist_factor / vol_factor),
            cost_per_distance: round2(s.cost_per_distance / dist_factor),
            is_valid: s.is_valid,
            distance_unit: settings.distance_unit.clone(),
            volume_unit: settings.volume_unit.clone(),
            currency: settings.currency.clone(),
        })
        .collect();

    Ok(Json(history))
}

/// Validate the `from` and `to` query parameters if present.
///
/// # Errors
///
/// Returns `ApiError::BadRequest` if either date has an invalid format.
fn validate_query_dates(query: &StatsQuery) -> Result<(), ApiError> {
    if let Some(ref from) = query.from {
        validate_date_filter(from)?;
    }
    if let Some(ref to) = query.to {
        validate_date_filter(to)?;
    }
    Ok(())
}
