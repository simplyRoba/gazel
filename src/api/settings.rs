use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::error::{ApiError, JsonBody, db_error};

// ── Response type ────────────────────────────────────────

/// Settings as returned by the API.
#[derive(Serialize)]
pub struct Settings {
    pub unit_system: String,
    pub distance_unit: String,
    pub volume_unit: String,
    pub currency: String,
    pub color_mode: String,
    pub locale: String,
}

// ── Database row type ────────────────────────────────────

/// Settings row as stored in `SQLite`.
#[derive(sqlx::FromRow)]
struct SettingsRow {
    unit_system: String,
    distance_unit: String,
    volume_unit: String,
    currency: String,
    color_mode: String,
    locale: String,
}

impl From<SettingsRow> for Settings {
    fn from(row: SettingsRow) -> Self {
        Self {
            unit_system: row.unit_system,
            distance_unit: row.distance_unit,
            volume_unit: row.volume_unit,
            currency: row.currency,
            color_mode: row.color_mode,
            locale: row.locale,
        }
    }
}

// ── Shared SQL ───────────────────────────────────────────

const SETTINGS_SELECT: &str = "SELECT unit_system, distance_unit, volume_unit, currency, color_mode, locale FROM settings WHERE id = 1";

// ── Validation ───────────────────────────────────────────

const VALID_UNIT_SYSTEMS: &[&str] = &["metric", "imperial", "custom"];
const VALID_DISTANCE_UNITS: &[&str] = &["km", "mi"];
const VALID_VOLUME_UNITS: &[&str] = &["l", "gal"];
const VALID_COLOR_MODES: &[&str] = &["light", "dark", "system"];
const VALID_CURRENCIES: &[&str] = &["USD", "EUR"];
const VALID_LOCALES: &[&str] = &["en"];

/// Validate a single optional field against allowed values.
///
/// # Errors
///
/// Returns `ApiError::Validation` with the given code if the value is not in the allowed set.
fn validate_field(
    value: Option<&String>,
    allowed: &[&str],
    error_code: &'static str,
) -> Result<(), ApiError> {
    if let Some(v) = value
        && !allowed.contains(&v.as_str())
    {
        return Err(ApiError::Validation(error_code));
    }
    Ok(())
}

// ── Request type ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateSettings {
    pub unit_system: Option<String>,
    pub distance_unit: Option<String>,
    pub volume_unit: Option<String>,
    pub currency: Option<String>,
    pub color_mode: Option<String>,
    pub locale: Option<String>,
}

// ── Handlers ─────────────────────────────────────────────

/// Read current settings.
///
/// # Errors
///
/// Returns `ApiError::InternalError` on database failures.
pub async fn get_settings(State(pool): State<SqlitePool>) -> Result<Json<Settings>, ApiError> {
    let row = sqlx::query_as::<_, SettingsRow>(SETTINGS_SELECT)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    Ok(Json(Settings::from(row)))
}

/// Update settings with partial body. Omitted fields keep their current value.
///
/// # Errors
///
/// Returns `ApiError::Validation` for invalid field values, or
/// `ApiError::InternalError` on database failures.
pub async fn update_settings(
    State(pool): State<SqlitePool>,
    JsonBody(body): JsonBody<UpdateSettings>,
) -> Result<Json<Settings>, ApiError> {
    validate_field(
        body.unit_system.as_ref(),
        VALID_UNIT_SYSTEMS,
        "SETTINGS_INVALID_UNIT_SYSTEM",
    )?;
    validate_field(
        body.distance_unit.as_ref(),
        VALID_DISTANCE_UNITS,
        "SETTINGS_INVALID_DISTANCE_UNIT",
    )?;
    validate_field(
        body.volume_unit.as_ref(),
        VALID_VOLUME_UNITS,
        "SETTINGS_INVALID_VOLUME_UNIT",
    )?;
    validate_field(
        body.currency.as_ref(),
        VALID_CURRENCIES,
        "SETTINGS_INVALID_CURRENCY",
    )?;
    validate_field(
        body.color_mode.as_ref(),
        VALID_COLOR_MODES,
        "SETTINGS_INVALID_COLOR_MODE",
    )?;
    validate_field(
        body.locale.as_ref(),
        VALID_LOCALES,
        "SETTINGS_INVALID_LOCALE",
    )?;

    sqlx::query(
        "UPDATE settings SET \
         unit_system = COALESCE(?, unit_system), \
         distance_unit = COALESCE(?, distance_unit), \
         volume_unit = COALESCE(?, volume_unit), \
         currency = COALESCE(?, currency), \
         color_mode = COALESCE(?, color_mode), \
         locale = COALESCE(?, locale) \
         WHERE id = 1",
    )
    .bind(&body.unit_system)
    .bind(&body.distance_unit)
    .bind(&body.volume_unit)
    .bind(&body.currency)
    .bind(&body.color_mode)
    .bind(&body.locale)
    .execute(&pool)
    .await
    .map_err(db_error)?;

    let row = sqlx::query_as::<_, SettingsRow>(SETTINGS_SELECT)
        .fetch_one(&pool)
        .await
        .map_err(db_error)?;

    Ok(Json(Settings::from(row)))
}
