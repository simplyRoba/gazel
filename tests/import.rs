mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────

fn valid_export_json() -> String {
    format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [
                {{
                    "name": "Test Car",
                    "make": "Toyota",
                    "model": "Corolla",
                    "year": 2020,
                    "fuel_type": "gasoline",
                    "notes": null,
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z",
                    "fillups": [
                        {{
                            "date": "2026-01-01",
                            "odometer": 10000.0,
                            "fuel_amount": 30.0,
                            "fuel_unit": "l",
                            "cost": 50.0,
                            "currency": "USD",
                            "is_full_tank": true,
                            "is_missed": false,
                            "station": "Shell",
                            "notes": null,
                            "created_at": "2026-01-01T00:00:00Z",
                            "updated_at": "2026-01-01T00:00:00Z"
                        }},
                        {{
                            "date": "2026-02-01",
                            "odometer": 11000.0,
                            "fuel_amount": 25.0,
                            "fuel_unit": "l",
                            "cost": 45.0,
                            "currency": "USD",
                            "is_full_tank": true,
                            "is_missed": false,
                            "station": null,
                            "notes": null,
                            "created_at": "2026-02-01T00:00:00Z",
                            "updated_at": "2026-02-01T00:00:00Z"
                        }}
                    ]
                }}
            ]
        }}"#,
        env!("CARGO_PKG_VERSION")
    )
}

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

// ── Replace mode ────────────────────────────────────────

#[tokio::test]
async fn import_replace_valid() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request(
            "POST",
            "/api/import",
            Some(&valid_export_json()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles_created"], 1);
    assert_eq!(json["fillups_created"], 2);
}

#[tokio::test]
async fn import_replace_clears_existing_data() {
    let mut app = common::test_app().await;

    // Create existing data
    let v = create_vehicle(&mut app, r#"{"name":"Old Car"}"#).await;
    let vid = v["id"].as_i64().unwrap();
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","fuel_amount":20.0,"odometer":5000,"cost":30.0}"#,
    )
    .await;

    // Import replaces everything
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import",
            Some(&valid_export_json()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles_created"], 1);

    // Verify old vehicle is gone
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    let vehicles = common::body_json(resp).await;
    let vehicles = vehicles.as_array().unwrap();
    assert_eq!(vehicles.len(), 1);
    assert_eq!(vehicles[0]["name"], "Test Car");
}

// ── Preview mode ────────────────────────────────────────

#[tokio::test]
async fn import_preview_replace() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request(
            "POST",
            "/api/import?preview=true",
            Some(&valid_export_json()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["preview"], true);
    assert_eq!(json["vehicles"], 1);
    assert_eq!(json["fillups"], 2);
}

#[tokio::test]
async fn import_preview_does_not_modify_data() {
    let mut app = common::test_app().await;

    let v = create_vehicle(&mut app, r#"{"name":"Existing"}"#).await;
    let vid = v["id"].as_i64().unwrap();

    // Preview
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import?preview=true",
            Some(&valid_export_json()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify existing vehicle is still there
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{vid}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "Existing");
}

// ── Version checks ──────────────────────────────────────

#[tokio::test]
async fn import_version_mismatch_major() {
    let app = common::test_app().await;

    let body = r#"{
        "version": "99.0.0",
        "exported_at": "2026-01-01T00:00:00Z",
        "vehicles": []
    }"#;

    let resp = app
        .oneshot(common::json_request("POST", "/api/import", Some(body)))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "IMPORT_VERSION_MISMATCH");
}

#[tokio::test]
async fn import_version_mismatch_minor() {
    let app = common::test_app().await;

    // Same major, different minor
    let parts: Vec<&str> = env!("CARGO_PKG_VERSION").split('.').collect();
    let major = parts[0];
    let minor: u32 = parts[1].parse::<u32>().unwrap() + 1;
    let body = format!(
        r#"{{
            "version": "{major}.{minor}.0",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": []
        }}"#
    );

    let resp = app
        .oneshot(common::json_request("POST", "/api/import", Some(&body)))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "IMPORT_VERSION_MISMATCH");
}

#[tokio::test]
async fn import_patch_version_difference_allowed() {
    let app = common::test_app().await;

    let parts: Vec<&str> = env!("CARGO_PKG_VERSION").split('.').collect();
    let major = parts[0];
    let minor = parts[1];
    let body = format!(
        r#"{{
            "version": "{major}.{minor}.99",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": []
        }}"#
    );

    let resp = app
        .oneshot(common::json_request("POST", "/api/import", Some(&body)))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Validation errors ───────────────────────────────────

#[tokio::test]
async fn import_malformed_json() {
    let app = common::test_app().await;

    let resp = app
        .oneshot(common::json_request(
            "POST",
            "/api/import",
            Some("not json at all"),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn import_missing_vehicle_name() {
    let app = common::test_app().await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "",
                "fuel_type": "gasoline",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": []
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .oneshot(common::json_request("POST", "/api/import", Some(&body)))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "IMPORT_VALIDATION_ERROR");
}

#[tokio::test]
async fn import_negative_odometer() {
    let app = common::test_app().await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "Car",
                "fuel_type": "gasoline",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": [{{
                    "date": "2026-01-01",
                    "odometer": -100.0,
                    "fuel_amount": 30.0,
                    "fuel_unit": "l",
                    "cost": 50.0,
                    "currency": "USD",
                    "is_full_tank": true,
                    "is_missed": false,
                    "station": null,
                    "notes": null,
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z"
                }}]
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .oneshot(common::json_request("POST", "/api/import", Some(&body)))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "IMPORT_VALIDATION_ERROR");
}

#[tokio::test]
async fn import_invalid_mode() {
    let app = common::test_app().await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": []
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .oneshot(common::json_request(
            "POST",
            "/api/import?mode=invalid",
            Some(&body),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "IMPORT_INVALID_MODE");
}

#[tokio::test]
async fn import_atomicity_preserves_data_on_failure() {
    let mut app = common::test_app().await;

    // Create existing data
    let v = create_vehicle(&mut app, r#"{"name":"Keeper"}"#).await;
    let vid = v["id"].as_i64().unwrap();
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2025-01-01","fuel_amount":20.0,"odometer":5000,"cost":30.0}"#,
    )
    .await;

    // Attempt import with invalid data (empty name vehicle)
    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "  ",
                "fuel_type": "gasoline",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": []
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .clone()
        .oneshot(common::json_request("POST", "/api/import", Some(&body)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Verify original data is preserved
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    let vehicles = common::body_json(resp).await;
    let vehicles = vehicles.as_array().unwrap();
    assert_eq!(vehicles.len(), 1);
    assert_eq!(vehicles[0]["name"], "Keeper");
}

// ── Merge mode ──────────────────────────────────────────

#[tokio::test]
async fn import_merge_inserts_new_vehicle() {
    let mut app = common::test_app().await;

    create_vehicle(&mut app, r#"{"name":"Existing Car"}"#).await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "New Car",
                "fuel_type": "diesel",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": []
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import?mode=merge",
            Some(&body),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles_created"], 1);
    assert_eq!(json["vehicles_updated"], 0);

    // Both vehicles should exist
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    let vehicles = common::body_json(resp).await;
    assert_eq!(vehicles.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn import_merge_updates_existing_vehicle() {
    let mut app = common::test_app().await;

    create_vehicle(
        &mut app,
        r#"{"name":"Test Car","make":"Honda","fuel_type":"gasoline"}"#,
    )
    .await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "Test Car",
                "make": "Toyota",
                "model": "Corolla",
                "year": 2020,
                "fuel_type": "gasoline",
                "notes": null,
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": []
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import?mode=merge",
            Some(&body),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["vehicles_created"], 0);
    assert_eq!(json["vehicles_updated"], 1);

    // Verify fields updated
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    let vehicles = common::body_json(resp).await;
    let vehicles = vehicles.as_array().unwrap();
    assert_eq!(vehicles.len(), 1);
    assert_eq!(vehicles[0]["make"], "Toyota");
    assert_eq!(vehicles[0]["model"], "Corolla");
}

#[tokio::test]
async fn import_merge_skips_duplicate_fillups() {
    let mut app = common::test_app().await;

    let v = create_vehicle(&mut app, r#"{"name":"Test Car"}"#).await;
    let vid = v["id"].as_i64().unwrap();
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [{{
                "name": "Test Car",
                "fuel_type": "gasoline",
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
                "fillups": [
                    {{
                        "date": "2026-01-01",
                        "odometer": 10000.0,
                        "fuel_amount": 30.0,
                        "fuel_unit": "l",
                        "cost": 50.0,
                        "currency": "USD",
                        "is_full_tank": true,
                        "is_missed": false,
                        "station": null,
                        "notes": null,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-01T00:00:00Z"
                    }},
                    {{
                        "date": "2026-02-01",
                        "odometer": 11000.0,
                        "fuel_amount": 25.0,
                        "fuel_unit": "l",
                        "cost": 45.0,
                        "currency": "USD",
                        "is_full_tank": true,
                        "is_missed": false,
                        "station": null,
                        "notes": null,
                        "created_at": "2026-02-01T00:00:00Z",
                        "updated_at": "2026-02-01T00:00:00Z"
                    }}
                ]
            }}]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import?mode=merge",
            Some(&body),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["fillups_created"], 1); // only the new one
    assert_eq!(json["fillups_skipped"], 1); // the duplicate
}

#[tokio::test]
async fn import_merge_preview() {
    let mut app = common::test_app().await;

    create_vehicle(&mut app, r#"{"name":"Test Car"}"#).await;

    let body = format!(
        r#"{{
            "version": "{}",
            "exported_at": "2026-01-01T00:00:00Z",
            "vehicles": [
                {{
                    "name": "Test Car",
                    "fuel_type": "gasoline",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z",
                    "fillups": [{{
                        "date": "2026-01-01",
                        "odometer": 10000.0,
                        "fuel_amount": 30.0,
                        "fuel_unit": "l",
                        "cost": 50.0,
                        "currency": "USD",
                        "is_full_tank": true,
                        "is_missed": false,
                        "station": null,
                        "notes": null,
                        "created_at": "2026-01-01T00:00:00Z",
                        "updated_at": "2026-01-01T00:00:00Z"
                    }}]
                }},
                {{
                    "name": "New Car",
                    "fuel_type": "diesel",
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z",
                    "fillups": []
                }}
            ]
        }}"#,
        env!("CARGO_PKG_VERSION")
    );

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import?preview=true&mode=merge",
            Some(&body),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["preview"], true);
    assert_eq!(json["vehicles_existing"], 1);
    assert_eq!(json["vehicles_new"], 1);
    assert_eq!(json["fillups_new"], 1);
}

// ── Round-trip ──────────────────────────────────────────

#[tokio::test]
async fn round_trip_export_import() {
    let mut app = common::test_app().await;

    // Create data
    let v = create_vehicle(
        &mut app,
        r#"{"name":"Round Trip","make":"Ford","fuel_type":"diesel"}"#,
    )
    .await;
    let vid = v["id"].as_i64().unwrap();

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    // Export
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let export_json = common::body_json(resp).await;

    // Import into fresh app
    let app2 = common::test_app().await;
    let resp = app2
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/import",
            Some(&export_json.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Re-export and compare data
    let resp = app2
        .clone()
        .oneshot(common::json_request("GET", "/api/export", None))
        .await
        .unwrap();
    let re_export = common::body_json(resp).await;

    // Vehicles and fillups should match (timestamps may differ)
    assert_eq!(
        re_export["vehicles"][0]["name"],
        export_json["vehicles"][0]["name"]
    );
    assert_eq!(
        re_export["vehicles"][0]["make"],
        export_json["vehicles"][0]["make"]
    );
    assert_eq!(
        re_export["vehicles"][0]["fillups"]
            .as_array()
            .unwrap()
            .len(),
        export_json["vehicles"][0]["fillups"]
            .as_array()
            .unwrap()
            .len()
    );
}
