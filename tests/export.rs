mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────

async fn create_vehicle(app: &mut axum::Router, json: &str) -> serde_json::Value {
    let req = common::json_request("POST", "/api/vehicles", Some(json));
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    common::body_json(resp).await
}

async fn create_fillup(app: &mut axum::Router, vehicle_id: i64, json: &str) {
    let req = common::json_request(
        "POST",
        &format!("/api/vehicles/{vehicle_id}/fillups"),
        Some(json),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

// ── Full export ─────────────────────────────────────────

#[tokio::test]
async fn export_empty_database() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let disposition = resp
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(disposition, "attachment; filename=\"gazel-export.json\"");

    let json = common::body_json(resp).await;
    assert!(json["version"].is_string());
    assert!(json["exported_at"].is_string());
    assert_eq!(json["vehicles"], serde_json::json!([]));
}

#[tokio::test]
async fn export_populated_database() {
    let mut app = common::test_app().await;

    let v = create_vehicle(&mut app, r#"{"name":"Test Car","make":"Toyota"}"#).await;
    let vid = v["id"].as_i64().unwrap();

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-02-01","fuel_amount":25.0,"odometer":11000,"cost":45.0}"#,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles"].as_array().unwrap().len(), 1);

    let vehicle = &json["vehicles"][0];
    assert_eq!(vehicle["name"], "Test Car");
    assert_eq!(vehicle["make"], "Toyota");
    assert_eq!(vehicle["fillups"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn export_fillups_sorted_by_date_ascending() {
    let mut app = common::test_app().await;

    let v = create_vehicle(&mut app, r#"{"name":"Car"}"#).await;
    let vid = v["id"].as_i64().unwrap();

    // Create in ascending odometer order (required by validation)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":20.0,"odometer":10000,"cost":35.0}"#,
    )
    .await;
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-03-01","fuel_amount":30.0,"odometer":12000,"cost":50.0}"#,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();

    let json = common::body_json(resp).await;
    let fillups = json["vehicles"][0]["fillups"].as_array().unwrap();
    assert_eq!(fillups.len(), 2);
    // Should be sorted by date ascending
    assert_eq!(fillups[0]["date"], "2026-01-01");
    assert_eq!(fillups[1]["date"], "2026-03-01");
}

// ── Single vehicle export ───────────────────────────────

#[tokio::test]
async fn export_single_vehicle() {
    let mut app = common::test_app().await;

    let v1 = create_vehicle(&mut app, r#"{"name":"Car One"}"#).await;
    let vid1 = v1["id"].as_i64().unwrap();
    create_vehicle(&mut app, r#"{"name":"Car Two"}"#).await;

    create_fillup(
        &mut app,
        vid1,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{vid1}/export"),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let disposition = resp
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(
        disposition,
        "attachment; filename=\"gazel-export-car-one.json\""
    );

    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles"].as_array().unwrap().len(), 1);
    assert_eq!(json["vehicles"][0]["name"], "Car One");
    assert_eq!(json["vehicles"][0]["fillups"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn export_vehicle_not_found() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request(
            "GET",
            "/api/vehicles/999/export",
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

#[tokio::test]
async fn export_version_matches_cargo_version() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();

    let json = common::body_json(resp).await;
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}
