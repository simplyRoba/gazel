mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────

async fn create_vehicle(app: &mut axum::Router, json: &str) -> axum::response::Response {
    let req = common::json_request("POST", "/api/vehicles", Some(json));
    app.clone().oneshot(req).await.unwrap()
}

// ── List ─────────────────────────────────────────────────

#[tokio::test]
async fn list_empty() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn list_ordered_by_name() {
    let mut app = common::test_app().await;

    create_vehicle(&mut app, r#"{"name":"Zebra"}"#).await;
    create_vehicle(&mut app, r#"{"name":"Alpha"}"#).await;
    create_vehicle(&mut app, r#"{"name":"Middle"}"#).await;

    let resp = app
        .clone()
        .oneshot(common::json_request("GET", "/api/vehicles", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    let names: Vec<&str> = json
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Alpha", "Middle", "Zebra"]);
}

// ── Create ───────────────────────────────────────────────

#[tokio::test]
async fn create_with_all_fields() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(
        &mut app,
        r#"{"name":"Civic","make":"Honda","model":"Civic","year":2024,"fuel_type":"gasoline","notes":"Daily driver"}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "Civic");
    assert_eq!(json["make"], "Honda");
    assert_eq!(json["model"], "Civic");
    assert_eq!(json["year"], 2024);
    assert_eq!(json["fuel_type"], "gasoline");
    assert_eq!(json["notes"], "Daily driver");
    assert!(json["id"].is_number());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

#[tokio::test]
async fn create_with_only_name() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"My Car"}"#).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "My Car");
    assert_eq!(json["fuel_type"], "gasoline");
    assert!(json["make"].is_null());
    assert!(json["model"].is_null());
    assert!(json["year"].is_null());
    assert!(json["notes"].is_null());
}

#[tokio::test]
async fn create_without_name() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{}"#).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NAME_REQUIRED");
}

#[tokio::test]
async fn create_with_empty_name() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"   "}"#).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NAME_REQUIRED");
}

#[tokio::test]
async fn create_with_invalid_fuel_type() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"Car","fuel_type":"plutonium"}"#).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_FUEL_TYPE");
}

#[tokio::test]
async fn create_with_invalid_year() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"Car","year":1800}"#).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_YEAR");
}

#[tokio::test]
async fn create_trims_name() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"  Civic  "}"#).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "Civic");
}

#[tokio::test]
async fn create_with_null_year() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name":"Car","year":null}"#).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert!(json["year"].is_null());
}

// ── Get ──────────────────────────────────────────────────

#[tokio::test]
async fn get_existing() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Test"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["id"], id);
    assert_eq!(json["name"], "Test");
}

#[tokio::test]
async fn get_nonexistent() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request("GET", "/api/vehicles/999", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Update (PUT) ─────────────────────────────────────────

#[tokio::test]
async fn update_changes_fields() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Old Name"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();
    let original_updated = created["updated_at"].as_str().unwrap().to_string();

    // Small delay to ensure timestamp changes
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"name":"New Name","fuel_type":"diesel","make":"Toyota","model":"Hilux","year":2023,"notes":"Updated"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "New Name");
    assert_eq!(json["fuel_type"], "diesel");
    assert_eq!(json["make"], "Toyota");
    assert_ne!(json["updated_at"].as_str().unwrap(), original_updated);
}

#[tokio::test]
async fn update_with_invalid_fuel_type() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Car"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"name":"Car","fuel_type":"plutonium"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_FUEL_TYPE");
}

#[tokio::test]
async fn update_with_invalid_year() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Car"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"name":"Car","fuel_type":"gasoline","year":1800}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_YEAR");
}

// ── Patch ────────────────────────────────────────────────

#[tokio::test]
async fn patch_with_invalid_fuel_type() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Car"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PATCH",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"fuel_type":"plutonium"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_FUEL_TYPE");
}

#[tokio::test]
async fn patch_with_invalid_year() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Car"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PATCH",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"year":1800}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_INVALID_YEAR");
}

#[tokio::test]
async fn patch_updates_only_sent_fields() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(
        &mut app,
        r#"{"name":"Original","make":"Honda","model":"Civic","fuel_type":"gasoline"}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PATCH",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"name":"Renamed"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["name"], "Renamed");
    // Preserved fields
    assert_eq!(json["make"], "Honda");
    assert_eq!(json["model"], "Civic");
    assert_eq!(json["fuel_type"], "gasoline");
}

#[tokio::test]
async fn patch_null_clears_field() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"Car","notes":"Some notes"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["notes"], "Some notes");

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PATCH",
            &format!("/api/vehicles/{id}"),
            Some(r#"{"notes":null}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert!(json["notes"].is_null());
}

// ── Delete ───────────────────────────────────────────────

#[tokio::test]
async fn delete_returns_204() {
    let mut app = common::test_app().await;
    let create_resp = create_vehicle(&mut app, r#"{"name":"ToDelete"}"#).await;
    let created = common::body_json(create_resp).await;
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Subsequent GET should return 404
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{id}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request("DELETE", "/api/vehicles/999", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Bad request ──────────────────────────────────────────

#[tokio::test]
async fn malformed_json_returns_400() {
    let mut app = common::test_app().await;
    let resp = create_vehicle(&mut app, r#"{"name": invalid}"#).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "INVALID_REQUEST_BODY");
}
