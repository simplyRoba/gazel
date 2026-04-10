mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Singleton constraint ─────────────────────────────────

#[tokio::test]
async fn singleton_constraint_rejects_second_row() {
    let pool = common::test_pool().await;
    let result = sqlx::query("INSERT INTO settings (id) VALUES (2)")
        .execute(&pool)
        .await;
    assert!(result.is_err(), "Should reject insert with id != 1");
}

// ── Read ─────────────────────────────────────────────────

#[tokio::test]
async fn get_settings_returns_defaults() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request("GET", "/api/settings", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["unit_system"], "metric");
    assert_eq!(json["distance_unit"], "km");
    assert_eq!(json["volume_unit"], "l");
    assert_eq!(json["currency"], "EUR");
    assert_eq!(json["color_mode"], "system");
    assert_eq!(json["locale"], "en");
}

// ── Update single field ──────────────────────────────────

#[tokio::test]
async fn update_single_field() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"color_mode":"dark"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["color_mode"], "dark");
    // Other fields unchanged.
    assert_eq!(json["unit_system"], "metric");
    assert_eq!(json["distance_unit"], "km");
    assert_eq!(json["volume_unit"], "l");
    assert_eq!(json["currency"], "EUR");
    assert_eq!(json["locale"], "en");
}

// ── Update multiple fields ───────────────────────────────

#[tokio::test]
async fn update_multiple_fields() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"unit_system":"imperial","distance_unit":"mi","volume_unit":"gal"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["unit_system"], "imperial");
    assert_eq!(json["distance_unit"], "mi");
    assert_eq!(json["volume_unit"], "gal");
}

// ── Empty body ───────────────────────────────────────────

#[tokio::test]
async fn update_empty_body_preserves_values() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request("PUT", "/api/settings", Some(r#"{}"#)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["unit_system"], "metric");
    assert_eq!(json["distance_unit"], "km");
    assert_eq!(json["volume_unit"], "l");
    assert_eq!(json["currency"], "EUR");
    assert_eq!(json["color_mode"], "system");
    assert_eq!(json["locale"], "en");
}

// ── Validation: invalid color_mode ───────────────────────

#[tokio::test]
async fn invalid_color_mode_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"color_mode":"purple"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_COLOR_MODE");
}

// ── Validation: all other invalid fields ─────────────────

#[tokio::test]
async fn invalid_unit_system_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"unit_system":"martian"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_UNIT_SYSTEM");
}

#[tokio::test]
async fn invalid_distance_unit_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"distance_unit":"parsec"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_DISTANCE_UNIT");
}

#[tokio::test]
async fn invalid_volume_unit_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"volume_unit":"barrel"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_VOLUME_UNIT");
}

#[tokio::test]
async fn invalid_currency_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"currency":"DOGECOIN"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_CURRENCY");
}

#[tokio::test]
async fn invalid_locale_returns_422() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"locale":"klingon"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "SETTINGS_INVALID_LOCALE");
}

#[tokio::test]
async fn update_locale_to_de_returns_200() {
    let app = common::test_app().await;
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            "/api/settings",
            Some(r#"{"locale":"de"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["locale"], "de");
}
