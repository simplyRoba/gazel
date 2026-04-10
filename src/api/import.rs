use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use super::error::{ApiError, JsonBody};
use super::export::{ExportData, ExportFillup, ExportVehicle};

// ── Query parameters ────────────────────────────────────

#[derive(Deserialize)]
pub struct ImportParams {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub preview: Option<bool>,
}

// ── Response types ──────────────────────────────────────

/// Result of a successful replace import.
#[derive(Serialize)]
pub struct ReplaceResult {
    pub vehicles_created: usize,
    pub fillups_created: usize,
}

/// Result of a successful merge import.
#[derive(Serialize)]
pub struct MergeResult {
    pub vehicles_created: usize,
    pub vehicles_updated: usize,
    pub fillups_created: usize,
    pub fillups_skipped: usize,
}

/// Preview result for replace mode.
#[derive(Serialize)]
pub struct ReplacePreview {
    pub preview: bool,
    pub vehicles: usize,
    pub fillups: usize,
}

/// Preview result for merge mode.
#[derive(Serialize)]
pub struct MergePreview {
    pub preview: bool,
    pub vehicles_new: usize,
    pub vehicles_existing: usize,
    pub fillups_new: usize,
    pub fillups_existing: usize,
}

/// Unified import response (serde will flatten the correct variant).
#[derive(Serialize)]
#[serde(untagged)]
pub enum ImportResponse {
    ReplaceResult(ReplaceResult),
    MergeResult(MergeResult),
    ReplacePreview(ReplacePreview),
    MergePreview(MergePreview),
}

// ── Validation ──────────────────────────────────────────

/// Check that the export document's major.minor version matches the running
/// server.
///
/// # Errors
///
/// Returns `ApiError::Validation` if the version is missing or incompatible.
fn check_version(archive_version: &str) -> Result<(), ApiError> {
    let server_version = env!("CARGO_PKG_VERSION");
    let server_parts: Vec<&str> = server_version.split('.').collect();
    let archive_parts: Vec<&str> = archive_version.split('.').collect();

    if server_parts.len() < 2 || archive_parts.len() < 2 {
        return Err(ApiError::Validation("IMPORT_VERSION_MISMATCH"));
    }

    if server_parts[0] != archive_parts[0] || server_parts[1] != archive_parts[1] {
        return Err(ApiError::Validation("IMPORT_VERSION_MISMATCH"));
    }

    Ok(())
}

/// Validate all vehicles and fill-ups in the export document.
///
/// # Errors
///
/// Returns `ApiError::Validation` if any record fails validation.
fn validate_import(data: &ExportData) -> Result<(), ApiError> {
    for vehicle in &data.vehicles {
        validate_vehicle(vehicle)?;
        for fillup in &vehicle.fillups {
            validate_fillup(fillup)?;
        }
    }
    Ok(())
}

fn validate_vehicle(vehicle: &ExportVehicle) -> Result<(), ApiError> {
    if vehicle.name.trim().is_empty() {
        return Err(ApiError::Validation("IMPORT_VALIDATION_ERROR"));
    }
    Ok(())
}

fn validate_fillup(fillup: &ExportFillup) -> Result<(), ApiError> {
    if fillup.odometer < 0.0 {
        return Err(ApiError::Validation("IMPORT_VALIDATION_ERROR"));
    }
    if fillup.cost < 0.0 {
        return Err(ApiError::Validation("IMPORT_VALIDATION_ERROR"));
    }
    if fillup.fuel_amount < 0.0 {
        return Err(ApiError::Validation("IMPORT_VALIDATION_ERROR"));
    }
    if fillup.date.trim().is_empty() {
        return Err(ApiError::Validation("IMPORT_VALIDATION_ERROR"));
    }
    Ok(())
}

// ── Replace mode ────────────────────────────────────────

async fn execute_replace(pool: &SqlitePool, data: &ExportData) -> Result<ReplaceResult, ApiError> {
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    // Delete all fill-ups first (FK order), then vehicles
    sqlx::query("DELETE FROM fillups")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;

    sqlx::query("DELETE FROM vehicles")
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {e}");
            ApiError::InternalError("INTERNAL_ERROR")
        })?;

    let mut vehicles_created: usize = 0;
    let mut fillups_created: usize = 0;

    for vehicle in &data.vehicles {
        let vehicle_id = insert_vehicle(&mut tx, vehicle).await?;
        vehicles_created += 1;

        for fillup in &vehicle.fillups {
            insert_fillup(&mut tx, vehicle_id, fillup).await?;
            fillups_created += 1;
        }
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    Ok(ReplaceResult {
        vehicles_created,
        fillups_created,
    })
}

// ── Merge mode ──────────────────────────────────────────

async fn execute_merge(pool: &SqlitePool, data: &ExportData) -> Result<MergeResult, ApiError> {
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    let mut vehicles_created: usize = 0;
    let mut vehicles_updated: usize = 0;
    let mut fillups_created: usize = 0;
    let mut fillups_skipped: usize = 0;

    for vehicle in &data.vehicles {
        // Try to match by name (case-insensitive)
        let existing_id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM vehicles WHERE LOWER(name) = LOWER(?)")
                .bind(&vehicle.name)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| {
                    tracing::error!("Database error: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?;

        let vehicle_id = if let Some(id) = existing_id {
            // Update existing vehicle fields
            update_vehicle(&mut tx, id, vehicle).await?;
            vehicles_updated += 1;
            id
        } else {
            let id = insert_vehicle(&mut tx, vehicle).await?;
            vehicles_created += 1;
            id
        };

        for fillup in &vehicle.fillups {
            // Match by (vehicle_id, date, odometer)
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM fillups \
                 WHERE vehicle_id = ? AND date = ? AND odometer = ?)",
            )
            .bind(vehicle_id)
            .bind(&fillup.date)
            .bind(fillup.odometer)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {e}");
                ApiError::InternalError("INTERNAL_ERROR")
            })?;

            if exists {
                fillups_skipped += 1;
            } else {
                insert_fillup(&mut tx, vehicle_id, fillup).await?;
                fillups_created += 1;
            }
        }
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    Ok(MergeResult {
        vehicles_created,
        vehicles_updated,
        fillups_created,
        fillups_skipped,
    })
}

/// Preview merge without modifying data.
async fn preview_merge(pool: &SqlitePool, data: &ExportData) -> Result<MergePreview, ApiError> {
    let mut vehicles_new: usize = 0;
    let mut vehicles_existing: usize = 0;
    let mut fillups_new: usize = 0;
    let mut fillups_existing: usize = 0;

    for vehicle in &data.vehicles {
        let existing_id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM vehicles WHERE LOWER(name) = LOWER(?)")
                .bind(&vehicle.name)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?;

        if let Some(id) = existing_id {
            vehicles_existing += 1;

            for fillup in &vehicle.fillups {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM fillups \
                     WHERE vehicle_id = ? AND date = ? AND odometer = ?)",
                )
                .bind(id)
                .bind(&fillup.date)
                .bind(fillup.odometer)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error: {e}");
                    ApiError::InternalError("INTERNAL_ERROR")
                })?;

                if exists {
                    fillups_existing += 1;
                } else {
                    fillups_new += 1;
                }
            }
        } else {
            vehicles_new += 1;
            fillups_new += vehicle.fillups.len();
        }
    }

    Ok(MergePreview {
        preview: true,
        vehicles_new,
        vehicles_existing,
        fillups_new,
        fillups_existing,
    })
}

// ── SQL helpers ─────────────────────────────────────────

async fn insert_vehicle(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    vehicle: &ExportVehicle,
) -> Result<i64, ApiError> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query_scalar::<_, i64>(
        "INSERT INTO vehicles (name, make, model, year, fuel_type, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(&vehicle.name)
    .bind(&vehicle.make)
    .bind(&vehicle.model)
    .bind(vehicle.year)
    .bind(&vehicle.fuel_type)
    .bind(&vehicle.notes)
    .bind(&vehicle.created_at)
    .bind(&now)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })
}

async fn update_vehicle(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: i64,
    vehicle: &ExportVehicle,
) -> Result<(), ApiError> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query(
        "UPDATE vehicles SET name = ?, make = ?, model = ?, year = ?, \
         fuel_type = ?, notes = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&vehicle.name)
    .bind(&vehicle.make)
    .bind(&vehicle.model)
    .bind(vehicle.year)
    .bind(&vehicle.fuel_type)
    .bind(&vehicle.notes)
    .bind(&now)
    .bind(id)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    Ok(())
}

async fn insert_fillup(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    vehicle_id: i64,
    fillup: &ExportFillup,
) -> Result<(), ApiError> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    sqlx::query(
        "INSERT INTO fillups (vehicle_id, date, odometer, fuel_amount, fuel_unit, \
         cost, currency, is_full_tank, is_missed, station, notes, created_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(vehicle_id)
    .bind(&fillup.date)
    .bind(fillup.odometer)
    .bind(fillup.fuel_amount)
    .bind(&fillup.fuel_unit)
    .bind(fillup.cost)
    .bind(&fillup.currency)
    .bind(i32::from(fillup.is_full_tank))
    .bind(i32::from(fillup.is_missed))
    .bind(&fillup.station)
    .bind(&fillup.notes)
    .bind(&fillup.created_at)
    .bind(&now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {e}");
        ApiError::InternalError("INTERNAL_ERROR")
    })?;

    Ok(())
}

// ── Handler ─────────────────────────────────────────────

/// Import vehicles and fill-ups from an export JSON document.
///
/// Query parameters:
/// - `mode`: `replace` (default) or `merge`
/// - `preview`: `true` to validate and return counts without writing
///
/// # Errors
///
/// Returns validation errors for invalid data, version mismatches, or
/// database errors.
pub async fn import_data(
    State(pool): State<SqlitePool>,
    axum::extract::Query(params): axum::extract::Query<ImportParams>,
    JsonBody(data): JsonBody<ExportData>,
) -> Result<Json<ImportResponse>, ApiError> {
    let mode = params.mode.as_deref().unwrap_or("replace");
    let preview = params.preview.unwrap_or(false);

    // Validate mode
    if mode != "replace" && mode != "merge" {
        return Err(ApiError::Validation("IMPORT_INVALID_MODE"));
    }

    // Validate version
    check_version(&data.version)?;

    // Validate data
    validate_import(&data)?;

    if preview {
        return if mode == "merge" {
            let result = preview_merge(&pool, &data).await?;
            Ok(Json(ImportResponse::MergePreview(result)))
        } else {
            let total_fillups: usize = data.vehicles.iter().map(|v| v.fillups.len()).sum();
            Ok(Json(ImportResponse::ReplacePreview(ReplacePreview {
                preview: true,
                vehicles: data.vehicles.len(),
                fillups: total_fillups,
            })))
        };
    }

    if mode == "merge" {
        let result = execute_merge(&pool, &data).await?;
        info!(
            vehicles_created = result.vehicles_created,
            vehicles_updated = result.vehicles_updated,
            fillups_created = result.fillups_created,
            fillups_skipped = result.fillups_skipped,
            "Data merged"
        );
        Ok(Json(ImportResponse::MergeResult(result)))
    } else {
        let result = execute_replace(&pool, &data).await?;
        info!(
            vehicles_created = result.vehicles_created,
            fillups_created = result.fillups_created,
            "Data imported (replace)"
        );
        Ok(Json(ImportResponse::ReplaceResult(result)))
    }
}
