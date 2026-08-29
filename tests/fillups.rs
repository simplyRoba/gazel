mod common;

use axum::http::StatusCode;
use tower::ServiceExt;

// ── Helpers ──────────────────────────────────────────────

async fn create_vehicle(app: &mut axum::Router, json: &str) -> axum::response::Response {
    let req = common::json_request("POST", "/api/vehicles", Some(json));
    app.clone().oneshot(req).await.unwrap()
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

/// Create a vehicle and return its ID.
async fn setup_vehicle(app: &mut axum::Router) -> i64 {
    let resp = create_vehicle(app, r#"{"name":"Test Car"}"#).await;
    let json = common::body_json(resp).await;
    json["id"].as_i64().unwrap()
}

// ── List ─────────────────────────────────────────────────

async fn list_fillups(app: &axum::Router, uri: &str) -> serde_json::Value {
    let resp = app
        .clone()
        .oneshot(common::json_request("GET", uri, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    common::body_json(resp).await
}

async fn create_ordered_fillups(app: &mut axum::Router, vehicle_id: i64, count: u8) -> Vec<i64> {
    let mut ids = Vec::with_capacity(count.into());
    for day in 1..=count {
        let response = create_fillup(
            app,
            vehicle_id,
            &format!(
                r#"{{"date":"2026-01-{day:02}","fuel_amount":30.0,"odometer":{},"cost":50.0}}"#,
                10_000 + i64::from(day)
            ),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        ids.push(common::body_json(response).await["id"].as_i64().unwrap());
    }
    ids
}

fn item_ids(page: &serde_json::Value) -> Vec<i64> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fillup| fillup["id"].as_i64().unwrap())
        .collect()
}

#[tokio::test]
async fn list_empty_returns_empty_page() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let page = list_fillups(&app, &format!("/api/vehicles/{vid}/fillups")).await;
    assert_eq!(page["items"], serde_json::json!([]));
    assert!(page["next_cursor"].is_null());
}

#[tokio::test]
async fn list_uses_default_limit_and_returns_cursor_when_more_items_exist() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;
    let ids = create_ordered_fillups(&mut app, vid, 26).await;

    let page = list_fillups(&app, &format!("/api/vehicles/{vid}/fillups")).await;
    assert_eq!(
        item_ids(&page),
        ids.into_iter().rev().take(25).collect::<Vec<_>>()
    );
    assert!(page["next_cursor"].is_string());
}

#[tokio::test]
async fn list_honors_custom_limit() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;
    let ids = create_ordered_fillups(&mut app, vid, 4).await;

    let page = list_fillups(&app, &format!("/api/vehicles/{vid}/fillups?limit=2")).await;
    assert_eq!(
        item_ids(&page),
        ids.into_iter().rev().take(2).collect::<Vec<_>>()
    );
    assert!(page["next_cursor"].is_string());
}

#[tokio::test]
async fn list_orders_by_date_then_id_and_continues_after_same_date_items() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let first = common::body_json(
        create_fillup(
            &mut app,
            vid,
            r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
        )
        .await,
    )
    .await["id"]
        .as_i64()
        .unwrap();
    let same_date_first = common::body_json(
        create_fillup(
            &mut app,
            vid,
            r#"{"date":"2026-03-01","fuel_amount":30.0,"odometer":11000,"cost":50.0}"#,
        )
        .await,
    )
    .await["id"]
        .as_i64()
        .unwrap();
    let same_date_second = common::body_json(
        create_fillup(
            &mut app,
            vid,
            r#"{"date":"2026-03-01","fuel_amount":30.0,"odometer":12000,"cost":50.0}"#,
        )
        .await,
    )
    .await["id"]
        .as_i64()
        .unwrap();
    let middle = common::body_json(
        create_fillup(
            &mut app,
            vid,
            r#"{"date":"2026-02-01","fuel_amount":30.0,"odometer":13000,"cost":50.0}"#,
        )
        .await,
    )
    .await["id"]
        .as_i64()
        .unwrap();

    let first_page = list_fillups(&app, &format!("/api/vehicles/{vid}/fillups?limit=2")).await;
    assert_eq!(
        item_ids(&first_page),
        vec![same_date_second, same_date_first]
    );

    let cursor = first_page["next_cursor"].as_str().unwrap();
    let second_page = list_fillups(
        &app,
        &format!("/api/vehicles/{vid}/fillups?limit=2&cursor={cursor}"),
    )
    .await;
    assert_eq!(item_ids(&second_page), vec![middle, first]);
    assert!(second_page["next_cursor"].is_null());
}

#[tokio::test]
async fn list_continuation_returns_all_items_without_duplicates() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;
    let ids = create_ordered_fillups(&mut app, vid, 5).await;

    let first_page = list_fillups(&app, &format!("/api/vehicles/{vid}/fillups?limit=2")).await;
    let second_page = list_fillups(
        &app,
        &format!(
            "/api/vehicles/{vid}/fillups?limit=2&cursor={}",
            first_page["next_cursor"].as_str().unwrap()
        ),
    )
    .await;
    let terminal_page = list_fillups(
        &app,
        &format!(
            "/api/vehicles/{vid}/fillups?limit=2&cursor={}",
            second_page["next_cursor"].as_str().unwrap()
        ),
    )
    .await;

    let mut returned_ids = item_ids(&first_page);
    returned_ids.extend(item_ids(&second_page));
    returned_ids.extend(item_ids(&terminal_page));
    assert_eq!(returned_ids, ids.into_iter().rev().collect::<Vec<_>>());
    assert!(terminal_page["next_cursor"].is_null());
}

#[tokio::test]
async fn list_rejects_invalid_pagination_input_with_json_error_codes() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    for (query, code) in [
        ("limit=0", "FILLUP_INVALID_PAGE_LIMIT"),
        ("limit=101", "FILLUP_INVALID_PAGE_LIMIT"),
        ("limit=not-an-integer", "FILLUP_INVALID_PAGE_LIMIT"),
        ("cursor=not-a-cursor", "FILLUP_INVALID_CURSOR"),
        ("cursor=bm90LWpzb24", "FILLUP_INVALID_CURSOR"),
        (
            "cursor=eyJkYXRlIjoiICAgIiwiaWQiOjF9",
            "FILLUP_INVALID_CURSOR",
        ),
        (
            "cursor=eyJkYXRlIjoiMjAyNi0wMS0wMSIsImlkIjowfQ",
            "FILLUP_INVALID_CURSOR",
        ),
    ] {
        let resp = app
            .clone()
            .oneshot(common::json_request(
                "GET",
                &format!("/api/vehicles/{vid}/fillups?{query}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(common::body_json(resp).await["code"], code);
    }
}

#[tokio::test]
async fn list_for_nonexistent_vehicle() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request(
            "GET",
            "/api/vehicles/999/fillups",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Get ──────────────────────────────────────────────────

#[tokio::test]
async fn get_existing_fillup() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":40.0,"odometer":10000,"cost":60.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["id"], fid);
    assert_eq!(json["vehicle_id"], vid);
    assert_eq!(json["date"], "2026-04-01");
}

#[tokio::test]
async fn get_nonexistent_fillup() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{vid}/fillups/999"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_NOT_FOUND");
}

#[tokio::test]
async fn get_fillup_for_nonexistent_vehicle() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request(
            "GET",
            "/api/vehicles/999/fillups/1",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Create ───────────────────────────────────────────────

#[tokio::test]
async fn create_with_all_fields() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","odometer":15230.5,"fuel_amount":45.5,"cost":72.80,"is_full_tank":true,"is_missed":false,"station":"Shell Main St","notes":"Regular fill"}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["vehicle_id"], vid);
    assert_eq!(json["date"], "2026-04-09");
    assert_eq!(json["odometer"], 15230.5);
    assert_eq!(json["fuel_amount"], 45.5);
    // fuel_unit and currency are auto-populated from settings (defaults: "l" and "EUR")
    assert_eq!(json["fuel_unit"], "l");
    assert_eq!(json["cost"], 72.80);
    assert_eq!(json["currency"], "EUR");
    assert_eq!(json["is_full_tank"], true);
    assert_eq!(json["is_missed"], false);
    assert_eq!(json["station"], "Shell Main St");
    assert_eq!(json["notes"], "Regular fill");
    assert!(json["id"].is_number());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());
}

#[tokio::test]
async fn create_with_required_fields_only() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"odometer":5000,"cost":45.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    // fuel_unit and currency auto-populated from settings defaults
    assert_eq!(json["fuel_unit"], "l");
    assert_eq!(json["currency"], "EUR");
    // is_full_tank now defaults to true
    assert_eq!(json["is_full_tank"], true);
    assert_eq!(json["is_missed"], false);
    assert_eq!(json["odometer"], 5000.0);
    assert_eq!(json["cost"], 45.0);
    assert!(json["station"].is_null());
    assert!(json["notes"].is_null());
}

#[tokio::test]
async fn create_for_nonexistent_vehicle() {
    let mut app = common::test_app().await;
    let resp = create_fillup(
        &mut app,
        999,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Create validation ────────────────────────────────────

#[tokio::test]
async fn create_missing_date() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"fuel_amount":30.0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_DATE_REQUIRED");
}

#[tokio::test]
async fn create_empty_date() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"   ","fuel_amount":30.0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_DATE_REQUIRED");
}

#[tokio::test]
async fn create_missing_fuel_amount() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_FUEL_AMOUNT_REQUIRED");
}

#[tokio::test]
async fn create_zero_fuel_amount() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_FUEL_AMOUNT");
}

#[tokio::test]
async fn create_negative_fuel_amount() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":-5.0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_FUEL_AMOUNT");
}

#[tokio::test]
async fn create_negative_cost() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"odometer":1000,"cost":-5.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_COST");
}

#[tokio::test]
async fn create_missing_odometer() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_ODOMETER_REQUIRED");
}

#[tokio::test]
async fn create_missing_cost() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"odometer":1000}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_COST_REQUIRED");
}

#[tokio::test]
async fn create_trims_date() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"  2026-04-09  ","fuel_amount":30.0,"odometer":1000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["date"], "2026-04-09");
}

#[tokio::test]
async fn create_odometer_lower_than_previous() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // First fill-up at 10000
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    // Second fill-up at 5000 should fail
    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-02-01","fuel_amount":30.0,"odometer":5000,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_ODOMETER");
}

#[tokio::test]
async fn create_null_odometer_rejected() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-02-01","fuel_amount":25.0,"odometer":null,"cost":40.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_ODOMETER_REQUIRED");
}

#[tokio::test]
async fn create_odometer_equal_to_previous_accepted() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    // Same odometer value should succeed (split fueling)
    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":15.0,"odometer":10000,"cost":25.0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
}

// ── Update (PUT) ─────────────────────────────────────────

#[tokio::test]
async fn update_changes_fields() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();
    let original_updated = created["updated_at"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            Some(r#"{"date":"2026-04-02","fuel_amount":35.0,"odometer":10500,"cost":60.0,"station":"BP"}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["date"], "2026-04-02");
    assert_eq!(json["fuel_amount"], 35.0);
    assert_eq!(json["cost"], 60.0);
    assert_eq!(json["station"], "BP");
    // fuel_unit and currency auto-populated from settings on update
    assert_eq!(json["fuel_unit"], "l");
    assert_eq!(json["currency"], "EUR");
    assert_ne!(json["updated_at"].as_str().unwrap(), original_updated);
}

#[tokio::test]
async fn update_nonexistent_fillup() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{vid}/fillups/999"),
            Some(r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_NOT_FOUND");
}

#[tokio::test]
async fn update_for_nonexistent_vehicle() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request(
            "PUT",
            "/api/vehicles/999/fillups/1",
            Some(r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

#[tokio::test]
async fn update_odometer_excludes_self() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Create fill-up at 10000
    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();

    // Updating the same fill-up to a lower odometer should succeed
    // (it excludes itself from the max check)
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            Some(r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":9000,"cost":50.0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = common::body_json(resp).await;
    assert_eq!(json["odometer"], 9000.0);
}

#[tokio::test]
async fn update_odometer_below_other_fillup_rejected() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    // Create two fill-ups
    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-01-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-02-01","fuel_amount":30.0,"odometer":11000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid2 = created["id"].as_i64().unwrap();

    // Updating the second fill-up below the first's odometer should fail
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{vid}/fillups/{fid2}"),
            Some(r#"{"date":"2026-02-01","fuel_amount":30.0,"odometer":9000,"cost":50.0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_ODOMETER");
}

#[tokio::test]
async fn update_negative_cost_rejected() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "PUT",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            Some(r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":-5.0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_INVALID_COST");
}

#[tokio::test]
async fn create_zero_cost_accepted() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-09","fuel_amount":30.0,"odometer":5000,"cost":0}"#,
    )
    .await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = common::body_json(resp).await;
    assert_eq!(json["cost"], 0.0);
}

// ── Delete ───────────────────────────────────────────────

#[tokio::test]
async fn delete_fillup_returns_204() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
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
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_fillup() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = app
        .clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{vid}/fillups/999"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "FILLUP_NOT_FOUND");
}

#[tokio::test]
async fn delete_fillup_for_nonexistent_vehicle() {
    let app = common::test_app().await;
    let resp = app
        .oneshot(common::json_request(
            "DELETE",
            "/api/vehicles/999/fillups/1",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_NOT_FOUND");
}

// ── Vehicle delete guard ─────────────────────────────────

#[tokio::test]
async fn vehicle_delete_blocked_when_fillups_exist() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;

    // Attempt to delete vehicle should fail with 409
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{vid}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "VEHICLE_HAS_FILLUPS");
}

#[tokio::test]
async fn vehicle_delete_succeeds_after_fillups_removed() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let create_resp = create_fillup(
        &mut app,
        vid,
        r#"{"date":"2026-04-01","fuel_amount":30.0,"odometer":10000,"cost":50.0}"#,
    )
    .await;
    let created = common::body_json(create_resp).await;
    let fid = created["id"].as_i64().unwrap();

    // Delete the fill-up first
    app.clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{vid}/fillups/{fid}"),
            None,
        ))
        .await
        .unwrap();

    // Now vehicle delete should succeed
    let resp = app
        .clone()
        .oneshot(common::json_request(
            "DELETE",
            &format!("/api/vehicles/{vid}"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

// ── Bad request ──────────────────────────────────────────

#[tokio::test]
async fn malformed_json_returns_400() {
    let mut app = common::test_app().await;
    let vid = setup_vehicle(&mut app).await;

    let resp = create_fillup(&mut app, vid, r#"{"date": invalid}"#).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = common::body_json(resp).await;
    assert_eq!(json["code"], "INVALID_REQUEST_BODY");
}
