use axum::Json;
use axum::extract::{Path, RawQuery, State};
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
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
    pub odometer: f64,
    pub fuel_amount: f64,
    pub fuel_unit: String,
    pub cost: f64,
    pub currency: String,
    pub is_full_tank: bool,
    pub is_missed: bool,
    pub station: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A cursor-paginated page of fill-ups.
#[derive(Serialize)]
pub struct FillupPage {
    pub items: Vec<Fillup>,
    pub next_cursor: Option<String>,
}

// ── Database row type ────────────────────────────────────

/// Fill-up row as stored in `SQLite`.
///
/// The `odometer`, `cost`, and `currency` columns are `NOT NULL` at the
/// application level (enforced by validation), but the database schema still
/// allows `NULL` for backwards compatibility with rows created before these
/// fields became required. The `From` impl maps `NULL` to sensible defaults
/// so the API response type always has non-optional values.
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
            odometer: row.odometer.unwrap_or(0.0),
            fuel_amount: row.fuel_amount,
            fuel_unit: row.fuel_unit,
            cost: row.cost.unwrap_or(0.0),
            currency: row.currency.unwrap_or_default(),
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
    pub cost: Option<f64>,
    pub is_full_tank: Option<bool>,
    pub is_missed: Option<bool>,
    pub station: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateFillup {
    pub date: String,
    pub odometer: f64,
    pub fuel_amount: f64,
    pub cost: f64,
    pub is_full_tank: Option<bool>,
    pub is_missed: Option<bool>,
    pub station: Option<String>,
    pub notes: Option<String>,
}

const DEFAULT_PAGE_LIMIT: u16 = 25;
const MAX_PAGE_LIMIT: u16 = 100;

/// Decoded position of the final item in a page.
#[derive(Deserialize, Serialize)]
struct FillupCursor {
    date: String,
    id: i64,
}

/// Validated pagination parameters for fill-up listing.
struct PageQuery {
    limit: u16,
    cursor: Option<FillupCursor>,
}

impl PageQuery {
    fn parse(raw_query: Option<&str>) -> Result<Self, ApiError> {
        let mut limit = None;
        let mut cursor = None;

        for (key, value) in url::form_urlencoded::parse(raw_query.unwrap_or_default().as_bytes()) {
            match key.as_ref() {
                "limit" => {
                    if limit.is_some() {
                        return Err(ApiError::BadRequest("FILLUP_INVALID_PAGE_LIMIT"));
                    }

                    let parsed_limit = value
                        .parse::<u16>()
                        .map_err(|_| ApiError::BadRequest("FILLUP_INVALID_PAGE_LIMIT"))?;
                    if !(1..=MAX_PAGE_LIMIT).contains(&parsed_limit) {
                        return Err(ApiError::BadRequest("FILLUP_INVALID_PAGE_LIMIT"));
                    }
                    limit = Some(parsed_limit);
                }
                "cursor" => {
                    if cursor.is_some() {
                        return Err(ApiError::BadRequest("FILLUP_INVALID_CURSOR"));
                    }
                    cursor = Some(decode_cursor(&value)?);
                }
                _ => {}
            }
        }

        Ok(Self {
            limit: limit.unwrap_or(DEFAULT_PAGE_LIMIT),
            cursor,
        })
    }
}

fn decode_cursor(value: &str) -> Result<FillupCursor, ApiError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ApiError::BadRequest("FILLUP_INVALID_CURSOR"))?;
    let cursor = serde_json::from_slice::<FillupCursor>(&bytes)
        .map_err(|_| ApiError::BadRequest("FILLUP_INVALID_CURSOR"))?;

    if cursor.date.trim().is_empty() || cursor.id <= 0 {
        return Err(ApiError::BadRequest("FILLUP_INVALID_CURSOR"));
    }

    Ok(cursor)
}

fn encode_cursor(cursor: &FillupCursor) -> Result<String, ApiError> {
    let bytes = serde_json::to_vec(cursor).map_err(|error| {
        tracing::error!(%error, "Failed to serialize fill-up cursor");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
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
/// Validate that the odometer reading is consistent with neighboring
/// fill-ups.
///
/// For **creates** (`exclude_id = None`): the new odometer must be ≥ the
/// current maximum (new fill-ups go at the top).
///
/// For **updates** (`exclude_id = Some`): the odometer must fit between
/// its chronological neighbors — ≥ the previous fill-up and ≤ the next
/// fill-up (by date, then by id as tie-breaker).
async fn validate_odometer(
    pool: &SqlitePool,
    vehicle_id: i64,
    odometer: f64,
    exclude_id: Option<i64>,
) -> Result<(), ApiError> {
    if let Some(eid) = exclude_id {
        // Find the date of the fill-up being edited.
        let date: String =
            sqlx::query_scalar("SELECT date FROM fillups WHERE id = ? AND vehicle_id = ?")
                .bind(eid)
                .bind(vehicle_id)
                .fetch_one(pool)
                .await
                .map_err(db_error)?;

        // Previous fill-up: the one right before this one chronologically.
        let prev: Option<f64> = sqlx::query_scalar(
            "SELECT odometer FROM fillups \
             WHERE vehicle_id = ? AND id != ? AND (date < ? OR (date = ? AND id < ?)) \
             ORDER BY date DESC, id DESC LIMIT 1",
        )
        .bind(vehicle_id)
        .bind(eid)
        .bind(&date)
        .bind(&date)
        .bind(eid)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

        if let Some(p) = prev
            && odometer < p
        {
            return Err(ApiError::Validation("FILLUP_INVALID_ODOMETER"));
        }

        // Next fill-up: the one right after this one chronologically.
        let next: Option<f64> = sqlx::query_scalar(
            "SELECT odometer FROM fillups \
             WHERE vehicle_id = ? AND id != ? AND (date > ? OR (date = ? AND id > ?)) \
             ORDER BY date ASC, id ASC LIMIT 1",
        )
        .bind(vehicle_id)
        .bind(eid)
        .bind(&date)
        .bind(&date)
        .bind(eid)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

        if let Some(n) = next
            && odometer > n
        {
            return Err(ApiError::Validation("FILLUP_INVALID_ODOMETER"));
        }
    } else {
        // Create: new fill-up must have the highest odometer.
        let max_odometer: Option<f64> =
            sqlx::query_scalar("SELECT MAX(odometer) FROM fillups WHERE vehicle_id = ?")
                .bind(vehicle_id)
                .fetch_one(pool)
                .await
                .map_err(db_error)?;

        if let Some(max) = max_odometer
            && odometer < max
        {
            return Err(ApiError::Validation("FILLUP_INVALID_ODOMETER"));
        }
    }

    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the cost is negative.
fn validate_cost(cost: f64) -> Result<(), ApiError> {
    if cost < 0.0 {
        return Err(ApiError::Validation("FILLUP_INVALID_COST"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist.
pub(crate) async fn ensure_vehicle_exists(
    pool: &SqlitePool,
    vehicle_id: i64,
) -> Result<(), ApiError> {
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

/// Read `volume_unit` and `currency` from the settings table.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failures.
async fn read_settings(pool: &SqlitePool) -> Result<(String, String), ApiError> {
    let row: (String, String) =
        sqlx::query_as("SELECT volume_unit, currency FROM settings WHERE id = 1")
            .fetch_one(pool)
            .await
            .map_err(db_error)?;
    Ok(row)
}

// ── Handlers ─────────────────────────────────────────────

/// List a cursor-paginated fill-up history for a vehicle, sorted by date and ID
/// descending.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::BadRequest` if pagination parameters are invalid.
pub async fn list(
    State(pool): State<SqlitePool>,
    Path(vehicle_id): Path<i64>,
    RawQuery(raw_query): RawQuery,
) -> Result<Json<FillupPage>, ApiError> {
    let page_query = PageQuery::parse(raw_query.as_deref())?;
    ensure_vehicle_exists(&pool, vehicle_id).await?;

    let mut rows = if let Some(cursor) = page_query.cursor {
        let query = format!(
            "{FILLUP_SELECT} WHERE vehicle_id = ? \
             AND (date < ? OR (date = ? AND id < ?)) \
             ORDER BY date DESC, id DESC LIMIT ?"
        );
        sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(query.as_str()))
            .bind(vehicle_id)
            .bind(&cursor.date)
            .bind(&cursor.date)
            .bind(cursor.id)
            .bind(i64::from(page_query.limit) + 1)
            .fetch_all(&pool)
            .await
            .map_err(db_error)?
    } else {
        let query =
            format!("{FILLUP_SELECT} WHERE vehicle_id = ? ORDER BY date DESC, id DESC LIMIT ?");
        sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(query.as_str()))
            .bind(vehicle_id)
            .bind(i64::from(page_query.limit) + 1)
            .fetch_all(&pool)
            .await
            .map_err(db_error)?
    };

    let next_cursor = if rows.len() > usize::from(page_query.limit) {
        rows.pop();
        rows.last()
            .map(|row| {
                encode_cursor(&FillupCursor {
                    date: row.date.clone(),
                    id: row.id,
                })
            })
            .transpose()?
    } else {
        None
    };

    Ok(Json(FillupPage {
        items: rows.into_iter().map(Fillup::from).collect(),
        next_cursor,
    }))
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
    let row = sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(query.as_str()))
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

    let odometer = body
        .odometer
        .ok_or(ApiError::Validation("FILLUP_ODOMETER_REQUIRED"))?;
    validate_odometer(&pool, vehicle_id, odometer, None).await?;

    let cost = body
        .cost
        .ok_or(ApiError::Validation("FILLUP_COST_REQUIRED"))?;
    validate_cost(cost)?;

    let (fuel_unit, currency) = read_settings(&pool).await?;
    let is_full_tank = i32::from(body.is_full_tank.unwrap_or(true));
    let is_missed = i32::from(body.is_missed.unwrap_or(false));

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO fillups (vehicle_id, date, odometer, fuel_amount, fuel_unit, \
         cost, currency, is_full_tank, is_missed, station, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(vehicle_id)
    .bind(&date)
    .bind(odometer)
    .bind(fuel_amount)
    .bind(&fuel_unit)
    .bind(cost)
    .bind(&currency)
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
    let row = sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(query.as_str()))
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
    sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(exists_query.as_str()))
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

    let (fuel_unit, currency) = read_settings(&pool).await?;
    let is_full_tank = i32::from(body.is_full_tank.unwrap_or(true));
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
    .bind(&currency)
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
    let row = sqlx::query_as::<_, FillupRow>(sqlx::AssertSqlSafe(query.as_str()))
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
