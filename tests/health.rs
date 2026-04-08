mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn health_returns_ok_with_version() {
    let app = common::test_app().await;

    let response = app
        .oneshot(common::json_request("GET", "/health", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json = common::body_json(response).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn health_response_has_correct_shape() {
    let app = common::test_app().await;

    let response = app
        .oneshot(common::json_request("GET", "/health", None))
        .await
        .unwrap();

    let json = common::body_json(response).await;

    // Must have exactly these two fields.
    assert!(json.get("status").is_some(), "missing 'status' field");
    assert!(json.get("version").is_some(), "missing 'version' field");
    assert!(json["status"].is_string());
    assert!(json["version"].is_string());
}
