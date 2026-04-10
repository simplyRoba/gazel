use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use super::error::{ApiError, db_error};
use super::vehicles::Vehicle;

// ── Export document types ────────────────────────────────

/// Top-level export document containing version metadata and all data.
#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: String,
    pub vehicles: Vec<ExportVehicle>,
}

/// A vehicle with its embedded fill-ups for export.
#[derive(Serialize, Deserialize)]
pub struct ExportVehicle {
    pub name: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i64>,
    pub fuel_type: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub fillups: Vec<ExportFillup>,
}

/// A fill-up record for export (no internal IDs).
#[derive(Serialize, Deserialize)]
pub struct ExportFillup {
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

// ── Database row types ──────────────────────────────────

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

#[derive(sqlx::FromRow)]
struct FillupRow {
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

// ── Helpers ─────────────────────────────────────────────

/// Build an `ExportData` document from all vehicles and their fill-ups.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failure.
pub async fn build_export_all(pool: &SqlitePool) -> Result<ExportData, ApiError> {
    let vehicles = sqlx::query_as::<_, VehicleRow>(
        "SELECT id, name, make, model, year, fuel_type, notes, created_at, updated_at \
         FROM vehicles ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    let mut export_vehicles = Vec::with_capacity(vehicles.len());
    for v in vehicles {
        let fillups = fetch_fillups_for_vehicle(pool, v.id).await?;
        export_vehicles.push(ExportVehicle {
            name: v.name,
            make: v.make,
            model: v.model,
            year: v.year,
            fuel_type: v.fuel_type,
            notes: v.notes,
            created_at: v.created_at,
            updated_at: v.updated_at,
            fillups,
        });
    }

    Ok(ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        vehicles: export_vehicles,
    })
}

/// Build an `ExportData` document for a single vehicle.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::InternalError` on database failure.
pub async fn build_export_vehicle(
    pool: &SqlitePool,
    vehicle_id: i64,
) -> Result<(ExportData, Vehicle), ApiError> {
    let v = sqlx::query_as::<_, VehicleRow>(
        "SELECT id, name, make, model, year, fuel_type, notes, created_at, updated_at \
         FROM vehicles WHERE id = ?",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?
    .ok_or(ApiError::NotFound("VEHICLE_NOT_FOUND"))?;

    let fillups = fetch_fillups_for_vehicle(pool, v.id).await?;

    let vehicle_response = Vehicle {
        id: v.id,
        name: v.name.clone(),
        make: v.make.clone(),
        model: v.model.clone(),
        year: v.year,
        fuel_type: v.fuel_type.clone(),
        notes: v.notes.clone(),
        created_at: v.created_at.clone(),
        updated_at: v.updated_at.clone(),
    };

    let export = ExportData {
        version: env!("CARGO_PKG_VERSION").to_string(),
        exported_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        vehicles: vec![ExportVehicle {
            name: v.name,
            make: v.make,
            model: v.model,
            year: v.year,
            fuel_type: v.fuel_type,
            notes: v.notes,
            created_at: v.created_at,
            updated_at: v.updated_at,
            fillups,
        }],
    };

    Ok((export, vehicle_response))
}

/// Fetch all fill-ups for a vehicle, sorted by date ascending for export.
async fn fetch_fillups_for_vehicle(
    pool: &SqlitePool,
    vehicle_id: i64,
) -> Result<Vec<ExportFillup>, ApiError> {
    let rows = sqlx::query_as::<_, FillupRow>(
        "SELECT date, odometer, fuel_amount, fuel_unit, cost, currency, \
         is_full_tank, is_missed, station, notes, created_at, updated_at \
         FROM fillups WHERE vehicle_id = ? ORDER BY date ASC, id ASC",
    )
    .bind(vehicle_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    Ok(rows
        .into_iter()
        .map(|r| ExportFillup {
            date: r.date,
            odometer: r.odometer.unwrap_or(0.0),
            fuel_amount: r.fuel_amount,
            fuel_unit: r.fuel_unit,
            cost: r.cost.unwrap_or(0.0),
            currency: r.currency.unwrap_or_default(),
            is_full_tank: r.is_full_tank != 0,
            is_missed: r.is_missed != 0,
            station: r.station,
            notes: r.notes,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// Convert a string to kebab-case for filenames.
fn to_kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

// ── Handlers ────────────────────────────────────────────

/// Export all vehicles and fill-ups as a JSON document.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failure.
pub async fn export_all(State(pool): State<SqlitePool>) -> Result<Response, ApiError> {
    let data = build_export_all(&pool).await?;
    info!(vehicles = data.vehicles.len(), "Full data export requested");

    let json = serde_json::to_string_pretty(&data)
        .map_err(|_| ApiError::InternalError("INTERNAL_ERROR"))?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"gazel-export.json\"",
            ),
        ],
        json,
    )
        .into_response())
}

/// Export a single vehicle and its fill-ups as a JSON document.
///
/// # Errors
///
/// Returns `ApiError::NotFound` if the vehicle does not exist, or
/// `ApiError::InternalError` on database failure.
pub async fn export_vehicle(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let (data, vehicle) = build_export_vehicle(&pool, id).await?;
    info!(vehicle_id = id, name = %vehicle.name, "Single vehicle export requested");

    let kebab_name = to_kebab_case(&vehicle.name);
    let filename = format!("gazel-export-{kebab_name}.json");
    let disposition = format!("attachment; filename=\"{filename}\"");

    let json = serde_json::to_string_pretty(&data)
        .map_err(|_| ApiError::InternalError("INTERNAL_ERROR"))?;

    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        json,
    )
        .into_response())
}
