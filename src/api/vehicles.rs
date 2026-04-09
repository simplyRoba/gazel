use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, info};

use super::error::{ApiError, JsonBody, db_error, deserialize_nullable};

// ── Response type ────────────────────────────────────────

/// Vehicle as returned by the API.
#[derive(Serialize)]
pub struct Vehicle {
    pub id: i64,
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i64>,
    pub fuel_type: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── Database row type ────────────────────────────────────

/// Vehicle row as stored in `SQLite`.
#[derive(sqlx::FromRow)]
struct VehicleRow {
    id: i64,
    name: String,
    make: Option<String>,
    model: Option<String>,
    year: Option<i64>,
    fuel_type: String,
    notes: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<VehicleRow> for Vehicle {
    fn from(row: VehicleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            make: row.make,
            model: row.model,
            year: row.year,
            fuel_type: row.fuel_type,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// ── Shared SQL ───────────────────────────────────────────

const VEHICLE_SELECT: &str = "SELECT id, name, make, model, year, fuel_type, \
    notes, created_at, updated_at FROM vehicles";

// ── Validation ───────────────────────────────────────────

const VALID_FUEL_TYPES: &[&str] = &[
    "gasoline", "diesel", "electric", "hybrid", "lpg", "cng", "other",
];

/// # Errors
///
/// Returns `ApiError::Validation` if the name is empty or whitespace-only.
fn validate_vehicle_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(ApiError::Validation("VEHICLE_NAME_REQUIRED"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the fuel type is not in the allowed set.
fn validate_fuel_type(fuel_type: &str) -> Result<(), ApiError> {
    if !VALID_FUEL_TYPES.contains(&fuel_type) {
        return Err(ApiError::Validation("VEHICLE_INVALID_FUEL_TYPE"));
    }
    Ok(())
}

/// # Errors
///
/// Returns `ApiError::Validation` if the year is outside 1900-2100.
fn validate_year(year: Option<i64>) -> Result<(), ApiError> {
    if let Some(y) = year
        && !(1900..=2100).contains(&y)
    {
        return Err(ApiError::Validation("VEHICLE_INVALID_YEAR"));
    }
    Ok(())
}

// ── Request types ────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateVehicle {
    pub name: Option<String>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i64>,
    pub fuel_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateVehicle {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i64>,
    pub fuel_type: String,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct PatchVehicle {
    pub name: Option<String>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub make: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub model: Option<Option<String>>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub year: Option<Option<i64>>,
    pub fuel_type: Option<String>,
    #[allow(clippy::option_option)]
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub notes: Option<Option<String>>,
}

// ── Handlers ─────────────────────────────────────────────

/// List all vehicles, ordered by name.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failure.
pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Vehicle>>, ApiError> {
    let query = format!("{VEHICLE_SELECT} ORDER BY name");
    let rows = sqlx::query_as::<_, VehicleRow>(&query)
        .fetch_all(&pool)
        .await
        .map_err(db_error)?;
    Ok(Json(rows.into_iter().map(Vehicle::from).collect()))
}

/// Get a single vehicle by ID.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist.
pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Vehicle>, ApiError> {
    let query = format!("{VEHICLE_SELECT} WHERE id = ?");
    let row = sqlx::query_as::<_, VehicleRow>(&query)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?
        .ok_or(ApiError::NotFound("VEHICLE_NOT_FOUND"))?;
    Ok(Json(Vehicle::from(row)))
}

/// Create a new vehicle.
///
/// # Errors
///
/// Returns `ApiError::Validation` if name is missing/empty, fuel type is
/// invalid, or year is out of range.
pub async fn create(
    State(pool): State<SqlitePool>,
    JsonBody(body): JsonBody<CreateVehicle>,
) -> Result<(StatusCode, Json<Vehicle>), ApiError> {
    let name = body
        .name
        .ok_or(ApiError::Validation("VEHICLE_NAME_REQUIRED"))?;
    validate_vehicle_name(&name)?;
    let name = name.trim().to_string();

    let fuel_type = body
        .fuel_type
        .filter(|f| !f.trim().is_empty())
        .unwrap_or_else(|| "gasoline".to_string());
    validate_fuel_type(&fuel_type)?;
    validate_year(body.year)?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO vehicles (name, make, model, year, fuel_type, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&name)
    .bind(&body.make)
    .bind(&body.model)
    .bind(body.year)
    .bind(&fuel_type)
    .bind(&body.notes)
    .bind(&now)
    .bind(&now)
    .fetch_one(&pool)
    .await
    .map_err(db_error)?;

    let query = format!("{VEHICLE_SELECT} WHERE id = ?");
    let row = sqlx::query_as::<_, VehicleRow>(&query)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    let vehicle = Vehicle::from(row);
    info!(vehicle_id = id, name = %vehicle.name, "Vehicle created");

    Ok((StatusCode::CREATED, Json(vehicle)))
}

/// Full update (PUT) of a vehicle.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::Validation` if fields are invalid.
pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    JsonBody(body): JsonBody<UpdateVehicle>,
) -> Result<Json<Vehicle>, ApiError> {
    // Check exists
    let query = format!("{VEHICLE_SELECT} WHERE id = ?");
    sqlx::query_as::<_, VehicleRow>(&query)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?
        .ok_or(ApiError::NotFound("VEHICLE_NOT_FOUND"))?;

    validate_vehicle_name(&body.name)?;
    let name = body.name.trim().to_string();
    validate_fuel_type(&body.fuel_type)?;
    validate_year(body.year)?;

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query(
        "UPDATE vehicles SET name = ?, make = ?, model = ?, year = ?, \
         fuel_type = ?, notes = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&body.make)
    .bind(&body.model)
    .bind(body.year)
    .bind(&body.fuel_type)
    .bind(&body.notes)
    .bind(&now)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(db_error)?;

    let row = sqlx::query_as::<_, VehicleRow>(&format!("{VEHICLE_SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    debug!(vehicle_id = id, "Vehicle updated");
    Ok(Json(Vehicle::from(row)))
}

/// Partial update (PATCH) of a vehicle.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::Validation` if merged fields are invalid.
pub async fn patch(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    JsonBody(body): JsonBody<PatchVehicle>,
) -> Result<Json<Vehicle>, ApiError> {
    let current = sqlx::query_as::<_, VehicleRow>(&format!("{VEHICLE_SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?
        .ok_or(ApiError::NotFound("VEHICLE_NOT_FOUND"))?;

    // Merge: outer None = keep current, Some(None) = clear, Some(Some(v)) = new
    let name = body.name.unwrap_or(current.name);
    validate_vehicle_name(&name)?;
    let name = name.trim().to_string();
    let make = body.make.unwrap_or(current.make);
    let model = body.model.unwrap_or(current.model);
    let year = body.year.unwrap_or(current.year);
    validate_year(year)?;
    let fuel_type = body.fuel_type.unwrap_or(current.fuel_type);
    validate_fuel_type(&fuel_type)?;
    let notes = body.notes.unwrap_or(current.notes);

    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query(
        "UPDATE vehicles SET name = ?, make = ?, model = ?, year = ?, \
         fuel_type = ?, notes = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&name)
    .bind(&make)
    .bind(&model)
    .bind(year)
    .bind(&fuel_type)
    .bind(&notes)
    .bind(&now)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(db_error)?;

    let row = sqlx::query_as::<_, VehicleRow>(&format!("{VEHICLE_SELECT} WHERE id = ?"))
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    debug!(vehicle_id = id, "Vehicle patched");
    Ok(Json(Vehicle::from(row)))
}

/// Delete a vehicle by ID.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::Conflict` if the vehicle has existing fill-ups.
pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    // Check vehicle exists first
    let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM vehicles WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(db_error)?;

    if exists.is_none() {
        return Err(ApiError::NotFound("VEHICLE_NOT_FOUND"));
    }

    // Check for existing fill-ups
    let has_fillups: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM fillups WHERE vehicle_id = ?)")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(db_error)?;

    if has_fillups {
        return Err(ApiError::Conflict("VEHICLE_HAS_FILLUPS"));
    }

    sqlx::query("DELETE FROM vehicles WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(db_error)?;

    info!(vehicle_id = id, "Vehicle deleted");
    Ok(StatusCode::NO_CONTENT)
}
