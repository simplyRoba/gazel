mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Router, http::StatusCode};
use chrono::{Days, NaiveDate};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{Layer, layer::SubscriberExt};

use gazel::{state::AppState, stats};

#[derive(Clone)]
struct HistoryReads(Arc<AtomicUsize>);

impl<S: tracing::Subscriber> Layer<S> for HistoryReads {
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        struct Visitor(bool);
        impl tracing::field::Visit for Visitor {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if field.name() == "event" && format!("{value:?}") == "\"history_fetch\"" {
                    self.0 = true;
                }
            }
        }
        let mut visitor = Visitor(false);
        event.record(&mut visitor);
        if event.metadata().target() == "gazel::stats" && visitor.0 {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
}

async fn setup() -> (Router, SqlitePool, i64) {
    let pool = common::test_pool().await;
    let app = gazel::server::router(AppState::new(pool.clone(), None));
    let id = vehicle(&pool).await;
    (app, pool, id)
}

async fn vehicle(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO vehicles (name, created_at, updated_at) VALUES ('Car', '', '') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// Direct writes deliberately exercise legacy nullable/invalid odometers and
// nonzero SQLite booleans without weakening application write validation.
async fn insert(pool: &SqlitePool, vehicle_id: i64, row: &mut stats::Fillup) {
    row.id = sqlx::query_scalar(
        "INSERT INTO fillups (vehicle_id, date, odometer, fuel_amount, fuel_unit, cost, \
         is_full_tank, is_missed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, '', '') RETURNING id",
    )
    .bind(vehicle_id).bind(&row.date).bind(row.odometer).bind(row.fuel_amount)
    .bind(&row.fuel_unit).bind(row.cost)
    .bind(if row.is_full_tank { -2 } else { 0 })
    .bind(if row.is_missed { 3 } else { 0 })
    .fetch_one(pool).await.unwrap();
}

fn row(odo: Option<f64>, fuel: f64, full: bool, missed: bool) -> stats::Fillup {
    stats::Fillup {
        id: 0,
        date: "2025-01-01".into(),
        odometer: odo,
        fuel_amount: fuel,
        fuel_unit: "l".into(),
        cost: Some(10.0),
        is_full_tank: full,
        is_missed: missed,
    }
}

async fn page(app: &Router, vehicle_id: i64, query: &str) -> Value {
    let reads = Arc::new(AtomicUsize::new(0));
    let subscriber = tracing_subscriber::registry().with(HistoryReads(reads.clone()));
    let response = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{vehicle_id}/fillups{query}"),
            None,
        ))
        .with_subscriber(subscriber)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        reads.load(Ordering::SeqCst),
        0,
        "list must not fetch statistics history"
    );
    let page = common::body_json(response).await;
    let items = page["items"].as_array().unwrap();
    let values = page["efficiency"]["values"].as_array().unwrap();
    assert_eq!(items.len(), values.len());
    for (item, value) in items.iter().zip(values) {
        assert_eq!(item["id"], value["fillup_id"]);
        assert_eq!(value.as_object().unwrap().len(), 2);
        assert!(item.get("efficiency").is_none());
    }
    page
}

fn efficiencies(page: &Value) -> Vec<Value> {
    page["efficiency"]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["efficiency"].clone())
        .collect()
}

#[tokio::test]
async fn small_page_aggregates_many_partials_from_start_outside_page_and_excludes_other_vehicle() {
    let (app, pool, id) = setup().await;
    insert(&pool, id, &mut row(Some(100.0), 999.0, true, true)).await;
    for i in 1..=40 {
        insert(
            &pool,
            id,
            &mut row(Some(100.0 + f64::from(i)), 1.0, false, false),
        )
        .await;
    }
    let other = vehicle(&pool).await;
    insert(&pool, other, &mut row(Some(100.0), 1000.0, true, true)).await;
    insert(&pool, id, &mut row(Some(600.0), 10.0, true, false)).await;
    let data = page(&app, id, "?limit=2").await;
    assert_eq!(efficiencies(&data), vec![json!(10.0), Value::Null]);
    let data = page(
        &app,
        id,
        &format!("?limit=2&cursor={}", data["next_cursor"].as_str().unwrap()),
    )
    .await;
    assert_eq!(efficiencies(&data), vec![Value::Null; 2]);
}

#[tokio::test]
async fn missed_partial_outside_page_invalidates_ending_but_missed_start_does_not() {
    let (app, pool, id) = setup().await;
    for mut fill in [
        row(Some(100.0), 50.0, true, false),
        row(None, 5.0, false, true),
        row(Some(200.0), 5.0, true, false),
        row(Some(300.0), 10.0, true, false),
        row(Some(400.0), 10.0, true, true),
        row(Some(500.0), 10.0, true, false),
    ] {
        insert(&pool, id, &mut fill).await;
    }
    let data = page(&app, id, "?limit=2").await;
    assert_eq!(efficiencies(&data), vec![json!(10.0), Value::Null]);
    let data = page(
        &app,
        id,
        &format!("?limit=2&cursor={}", data["next_cursor"].as_str().unwrap()),
    )
    .await;
    assert_eq!(efficiencies(&data), vec![json!(10.0), Value::Null]);
    let data = page(
        &app,
        id,
        &format!("?limit=2&cursor={}", data["next_cursor"].as_str().unwrap()),
    )
    .await;
    assert_eq!(efficiencies(&data), vec![Value::Null; 2]);
}

#[tokio::test]
async fn null_full_tanks_contribute_without_resetting_and_nonpositive_odometers_are_excluded() {
    let (app, pool, id) = setup().await;
    for mut fill in [
        row(Some(100.0), 999.0, true, false),
        row(None, 5.0, true, false),
        row(Some(0.0), 1000.0, true, true),
        row(Some(-1.0), 1000.0, true, true),
        row(Some(200.0), 5.0, true, false),
        row(None, 2.0, true, true),
        row(Some(300.0), 8.0, true, false),
    ] {
        insert(&pool, id, &mut fill).await;
    }
    let data = page(&app, id, "").await;
    assert_eq!(
        efficiencies(&data),
        vec![
            Value::Null,
            Value::Null,
            json!(10.0),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );
}

#[tokio::test]
async fn same_day_ids_and_equal_odometer_invalid_segment_still_reset_boundary() {
    let (app, pool, id) = setup().await;
    for mut fill in [
        row(Some(100.0), 90.0, true, false),
        row(Some(100.0), 900.0, true, true),
        row(Some(200.0), 10.0, true, false),
        row(Some(200.0), 7.0, false, false),
        row(None, 8.0, true, false),
    ] {
        insert(&pool, id, &mut fill).await;
    }
    let data = page(&app, id, "").await;
    assert_eq!(
        efficiencies(&data),
        vec![
            Value::Null,
            Value::Null,
            json!(10.0),
            Value::Null,
            Value::Null
        ]
    );
}

#[tokio::test]
async fn mixed_units_convert_raw_ratio_before_rounding_and_current_units_apply_on_empty_terminal_pages()
 {
    let (app, pool, id) = setup().await;
    sqlx::query("UPDATE settings SET distance_unit = 'mi', volume_unit = 'gal'")
        .execute(&pool)
        .await
        .unwrap();
    let empty = page(&app, id, "").await;
    assert_eq!(
        empty["efficiency"],
        json!({"distance_unit":"mi","volume_unit":"gal","values":[]})
    );
    let mut history = vec![
        row(Some(100.0), 99.0, true, false),
        row(None, 0.12345, false, false),
        row(Some(100.98765), 0.23456, true, false),
    ];
    history[1].fuel_unit = "gal".into();
    for fill in &mut history {
        insert(&pool, id, fill).await;
    }
    for (distance, volume) in [("mi", "gal"), ("km", "l"), ("mi", "l"), ("km", "gal")] {
        sqlx::query("UPDATE settings SET distance_unit = ?, volume_unit = ?")
            .bind(distance)
            .bind(volume)
            .execute(&pool)
            .await
            .unwrap();
        let data = page(&app, id, "?limit=1").await;
        let segment = stats::compute_segments(&history).remove(0);
        let displayed = stats::displayed_segment(&segment, distance, volume);
        assert_eq!(efficiencies(&data), vec![json!(displayed.efficiency)]);
        if volume == "gal" {
            assert_ne!(
                displayed.efficiency,
                stats::round2(displayed.distance / displayed.fuel)
            );
        }
        let terminal = page(
            &app,
            id,
            &format!("?cursor={}", data["next_cursor"].as_str().unwrap()),
        )
        .await;
        assert!(terminal["next_cursor"].is_null());
        assert_eq!(terminal["efficiency"]["distance_unit"], distance);
        assert_eq!(terminal["efficiency"]["volume_unit"], volume);
        assert_eq!(efficiencies(&terminal), vec![Value::Null; 2]);
    }
}

#[tokio::test]
async fn nonpositive_fuel_and_overflowing_ratio_have_no_badge() {
    let (app, pool, id) = setup().await;
    for mut fill in [
        row(Some(1.0), 10.0, true, false),
        row(Some(2.0), 0.0, true, false),
        row(Some(3.0), -10.0, true, false),
        row(Some(4.0), f64::MIN_POSITIVE, true, false),
        row(Some(5.0), 1.0, true, false),
    ] {
        insert(&pool, id, &mut fill).await;
    }
    assert_eq!(
        efficiencies(&page(&app, id, "").await),
        vec![
            json!(1.0),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );
}

#[tokio::test]
async fn nonfinite_legacy_numbers_cannot_form_boundaries_or_valid_ratios() {
    let (app, pool, id) = setup().await;
    for mut fill in [
        row(Some(1.0), 1.0, true, false),
        row(Some(2.0), 1.0, true, false),
        row(Some(f64::INFINITY), 1.0, true, false),
        row(Some(4.0), 1.0, true, false),
        row(Some(5.0), f64::INFINITY, true, false),
        row(Some(6.0), 1.0, true, false),
    ] {
        insert(&pool, id, &mut fill).await;
    }
    assert_eq!(
        efficiencies(&page(&app, id, "").await),
        vec![
            json!(1.0),
            Value::Null,
            json!(1.0),
            Value::Null,
            json!(1.0),
            Value::Null
        ]
    );
}

#[tokio::test]
async fn large_mixed_history_matches_full_calculation_oracle_for_every_paged_id() {
    let (app, pool, id) = setup().await;
    let other = vehicle(&pool).await;
    let mut history = Vec::new();
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    for i in 0..365_u32 {
        let odo = match i % 29 {
            1 => None,
            2 => Some(0.0),
            3 => Some(-1.0),
            _ => Some(100.0 + f64::from(i / 2) * 11.2345),
        };
        let mut fill = row(odo, 0.12345 + f64::from(i % 7), i % 3 != 0, i % 23 == 0);
        fill.date = start
            .checked_add_days(Days::new(u64::from(i / 4)))
            .unwrap()
            .to_string();
        if i % 2 == 0 {
            fill.fuel_unit = "gal".into();
        }
        insert(&pool, id, &mut fill).await;
        let mut foreign = fill.clone();
        foreign.fuel_amount = 9999.0;
        foreign.is_missed = true;
        insert(&pool, other, &mut foreign).await;
        history.push(fill);
    }
    for (distance, volume) in [("km", "l"), ("mi", "gal")] {
        sqlx::query("UPDATE settings SET distance_unit = ?, volume_unit = ?")
            .bind(distance)
            .bind(volume)
            .execute(&pool)
            .await
            .unwrap();
        let expected: HashMap<_, _> = stats::compute_segments(&history)
            .iter()
            .map(|segment| {
                let ratio = stats::displayed_segment(segment, distance, volume).efficiency;
                (
                    segment.end_fillup_id,
                    if segment.is_valid && ratio.is_finite() {
                        json!(ratio)
                    } else {
                        Value::Null
                    },
                )
            })
            .collect();
        let mut query = "?limit=100".to_string();
        let mut seen = Vec::new();
        loop {
            let data = page(&app, id, &query).await;
            assert!(data["items"].as_array().unwrap().len() <= 100);
            for value in data["efficiency"]["values"].as_array().unwrap() {
                let fid = value["fillup_id"].as_i64().unwrap();
                assert_eq!(
                    value["efficiency"],
                    *expected.get(&fid).unwrap_or(&Value::Null),
                    "fill-up {fid}"
                );
                seen.push(fid);
            }
            let Some(cursor) = data["next_cursor"].as_str() else {
                break;
            };
            query = format!("?limit=100&cursor={cursor}");
        }
        assert_eq!(seen, history.iter().rev().map(|f| f.id).collect::<Vec<_>>());
    }
}

#[tokio::test]
async fn one_hundred_full_endings_fit_in_one_batch() {
    let (app, pool, id) = setup().await;
    for i in 1..=101 {
        insert(
            &pool,
            id,
            &mut row(Some(f64::from(i) * 10.0), 1.0, true, false),
        )
        .await;
    }
    let data = page(&app, id, "?limit=100").await;
    assert_eq!(efficiencies(&data), vec![json!(10.0); 100]);
    let terminal = page(
        &app,
        id,
        &format!(
            "?limit=100&cursor={}",
            data["next_cursor"].as_str().unwrap()
        ),
    )
    .await;
    assert_eq!(efficiencies(&terminal), vec![Value::Null]);
    assert!(terminal["next_cursor"].is_null());
}

#[tokio::test]
async fn empty_continuation_reads_current_units_and_keeps_null_cursor() {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    let (app, pool, id) = setup().await;
    let mut first = row(Some(100.0), 10.0, true, false);
    insert(&pool, id, &mut first).await;
    sqlx::query("UPDATE settings SET distance_unit = 'mi', volume_unit = 'gal'")
        .execute(&pool)
        .await
        .unwrap();
    let cursor = URL_SAFE_NO_PAD.encode(json!({"date":first.date,"id":first.id}).to_string());
    let data = page(&app, id, &format!("?cursor={cursor}")).await;
    assert_eq!(data["items"], json!([]));
    assert!(data["next_cursor"].is_null());
    assert_eq!(
        data["efficiency"],
        json!({"distance_unit":"mi","volume_unit":"gal","values":[]})
    );
}

#[tokio::test]
async fn individual_get_post_put_keep_exact_field_keys() {
    // This also exercises the common app helper in this integration target.
    let app = common::test_app().await;
    let response = app
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/api/vehicles",
            Some("{\"name\":\"Car\"}"),
        ))
        .await
        .unwrap();
    let id = common::body_json(response).await["id"].as_i64().unwrap();
    let mut uri = format!("/api/vehicles/{id}/fillups");
    let payload =
        json!({"date":"2025-01-01","odometer":100.0,"fuel_amount":10.0,"cost":20.0}).to_string();
    let mut expected = vec![
        "id",
        "vehicle_id",
        "date",
        "odometer",
        "fuel_amount",
        "fuel_unit",
        "cost",
        "currency",
        "is_full_tank",
        "is_missed",
        "station",
        "notes",
        "created_at",
        "updated_at",
    ];
    expected.sort_unstable();
    for (method, status) in [
        ("POST", StatusCode::CREATED),
        ("GET", StatusCode::OK),
        ("PUT", StatusCode::OK),
    ] {
        let response = app
            .clone()
            .oneshot(common::json_request(
                method,
                &uri,
                if method == "GET" {
                    None
                } else {
                    Some(&payload)
                },
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        let body = common::body_json(response).await;
        let mut keys: Vec<_> = body
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, expected);
        if method == "POST" {
            uri = format!("{uri}/{}", body["id"].as_i64().unwrap());
        }
    }
}

#[tokio::test]
async fn database_failure_returns_safe_api_error() {
    let (app, pool, id) = setup().await;
    sqlx::query("DROP TABLE settings")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(common::json_request(
            "GET",
            &format!("/api/vehicles/{id}/fillups"),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = common::body_json(response).await;
    assert_eq!(body["code"], "INTERNAL_ERROR");
    assert!(!body.to_string().contains("settings"));
}
