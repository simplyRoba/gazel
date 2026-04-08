use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request};
use axum::response::Response;
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use gazel::state::AppState;

/// Create an in-memory SQLite pool with all migrations applied.
///
/// Uses max 1 connection to avoid concurrency complications in tests.
/// Each call produces a fully isolated database.
pub async fn test_pool() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to create test pool");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Create a fully wired application router backed by an in-memory database.
pub async fn test_app() -> Router {
    let pool = test_pool().await;
    let state = AppState { pool };
    gazel::server::router(state)
}

/// Build an HTTP request with an optional JSON body.
pub fn json_request(method: &str, uri: &str, body: Option<&str>) -> Request<Body> {
    let method = method.parse::<Method>().expect("Invalid HTTP method");
    let mut builder = Request::builder().method(method).uri(uri);

    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }

    let body = body.map_or_else(Body::empty, |b| Body::from(b.to_string()));

    builder.body(body).expect("Failed to build request")
}

/// Read a response body as a `serde_json::Value`.
pub async fn body_json(response: Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("Failed to read response body")
        .to_bytes();

    serde_json::from_slice(&bytes).expect("Response body is not valid JSON")
}
