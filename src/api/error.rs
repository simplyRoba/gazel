use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;

/// API error type that maps to a consistent JSON response.
///
/// Each variant carries a `&'static str` error code. The `IntoResponse`
/// implementation maps each variant to the appropriate HTTP status and returns
/// `{ "code": "<CODE>", "message": "<human-readable>" }`.
pub enum ApiError {
    Unauthorized(&'static str),
    NotFound(&'static str),
    Validation(&'static str),
    Conflict(&'static str),
    BadRequest(&'static str),
    InternalError(&'static str),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Unauthorized(code) => (StatusCode::UNAUTHORIZED, code),
            Self::NotFound(code) => (StatusCode::NOT_FOUND, code),
            Self::Validation(code) => (StatusCode::UNPROCESSABLE_ENTITY, code),
            Self::Conflict(code) => (StatusCode::CONFLICT, code),
            Self::BadRequest(code) => (StatusCode::BAD_REQUEST, code),
            Self::InternalError(code) => (StatusCode::INTERNAL_SERVER_ERROR, code),
        };

        let message = default_message(code);
        (status, Json(json!({ "code": code, "message": message }))).into_response()
    }
}

const ERROR_MESSAGES: &[(&str, &str)] = &[
    ("AUTHENTICATION_REQUIRED", "Authentication is required."),
    ("INTERNAL_ERROR", "An unexpected error occurred."),
    (
        "INVALID_REQUEST_BODY",
        "The request body is missing or malformed.",
    ),
    ("VEHICLE_NOT_FOUND", "Vehicle not found."),
    ("VEHICLE_NAME_REQUIRED", "Vehicle name is required."),
    ("VEHICLE_INVALID_FUEL_TYPE", "Invalid fuel type."),
    ("VEHICLE_INVALID_YEAR", "Invalid year."),
    (
        "VEHICLE_HAS_FILLUPS",
        "Cannot delete vehicle with existing fill-ups.",
    ),
    ("FILLUP_NOT_FOUND", "Fill-up not found."),
    ("FILLUP_DATE_REQUIRED", "Fill-up date is required."),
    ("FILLUP_FUEL_AMOUNT_REQUIRED", "Fuel amount is required."),
    (
        "FILLUP_INVALID_FUEL_AMOUNT",
        "Fuel amount must be greater than zero.",
    ),
    ("FILLUP_ODOMETER_REQUIRED", "Odometer reading is required."),
    (
        "FILLUP_INVALID_ODOMETER",
        "Odometer reading must not be less than the previous reading.",
    ),
    ("FILLUP_COST_REQUIRED", "Cost is required."),
    ("FILLUP_INVALID_COST", "Cost must not be negative."),
    (
        "FILLUP_INVALID_PAGE_LIMIT",
        "Page limit must be an integer between 1 and 100.",
    ),
    ("FILLUP_INVALID_CURSOR", "Fill-up cursor is invalid."),
    (
        "SETTINGS_INVALID_COLOR_MODE",
        "Invalid color mode. Must be light, dark, or system.",
    ),
    (
        "SETTINGS_INVALID_UNIT_SYSTEM",
        "Invalid unit system. Must be metric, imperial, or custom.",
    ),
    (
        "SETTINGS_INVALID_DISTANCE_UNIT",
        "Invalid distance unit. Must be km or mi.",
    ),
    (
        "SETTINGS_INVALID_VOLUME_UNIT",
        "Invalid volume unit. Must be l or gal.",
    ),
    ("SETTINGS_INVALID_CURRENCY", "Invalid currency."),
    ("SETTINGS_INVALID_LOCALE", "Invalid locale."),
    (
        "STATS_INVALID_DATE_FILTER",
        "Invalid date filter. Use YYYY-MM-DD format.",
    ),
    (
        "IMPORT_VERSION_MISMATCH",
        "Export version is incompatible with this server version.",
    ),
    (
        "IMPORT_INVALID_MODE",
        "Invalid import mode. Must be replace or merge.",
    ),
    (
        "IMPORT_VALIDATION_ERROR",
        "Import data contains invalid records.",
    ),
];

/// Map an error code to a default human-readable message.
///
/// Codes not listed here get a generic fallback. The frontend localizes known
/// codes and uses this message only as a fallback.
fn default_message(code: &str) -> &'static str {
    for &(known_code, message) in ERROR_MESSAGES {
        if known_code == code {
            return message;
        }
    }
    "An error occurred."
}

/// Deserialize a nullable field for PATCH semantics.
///
/// - Field absent from JSON → `None` (keep current value)
/// - Field explicitly `null` → `Some(None)` (clear to NULL)
/// - Field has a value → `Some(Some(value))`
///
/// # Errors
///
/// Returns a serde error if the value cannot be deserialized as `Option<T>`.
#[allow(clippy::option_option)]
pub fn deserialize_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let value = Option::<T>::deserialize(deserializer)?;
    Ok(Some(value))
}

/// Convert a `sqlx::Error` into an `ApiError::InternalError`, logging the
/// underlying error without leaking database details to the client.
#[allow(clippy::needless_pass_by_value)]
pub fn db_error(e: sqlx::Error) -> ApiError {
    tracing::error!("Database error: {e}");
    ApiError::InternalError("INTERNAL_ERROR")
}

/// Custom JSON body extractor that returns an `ApiError` on parse failure
/// instead of axum's default plain-text rejection.
///
/// Use `JsonBody<T>` in handler signatures where you would normally use
/// `axum::Json<T>` to keep all error responses in the same JSON shape.
pub struct JsonBody<T>(pub T);

impl<S, T> axum::extract::FromRequest<S> for JsonBody<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => {
                tracing::debug!("JSON parse error: {rejection}");
                Err(match rejection {
                    JsonRejection::MissingJsonContentType(_) => {
                        ApiError::BadRequest("INVALID_REQUEST_BODY")
                    }
                    _ => ApiError::BadRequest("INVALID_REQUEST_BODY"),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;

    use super::*;

    #[test]
    fn into_response_sets_correct_status() {
        let response = ApiError::Unauthorized("AUTHENTICATION_REQUIRED").into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = ApiError::NotFound("NOT_FOUND").into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = ApiError::Validation("VALIDATION_ERROR").into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let response = ApiError::Conflict("CONFLICT_ERROR").into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);

        let response = ApiError::BadRequest("BAD_REQUEST_ERROR").into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = ApiError::InternalError("INTERNAL_ERROR").into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn unauthorized_uses_the_stable_json_contract() {
        let response = ApiError::Unauthorized("AUTHENTICATION_REQUIRED").into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().get("location").is_none());
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body should be readable")
            .to_bytes();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).expect("body should be JSON"),
            json!({
                "code": "AUTHENTICATION_REQUIRED",
                "message": "Authentication is required."
            })
        );
    }

    #[test]
    fn default_message_returns_known_messages() {
        assert_eq!(
            default_message("AUTHENTICATION_REQUIRED"),
            "Authentication is required."
        );
        assert_eq!(
            default_message("INTERNAL_ERROR"),
            "An unexpected error occurred."
        );
        assert_eq!(
            default_message("INVALID_REQUEST_BODY"),
            "The request body is missing or malformed."
        );
    }

    #[test]
    fn default_message_returns_fallback_for_unknown_code() {
        assert_eq!(default_message("SOME_UNKNOWN_CODE"), "An error occurred.");
    }

    #[test]
    fn every_error_code_has_a_translation_in_each_locale() {
        let locales = [
            ("en", include_str!("../../ui/src/lib/i18n/en.json")),
            ("de", include_str!("../../ui/src/lib/i18n/de.json")),
        ];

        for (locale, source) in locales {
            let translations: serde_json::Value =
                serde_json::from_str(source).expect("translation file should contain valid JSON");
            for &(code, _) in ERROR_MESSAGES {
                let key = format!("error.{code}");
                assert!(
                    translations.get(&key).is_some(),
                    "{locale}.json is missing {key}"
                );
            }
        }
    }
}
