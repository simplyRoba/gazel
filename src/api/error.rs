use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde_json::json;

/// API error type that maps to a consistent JSON response.
///
/// Each variant carries a `&'static str` error code. The `IntoResponse`
/// implementation maps each variant to the appropriate HTTP status and returns
/// `{ "code": "<CODE>", "message": "<human-readable>" }`.
pub enum ApiError {
    NotFound(&'static str),
    Validation(&'static str),
    Conflict(&'static str),
    BadRequest(&'static str),
    InternalError(&'static str),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
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

/// Map an error code to a default human-readable message.
///
/// Codes not listed here get a generic fallback. As the app grows, new codes
/// are added to this match. Future i18n will replace this with translation
/// lookups on the frontend.
fn default_message(code: &str) -> &'static str {
    match code {
        "INTERNAL_ERROR" => "An unexpected error occurred.",
        "INVALID_REQUEST_BODY" => "The request body is missing or malformed.",
        _ => "An error occurred.",
    }
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
    use super::*;

    #[test]
    fn into_response_sets_correct_status() {
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

    #[test]
    fn default_message_returns_known_messages() {
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
}
