mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

#[tokio::test]
async fn accepts_an_allowlisted_public_client_diagnostic() {
    let app = common::test_app().await;
    let response = app
        .oneshot(common::json_request(
            "POST",
            "/client-diagnostics",
            Some(
                r#"{
                    "stage":"dashboard_loading_snapshot",
                    "outcome":"snapshot",
                    "pathname":"/",
                    "settings_loading":false,
                    "vehicles_loading":false,
                    "active_vehicle_selected":true,
                    "fillups_loading":true,
                    "stats_loading":false
                }"#,
            ),
        ))
        .await
        .expect("client diagnostic request should succeed");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rejects_unallowlisted_or_log_unsafe_client_diagnostics() {
    for body in [
        r#"{"stage":"custom","outcome":"failed","pathname":"/"}"#,
        r#"{"stage":"window_error","outcome":"failed","pathname":"/?secret=value"}"#,
        r#"{"stage":"window_error","outcome":"failed","pathname":"/","details":{"arbitrary":"content"}}"#,
    ] {
        let response = common::test_app()
            .await
            .oneshot(common::json_request(
                "POST",
                "/client-diagnostics",
                Some(body),
            ))
            .await
            .expect("client diagnostic request should succeed");

        assert!(
            matches!(
                response.status(),
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ),
            "unexpected status for rejected diagnostic: {}",
            response.status()
        );
    }
}
