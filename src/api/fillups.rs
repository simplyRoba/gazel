use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, info};

use super::error::{ApiError, JsonBody, db_error};

// ── Response type ────────────────────────────────────────

/// Fill-up as returned by the API.
#[derive(Serialize)]
pub struct Fillup {
    pub id: i64,
    pub vehicle_id: i64,
    pub date: String,
    pub odometer: Option<f64>,
    pub fuel_amount: f64,
    pub fuel_unit: String,
    pub cost: Option<f64>,
    pub currency: Option<String>,
    pub is_full_tank: bool,
    pub is_missed: bool,
    pub station: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── Database row type ────────────────────────────────────

/// Fill-up row as stored in `SQLite`.
#[derive(sqlx::FromRow)]
struct FillupRow {
    id: i64,
    vehicle_id: i64,
    date: String,
    odometer: Option<f64>,
    fuel_amount: f64,
    fuel_unit: String,
    cost: Option<f64>,
    currency: Option<String>,
    is_full_tank: i32,
    is_missed: i32,
    station: Option<String>,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<FillupRow> for Fillup {
    fn from(row: FillupRow) -> Self {
        Self {
            id: row.id,
            vehicle_id: row.vehicle_id,
            date: row.date,
            odometer: row.odometer,
            fuel_amount: row.fuel_amount,
            fuel_unit: row.fuel_unit,
            cost: row.cost,
            currency: row.currency,
            is_full_tank: row.is_full_tank != 0,
            is_missed: row.is_missed != 0,
            station: row.station,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// ── Shared SQL ───────────────────────────────────────────

const FILLUP_SELECT: &str = "SELECT id, vehicle_id, date, odometer, fuel_amount, \
    fuel_unit, cost, currency, is_full_tank, is_missed, station, notes, \
    created_at, updated_at FROM fillups";

// ── Request types ────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateFillup {
    pub date: Option<String>,
    pub odometer: Option<f64>,
    pub fuel_amount: Option<f64>,
    pub fuel_unit: Option<String>,
    pub cost: Option<f64>,
    pub currency: Option<String>,
    pub is_full_tank: Option<bool>,
    pub is_missed: Option<bool>,
    pub station: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateFillup {
    pub date: String,
    pub odometer: Option<f64>,
    pub fuel_amount: f64,
    pub fuel_unit: Option<String>,
    pub cost: Option<f64>,
    pub currency: Option<String>,
    pub is_full_tank: Option<bool>,
    pub is_missed: Option<bool>,
    pub station: Option<String>,
    pub notes: Option<String>,
}

// ── Validation ───────────────────────────────────────────

/// # Errors
///
/// Returns `ApiError::Validation` if the date is empty or whitespace-only.
fn validate_fillup_date(date: &str) -> Result<(), ApiError> {
    if date.trim().is_empty() {
        return Err(ApiError::Validation("FILLUP_DATE_REQUIRED"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the fuel amount is not positive.
fn validate_fuel_amount(amount: f64) -> Result<(), ApiError> {
    if amount <= 0.0 {
        return Err(ApiError::Validation("FILLUP_INVALID_FUEL_AMOUNT"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the odometer is less than the max
/// existing reading for the vehicle.
async fn validate_odometer(
    pool: &SqlitePool,
    vehicle_id: i64,
    odometer: Option<f64>,
    exclude_id: Option<i64>,
) -> Result<(), ApiError> {
    let Some(value) = odometer else {
        return Ok(());
    };

    let max_odometer: Option<f64> = if let Some(eid) = exclude_id {
        sqlx::query_scalar("SELECT MAX(odometer) FROM fillups WHERE vehicle_id = ? AND id != ?")
            .bind(vehicle_id)
            .bind(eid)
            .fetch_one(pool)
            .await
            .map_err(db_error)?
    } else {
        sqlx::query_scalar("SELECT MAX(odometer) FROM fillups WHERE vehicle_id = ?")
            .bind(vehicle_id)
            .fetch_one(pool)
            .await
            .map_err(db_error)?
    };

    if let Some(max) = max_odometer
        && value < max
    {
        return Err(ApiError::Validation("FILLUP_INVALID_ODOMETER"));
    }

    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the cost is negative.
fn validate_cost(cost: Option<f64>) -> Result<(), ApiError> {
    if let Some(c) = cost
        && c < 0.0
    {
        return Err(ApiError::Validation("FILLUP_INVALID_COST"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist.
async fn ensure_vehicle_exists(pool: &SqlitePool, vehicle_id: i64) -> Result<(), ApiError> {
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM vehicles WHERE id = ?")
        .bind(vehicle_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    if exists.is_none() {
        return Err(ApiError::NotFound("VEHICLE_NOT_FOUND"));
    }

    Ok(())
}

// ── Handlers ─────────────────────────────────────────────

/// List all fill-ups for a vehicle, sorted by date descending.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist.
pub async fn list(
    State(pool): State<SqlitePool>,
    Path(vehicle_id): Path<i64>,
) -> Result<Json<Vec<Fillup>>, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    let query = format!("{FILLUP_SELECT} WHERE vehicle_id = ? ORDER BY date DESC, id DESC");
    let rows = sqlx::query_as::<_, FillupRow>(&query)
        .bind(vehicle_id)
        .fetch_all(&pool)
        .await
        .map_err(db_error)?;

    Ok(Json(rows.into_iter().map(Fillup::from).collect()))
}

/// Get a single fill-up by ID, scoped to a vehicle.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle or fill-up does not exist.
pub async fn get(
    State(pool): State<SqlitePool>,
    Path((vehicle_id, id)): Path<(i64, i64)>,
) -> Result<Json<Fillup>, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    let query = format!("{FILLUP_SELECT} WHERE id = ? AND vehicle_id = ?");
    let row = sqlx::query_as::<_, FillupRow>(&query)
        .bind(id)
        .bind(vehicle_id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?
        .ok_or(ApiError::NotFound("FILLUP_NOT_FOUND"))?;

    Ok(Json(Fillup::from(row)))
}

/// Create a new fill-up for a vehicle.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::Validation` if fields are invalid.
pub async fn create(
    State(pool): State<SqlitePool>,
    Path(vehicle_id): Path<i64>,
    JsonBody(body): JsonBody<CreateFillup>,
) -> Result<(StatusCode, Json<Fillup>), ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    let date = body
        .date
        .ok_or(ApiError::Validation("FILLUP_DATE_REQUIRED"))?;
    validate_fillup_date(&date)?;
    let date = date.trim().to_string();

    let fuel_amount = body
        .fuel_amount
        .ok_or(ApiError::Validation("FILLUP_FUEL_AMOUNT_REQUIRED"))?;
    validate_fuel_amount(fuel_amount)?;
    validate_odometer(&pool, vehicle_id, body.odometer, None).await?;
    validate_cost(body.cost)?;

    let fuel_unit = body
        .fuel_unit
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "liters".to_string());
    let is_full_tank = i32::from(body.is_full_tank.unwrap_or(false));
    let is_missed = i32::from(body.is_missed.unwrap_or(false));

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO fillups (vehicle_id, date, odometer, fuel_amount, fuel_unit, \
         cost, currency, is_full_tank, is_missed, station, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(vehicle_id)
    .bind(&date)
    .bind(body.odometer)
    .bind(fuel_amount)
    .bind(&fuel_unit)
    .bind(body.cost)
    .bind(&body.currency)
    .bind(is_full_tank)
    .bind(is_missed)
    .bind(&body.station)
    .bind(&body.notes)
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .map_err(db_error)?;

    let query = format!("{FILLUP_SELECT} WHERE id = ?");
    let row = sqlx::query_as::<_, FillupRow>(&query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    let fillup = Fillup::from(row);
    info!(fillup_id = id, vehicle_id, "Fill-up created");

    Ok((StatusCode::CREATED, Json(fillup)))
}

/// Full update (PUT) of a fill-up.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle or fill-up does not exist, or
/// `ApiError::Validation` if fields are invalid.
pub async fn update(
    State(pool): State<SqlitePool>,
    Path((vehicle_id, id)): Path<(i64, i64)>,
    JsonBody(body): JsonBody<UpdateFillup>,
) -> Result<Json<Fillup>, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    // Check fillup exists and belongs to vehicle
    let exists_query = format!("{FILLUP_SELECT} WHERE id = ? AND vehicle_id = ?");
    sqlx::query_as::<_, FillupRow>(&exists_query)
        .bind(id)
        .bind(vehicle_id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?
        .ok_or(ApiError::NotFound("FILLUP_NOT_FOUND"))?;

    validate_fillup_date(&body.date)?;
    let date = body.date.trim().to_string();
    validate_fuel_amount(body.fuel_amount)?;
    validate_odometer(&pool, vehicle_id, body.odometer, Some(id)).await?;
    validate_cost(body.cost)?;

    let fuel_unit = body
        .fuel_unit
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "liters".to_string());
    let is_full_tank = i32::from(body.is_full_tank.unwrap_or(false));
    let is_missed = i32::from(body.is_missed.unwrap_or(false));

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query(
        "UPDATE fillups SET date = ?, odometer = ?, fuel_amount = ?, fuel_unit = ?, \
         cost = ?, currency = ?, is_full_tank = ?, is_missed = ?, station = ?, \
         notes = ?, updated_at = ? WHERE id = ? AND vehicle_id = ?",
    )
    .bind(&date)
    .bind(body.odometer)
    .bind(body.fuel_amount)
    .bind(&fuel_unit)
    .bind(body.cost)
    .bind(&body.currency)
    .bind(is_full_tank)
    .bind(is_missed)
    .bind(&body.station)
    .bind(&body.notes)
    .bind(&now)
    .bind(id)
    .bind(vehicle_id)
    .execute(&pool)
    .await
    .map_err(db_error)?;

    let query = format!("{FILLUP_SELECT} WHERE id = ?");
    let row = sqlx::query_as::<_, FillupRow>(&query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    debug!(fillup_id = id, vehicle_id, "Fill-up updated");
    Ok(Json(Fillup::from(row)))
}

/// Delete a fill-up by ID.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle or fill-up does not exist.
pub async fn delete(
    State(pool): State<SqlitePool>,
    Path((vehicle_id, id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    let result = sqlx::query("DELETE FROM fillups WHERE id = ? AND vehicle_id = ?")
        .bind(id)
        .bind(vehicle_id)
        .execute(&pool)
        .await
        .map_err(db_error)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("FILLUP_NOT_FOUND"));
    }

    info!(fillup_id = id, vehicle_id, "Fill-up deleted");
    Ok(StatusCode::NO_CONTENT)
}
