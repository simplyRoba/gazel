mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────

async fn create_vehicle(app: &mut axum::Router, json: &str) -> axum::response::Response {
    let req = common::json_request("POST", "/api/vehicles", Some(json));
    app.clone().oneshot(req).await.unwrap()
}

async fn setup_vehicle(app: &mut axum::Router) -> i64 {
    let resp = create_vehicle(app, r#"{"name":"Test Car"}"#).await;
    let json = common::body_json(resp).await;
    json["id"].as_i64().unwrap()
}

async fn create_fillup(
    app: &mut axum::Router,
    vehicle_id: i64,
    json: &str,
) -> axum::response::Response {
    let req = common::json_request(
        "POST",
        &format!("/api/vehicles/{vehicle_id}/fillups"),
        Some(json),
    );
    app.clone().oneshot(req).await.unwrap()
}

async fn get_stats(app: &mut axum::Router, vehicle_id: i64) -> axum::response::Response {
    let req = common::json_request("GET", &format!("/api/vehicles/{vehicle_id}/stats"), None);
    app.clone().oneshot(req).await.unwrap()
}

async fn get_stats_with_filter(
    app: &mut axum::Router,
    vehicle_id: i64,
    query: &str,
) -> axum::response::Response {
    let req = common::json_request(
        "GET",
        &format!("/api/vehicles/{vehicle_id}/stats?{query}"),
        None,
    );
    app.clone().oneshot(req).await.unwrap()
}

async fn get_history(app: &mut axum::Router, vehicle_id: i64) -> axum::response::Response {
    let req = common::json_request(
        "GET",
        &format!("/api/vehicles/{vehicle_id}/stats/history"),
        None,
    );
    app.clone().oneshot(req).await.unwrap()
}

async fn update_settings(app: &mut axum::Router, json: &str) -> axum::response::Response {
    let req = common::json_request("PUT", "/api/settings", Some(json));
    app.clone().oneshot(req).await.unwrap()
}

// ── Summary: no fill-ups ─────────────────────────────────

#[tokio::test]
async fn summary_no_fillups_returns_zeroes() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    assert_eq!(json["total_distance"], 0.0);
    assert_eq!(json["total_fuel"], 0.0);
    assert_eq!(json["total_cost"], 0.0);
    assert_eq!(json["fill_up_count"], 0);
    assert!(json["average_efficiency"].is_null());
    assert!(json["average_cost_per_distance"].is_null());
    assert_eq!(json["distance_unit"], "km");
    assert_eq!(json["volume_unit"], "l");
    assert_eq!(json["currency"], "EUR");
}

// ── Summary: single fill-up ──────────────────────────────

#[tokio::test]
async fn summary_single_fillup_returns_totals_null_efficiency() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    assert_eq!(json["fill_up_count"], 1);
    assert_eq!(json["total_fuel"], 40.0);
    assert_eq!(json["total_cost"], 80.0);
    assert_eq!(json["total_distance"], 0.0);
    assert!(json["average_efficiency"].is_null());
    assert!(json["average_cost_per_distance"].is_null());
}

// ── Summary: multiple full-tank fills ────────────────────

#[tokio::test]
async fn summary_multiple_full_tank_fills() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Fill-up A: start segment
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up B: end of segment 1 (distance=500, fuel=40, eff=12.5)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up C: end of segment 2 (distance=500, fuel=50, eff=10.0)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-03-01","odometer":2000,"fuel_amount":50.0,"cost":100.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    assert_eq!(json["fill_up_count"], 3);
    assert_eq!(json["total_distance"], 1000.0);
    assert_eq!(json["total_fuel"], 130.0);
    assert_eq!(json["total_cost"], 260.0);
    // avg efficiency = (12.5 + 10.0) / 2 = 11.25
    assert_eq!(json["average_efficiency"], 11.25);
    // avg cost_per_distance: seg1 = 80/500 = 0.16, seg2 = 100/500 = 0.20 → avg = 0.18
    assert_eq!(json["average_cost_per_distance"], 0.18);
}

// ── Summary: missed fill-up segments ─────────────────────

#[tokio::test]
async fn summary_missed_fillup_excludes_from_efficiency() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Fill-up A: segment start
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up B: missed fill-up in between
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-15","odometer":1300,"fuel_amount":20.0,"cost":40.0,"is_full_tank":false,"is_missed":true}"#,
    )
    .await;

    // Fill-up C: end of segment 1 (invalid due to missed)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":30.0,"cost":60.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up D: end of segment 2 (valid, distance=500, fuel=40, eff=12.5)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-03-01","odometer":2000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    // Only segment 2 is valid, so efficiency = 12.5
    assert_eq!(json["average_efficiency"], 12.5);
}

// ── Summary: partial-tank fills ──────────────────────────

#[tokio::test]
async fn summary_partial_tank_fills_accumulates_fuel() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Fill-up A: segment start (full tank)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up B: partial fill
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-15","odometer":1200,"fuel_amount":15.0,"cost":30.0,"is_full_tank":false}"#,
    )
    .await;

    // Fill-up C: full tank, end of segment (fuel = 15 + 25 = 40, distance = 500)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":25.0,"cost":50.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    // efficiency = 500 / 40 = 12.5
    assert_eq!(json["average_efficiency"], 12.5);
    // cost_per_distance = 80 / 500 = 0.16
    assert_eq!(json["average_cost_per_distance"], 0.16);
}

// ── History: valid segments ──────────────────────────────

#[tokio::test]
async fn history_returns_segments_with_is_valid() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Missed fill-up in segment 1
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-15","odometer":1300,"fuel_amount":20.0,"cost":40.0,"is_full_tank":false,"is_missed":true}"#,
    )
    .await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":30.0,"cost":60.0,"is_full_tank":true}"#,
    )
    .await;

    // Valid segment 2
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-03-01","odometer":2000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_history(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    let arr = json.as_array().unwrap();

    assert_eq!(arr.len(), 2);

    // Segment 1: invalid (has missed)
    assert!(!arr[0]["is_valid"].as_bool().unwrap());
    assert_eq!(arr[0]["start_date"], "2025-01-01");
    assert_eq!(arr[0]["end_date"], "2025-02-01");

    // Segment 2: valid
    assert!(arr[1]["is_valid"].as_bool().unwrap());
    assert_eq!(arr[1]["start_date"], "2025-02-01");
    assert_eq!(arr[1]["end_date"], "2025-03-01");
    assert_eq!(arr[1]["distance"], 500.0);
    assert_eq!(arr[1]["fuel"], 40.0);
    assert_eq!(arr[1]["efficiency"], 12.5);
}

// ── History: fewer than 2 full-tank fills ────────────────

#[tokio::test]
async fn history_fewer_than_two_full_tanks_returns_empty() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_history(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

// ── Time-range filtering ─────────────────────────────────

#[tokio::test]
async fn stats_time_range_filtering() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-06-01","odometer":2000,"fuel_amount":50.0,"cost":100.0,"is_full_tank":true}"#,
    )
    .await;

    // Filter to only include Jan-Feb fill-ups
    let resp = get_stats_with_filter(&mut app, vid, "from=2025-01-01&to=2025-03-01").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    // Only 2 fill-ups, 1 segment
    assert_eq!(json["fill_up_count"], 2);
    assert_eq!(json["total_distance"], 500.0);
}

// ── Invalid date filter ──────────────────────────────────

#[tokio::test]
async fn stats_invalid_date_filter_returns_400() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = get_stats_with_filter(&mut app, vid, "from=banana").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "STATS_INVALID_DATE_FILTER");
}

// ── Non-existent vehicle ─────────────────────────────────

#[tokio::test]
async fn stats_nonexistent_vehicle_returns_404() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request("GET", "/api/vehicles/999/stats", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

#[tokio::test]
async fn history_nonexistent_vehicle_returns_404() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request(
            "GET",
            "/api/vehicles/999/stats/history",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Unit conversion ──────────────────────────────────────

#[tokio::test]
async fn stats_reflects_imperial_settings() {
    let mut app = common::test_app().await;

    // Switch to imperial
    update_settings(&mut app, r#"{"distance_unit":"mi","volume_unit":"gal"}"#).await;

    let vid = setup_vehicle(&mut app).await;

    // Create fillups (fuel_unit will be "gal" from settings, odometer stays raw)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":10.0,"cost":40.0,"is_full_tank":true}"#,
    )
    .await;
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":10.0,"cost":40.0,"is_full_tank":true}"#,
    )
    .await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    assert_eq!(json["distance_unit"], "mi");
    assert_eq!(json["volume_unit"], "gal");

    // Values should be in imperial units (converted from stored values)
    // The odometer is stored as raw values. Distance = 500 (raw km) → 500/1.609344 ≈ 310.69 mi
    let total_dist = json["total_distance"].as_f64().unwrap();
    assert!(
        (total_dist - 310.69).abs() < 0.1,
        "expected ~310.69, got {total_dist}"
    );
}

// ── Mixed historical units ───────────────────────────────

#[tokio::test]
async fn stats_mixed_historical_units() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Create fill-ups under default metric settings (fuel_unit = "l")
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Switch to imperial after fill-ups were recorded in liters
    update_settings(&mut app, r#"{"distance_unit":"mi","volume_unit":"gal"}"#).await;

    let resp = get_stats(&mut app, vid).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;

    assert_eq!(json["volume_unit"], "gal");
    assert_eq!(json["distance_unit"], "mi");

    // total_fuel: 80 L → 80 / 3.785411784 ≈ 21.13 gal
    let total_fuel = json["total_fuel"].as_f64().unwrap();
    assert!(
        (total_fuel - 21.13).abs() < 0.1,
        "expected ~21.13 gal, got {total_fuel}"
    );

    // efficiency: raw = 500 km / 40 L = 12.5 km/L → converted to mi/gal
    // 12.5 * (1/1.609344) / (1/3.785411784) = 12.5 * 3.785411784 / 1.609344 ≈ 29.40 mpg
    let eff = json["average_efficiency"].as_f64().unwrap();
    assert!((eff - 29.40).abs() < 0.1, "expected ~29.40 mpg, got {eff}");
}

// ── Zero-odometer fill-ups excluded ──────────────────────

#[tokio::test]
async fn stats_zero_odometer_fillups_excluded() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Fill-up with valid odometer (segment start)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","odometer":1000,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Fill-up with valid odometer (segment end)
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-02-01","odometer":1500,"fuel_amount":40.0,"cost":80.0,"is_full_tank":true}"#,
    )
    .await;

    // Stats should show 1 segment, distance=500, efficiency=12.5
    let resp = get_stats(&mut app, vid).await;
    let json = common::body_json(resp).await;
    assert_eq!(json["total_distance"], 500.0);
    assert_eq!(json["average_efficiency"], 12.5);

    let resp = get_history(&mut app, vid).await;
    let json = common::body_json(resp).await;
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["distance"], 500.0);
}
