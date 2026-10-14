mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::{Router, http::StatusCode};
use chrono::{Days, NaiveDate};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::{Layer, layer::SubscriberExt};

use gazel::state::AppState;

#[derive(Clone)]
struct HistoryReads(Arc<AtomicUsize>);

impl<S: tracing::Subscriber> Layer<S> for HistoryReads {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
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

async fn setup() -> (Router, SqlitePool) {
    let pool = common::test_pool().await;
    let app = gazel::server::router(AppState::new(pool.clone(), None));
    (app, pool)
}

async fn request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
    status: StatusCode,
) -> Value {
    let text = body.map(|v| v.to_string());
    let response = app
        .clone()
        .oneshot(common::json_request(method, uri, text.as_deref()))
        .await
        .unwrap();
    assert_eq!(response.status(), status, "{method} {uri}");
    if status == StatusCode::NO_CONTENT {
        Value::Null
    } else {
        common::body_json(response).await
    }
}

// Each fresh successful request, including requests after writes, must execute
// the single shared history helper exactly once. No production counter/state.
async fn stats(app: &Router, query: &str) -> Value {
    let reads = Arc::new(AtomicUsize::new(0));
    let subscriber = tracing_subscriber::registry().with(HistoryReads(reads.clone()));
    let response = app
        .clone()
        .oneshot(common::json_request(
            "GET",
            &format!("/api/stats{query}"),
            None,
        ))
        .with_subscriber(subscriber)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    common::body_json(response).await
}

async fn vehicle(app: &Router) -> i64 {
    request(
        app,
        "POST",
        "/api/vehicles",
        Some(json!({"name":"Car"})),
        StatusCode::CREATED,
    )
    .await["id"]
        .as_i64()
        .unwrap()
}

async fn fill(app: &Router, id: i64, date: &str, odo: f64, fuel: f64, cost: f64) -> i64 {
    request(
        app,
        "POST",
        &format!("/api/vehicles/{id}/fillups"),
        Some(
            json!({"date":date,"odometer":odo,"fuel_amount":fuel,"cost":cost,"is_full_tank":true}),
        ),
        StatusCode::CREATED,
    )
    .await["id"]
        .as_i64()
        .unwrap()
}

fn near(value: &Value, expected: f64) {
    let actual = value.as_f64().unwrap();
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[tokio::test]
async fn empty_fleet_and_empty_selected_vehicle_have_explicit_nulls_and_units() {
    let (app, _) = setup().await;
    let empty = stats(&app, "").await;
    assert_eq!(empty["vehicles"], json!([]));
    assert!(empty["selected"].is_null());
    assert_eq!(empty["fleet"]["fill_up_count"], 0);
    assert!(empty["fleet"]["average_efficiency"].is_null());
    assert!(empty["fleet"]["cost_per_distance"].is_null());
    assert!(empty["fleet"]["cost_per_volume"].is_null());
    assert_eq!(empty["fleet_efficiency"]["points"], json!([]));
    assert_eq!(
        empty["units"],
        json!({"distance_unit":"km","volume_unit":"l","currency":"EUR"})
    );
    let id = vehicle(&app).await;
    let empty = stats(&app, &format!("?vehicle_id={id}")).await;
    assert_eq!(empty["vehicles"].as_array().unwrap().len(), 1);
    let s = &empty["selected"]["summary"];
    for field in [
        "total_distance",
        "total_fuel",
        "total_cost",
        "fill_up_count",
    ] {
        near(&s[field], 0.0);
    }
    assert!(s["average_efficiency"].is_null());
    assert!(s["average_cost_per_distance"].is_null());
    assert_eq!(empty["selected"]["charts"]["fuel_price"]["source_count"], 0);
    assert_eq!(
        empty["selected"]["charts"]["cost"]["monthly"]["points"],
        json!([])
    );
}

#[tokio::test]
async fn selection_errors_are_safe_uncached_and_unknown_selection_precedes_history() {
    let (app, pool) = setup().await;
    for query in [
        "vehicle_id=",
        "vehicle_id=abc",
        "vehicle_id=0",
        "vehicle_id=-1",
        "vehicle_id=9223372036854775808",
        "vehicle_id=1.5",
        "vehicle_id=1&vehicle_id=1",
        "vehicle_id=%FF",
        "vehicle_id=%2B",
        "vehicle_id=1%20",
    ] {
        let response = app
            .clone()
            .oneshot(common::json_request(
                "GET",
                &format!("/api/stats?{query}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{query}");
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(
            common::body_json(response).await,
            json!({"code":"STATS_INVALID_VEHICLE_ID","message":"Vehicle ID must be a positive integer."})
        );
    }
    sqlx::query("DROP TABLE fillups")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(common::json_request(
            "GET",
            "/api/stats?vehicle_id=999",
            None,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert_eq!(
        common::body_json(response).await["code"],
        "VEHICLE_NOT_FOUND"
    );
}

#[tokio::test]
async fn summaries_weight_full_rounded_segment_population_and_fleet_ratios_use_totals() {
    let (app, _) = setup().await;
    let a = vehicle(&app).await;
    let b = vehicle(&app).await;
    fill(&app, a, "2025-01-01", 1000.0, 40.0, 80.0).await;
    fill(&app, a, "2025-02-01", 1500.0, 40.0, 80.0).await;
    fill(&app, a, "2025-03-01", 2000.0, 50.0, 100.0).await;
    fill(&app, b, "2025-01-01", 1000.0, 20.0, 50.0).await;
    fill(&app, b, "2025-02-01", 1600.0, 37.0, 70.0).await;
    let data = stats(&app, &format!("?vehicle_id={a}&from=banana&to=1900-01-01")).await;
    let s = &data["selected"]["summary"];
    near(&s["total_distance"], 1000.0);
    near(&s["total_fuel"], 130.0);
    near(&s["total_cost"], 260.0);
    near(&s["average_efficiency"], 11.25);
    near(&s["average_cost_per_distance"], 0.18);
    near(
        &data["fleet"]["average_efficiency"],
        (12.5 + 10.0 + 16.22) / 3.0,
    );
    near(&data["fleet"]["cost_per_distance"], 380.0 / 1600.0);
    near(&data["fleet"]["cost_per_volume"], 380.0 / 187.0);
    let points = data["fleet_efficiency"]["points"].as_array().unwrap();
    near(&points[0]["value"], 8.0);
    near(&points[1]["value"], 100.0 / 16.22);
    near(&points[2]["value"], 10.0);
    assert_eq!(data["vehicles"][0], *s);
    assert!(data["vehicles"][1].get("charts").is_none());
    let fleet_only = stats(&app, "").await;
    assert!(fleet_only["selected"].is_null());
    assert_eq!(fleet_only["fleet"], data["fleet"]);
    assert_eq!(
        stats(&app, "?%76ehicle_id=%31").await["selected"]["vehicle_id"],
        a
    );
}

#[tokio::test]
async fn displayed_efficiency_means_rounded_samples_without_rounding_the_mean() {
    let (app, _) = setup().await;
    let id = vehicle(&app).await;
    fill(&app, id, "2025-01-01", 1000.0, 100.0, 80.0).await;
    fill(&app, id, "2025-01-01", 2000.4, 100.0, 80.0).await;
    fill(&app, id, "2025-01-01", 3001.8, 100.0, 80.0).await;
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    near(&data["selected"]["summary"]["average_efficiency"], 10.005);
    near(&data["fleet"]["average_efficiency"], 10.005);
    let points = &data["fleet_efficiency"]["points"];
    near(&points[0]["value"], 10.0);
    near(&points[1]["value"], 100.0 / 10.01);
    // Same-date history still creates both ID-ordered intervals; regression
    // has no time domain, even though all three real price dots remain.
    assert_eq!(data["selected"]["charts"]["fuel_price"]["source_count"], 3);
    assert!(data["selected"]["charts"]["fuel_price"]["trend"].is_null());
}

#[tokio::test]
async fn reduced_price_trend_uses_the_full_population_not_retained_extrema() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    for i in 0..260_u32 {
        let date = start
            .checked_add_days(Days::new(u64::from(i)))
            .unwrap()
            .to_string();
        let cost = if i == 17 { 1000.0 } else { 1.0 };
        sqlx::query("INSERT INTO fillups (vehicle_id,date,odometer,fuel_amount,fuel_unit,cost,is_full_tank) VALUES (?,?,NULL,1,'l',?,0)")
            .bind(id).bind(date).bind(cost).execute(&pool).await.unwrap();
    }
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    let prices = &data["selected"]["charts"]["fuel_price"];
    assert_eq!(prices["source_count"], 260);
    assert_eq!(prices["reduced"], true);
    let n = 260.0;
    let mean_x = 129.5;
    let sum_squared_x = n * (n * n - 1.0) / 12.0;
    let slope = 999.0 * (17.0 - mean_x) / sum_squared_x;
    let intercept = 1.0 + 999.0 / n - slope * mean_x;
    near(&prices["trend"][0]["value"], intercept);
    near(&prices["trend"][1]["value"], intercept + slope * 259.0);
    assert!(
        prices["points"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["value"] == 1000.0)
    );
    for point in prices["points"].as_array().unwrap() {
        assert_eq!(point.as_object().unwrap().len(), 2);
    }
}

#[tokio::test]
async fn partial_missed_null_zero_odometer_and_free_prices_use_separate_populations() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    // Direct writes cover legacy zero/null odometers and flags without changing
    // the API's unrelated write validation.
    for (date, odo, fuel, cost, full, missed) in [
        ("2025-01-01", Some(0.0), 3.0, 0.0, false, true),
        ("2025-01-02", Some(1000.0), 40.0, 80.0, true, false),
        ("2025-01-15", None, 20.0, 40.0, false, true),
        ("2025-02-01", Some(1500.0), 30.0, 60.0, true, false),
        ("2025-03-01", Some(2000.0), 40.0, 80.0, true, false),
    ] {
        sqlx::query("INSERT INTO fillups (vehicle_id,date,odometer,fuel_amount,fuel_unit,cost,is_full_tank,is_missed) VALUES (?,?,?,?, 'l',?,?,?)")
            .bind(id).bind(date).bind(odo).bind(fuel).bind(cost).bind(full).bind(missed).execute(&pool).await.unwrap();
    }
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    let s = &data["selected"]["summary"];
    assert_eq!(s["fill_up_count"], 4);
    near(&s["total_fuel"], 130.0);
    near(&s["total_cost"], 260.0);
    near(&s["average_efficiency"], 12.5);
    assert_eq!(data["fleet_efficiency"]["source_count"], 1);
    assert_eq!(data["selected"]["cost_per_distance"]["source_count"], 2);
    near(
        &data["selected"]["cost_per_distance"]["points"][0]["value"],
        0.2,
    );
    let prices = &data["selected"]["charts"]["fuel_price"];
    assert_eq!(prices["source_count"], 5);
    assert_eq!(
        prices["points"][0],
        json!({"date":"2025-01-01","value":0.0})
    );
    assert_eq!(
        data["selected"]["charts"]["cost"]["monthly"]["points"],
        json!([{"period":"2025-02","value":100.0},{"period":"2025-03","value":80.0}])
    );
}

#[tokio::test]
async fn first_partial_price_and_mixed_units_preserve_unrounded_prices_and_rounded_efficiencies() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    sqlx::query("INSERT INTO fillups (vehicle_id,date,odometer,fuel_amount,fuel_unit,cost,is_full_tank,is_missed) VALUES (?, '2025-01-01',NULL,3,'l',10,0,1)").bind(id).execute(&pool).await.unwrap();
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    near(
        &data["selected"]["charts"]["fuel_price"]["points"][0]["value"],
        10.0 / 3.0,
    );
    assert!(data["selected"]["charts"]["fuel_price"]["trend"].is_null());
    fill(&app, id, "2025-02-01", 1000.0, 40.0, 80.0).await;
    request(
        &app,
        "PUT",
        "/api/settings",
        Some(json!({"distance_unit":"mi","volume_unit":"gal","currency":"USD"})),
        StatusCode::OK,
    )
    .await;
    fill(&app, id, "2025-03-01", 1500.0, 10.0, 40.0).await;
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    near(&data["selected"]["summary"]["total_distance"], 310.69);
    near(
        &data["selected"]["summary"]["total_fuel"],
        gazel::stats::round2(43.0 / 3.785411784 + 10.0),
    );
    near(&data["selected"]["summary"]["average_efficiency"], 31.07);
    near(&data["fleet_efficiency"]["points"][0]["value"], 31.07);
    near(
        &data["selected"]["charts"]["fuel_price"]["points"][0]["value"],
        10.0 * 3.785411784 / 3.0,
    );
    near(
        &data["selected"]["charts"]["fuel_price"]["points"][2]["value"],
        4.0,
    );
    assert_eq!(
        data["units"],
        json!({"distance_unit":"mi","volume_unit":"gal","currency":"USD"})
    );
}

#[tokio::test]
async fn large_history_is_bounded_without_reducing_all_time_summaries_or_trend() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    let start = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
    let mut tx = pool.begin().await.unwrap();
    for i in 0..300_u32 {
        let date = start
            .checked_add_days(Days::new(u64::from(i) * 31))
            .unwrap()
            .to_string();
        sqlx::query("INSERT INTO fillups (vehicle_id,date,odometer,fuel_amount,fuel_unit,cost,is_full_tank) VALUES (?,?,?,10,'l',?,1)")
            .bind(id).bind(date).bind(1000.0 + f64::from(i) * 100.0).bind(10.0 + f64::from(i)).execute(&mut *tx).await.unwrap();
    }
    tx.commit().await.unwrap();
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    let selected = &data["selected"];
    assert_eq!(selected["summary"]["fill_up_count"], 300);
    near(&selected["summary"]["total_distance"], 29900.0);
    near(&selected["summary"]["total_cost"], 47850.0);
    for series in [&data["fleet_efficiency"], &selected["cost_per_distance"]] {
        assert_eq!(series["source_count"], 299);
        assert_eq!(series["reduced"], true);
        assert!(series["points"].as_array().unwrap().len() <= 60);
        assert_eq!(series["points"][0]["position"], 0.0);
        assert_eq!(
            series["points"].as_array().unwrap().last().unwrap()["position"],
            1.0
        );
    }
    for chart in ["cost", "distance"] {
        for mode in ["monthly", "yearly"] {
            let series = &selected["charts"][chart][mode];
            assert_eq!(series["points"].as_array().unwrap().len(), 12);
            assert_eq!(series["omitted_older_periods"], true);
            assert!(series["source_count"].as_u64().unwrap() > 12);
            assert_ne!(
                series["source_range"]["start"],
                series["displayed_range"]["start"]
            );
        }
    }
    let prices = &selected["charts"]["fuel_price"];
    assert_eq!(prices["source_count"], 300);
    assert_eq!(prices["reduced"], true);
    assert!(prices["points"].as_array().unwrap().len() <= 240);
    assert_eq!(
        prices["points"][0],
        json!({"date":"2000-01-01","value":1.0})
    );
    near(
        &prices["points"].as_array().unwrap().last().unwrap()["value"],
        30.9,
    );
    near(&prices["trend"][0]["value"], 1.0);
    near(&prices["trend"][1]["value"], 30.9);
    assert_eq!(prices["source_range"], prices["displayed_range"]);
    let text = data.to_string();
    for forbidden in [
        "fillup_id",
        "odometer",
        "fuel_amount",
        "fuel_unit",
        "start_date",
        "end_date",
        "is_valid",
        "station",
        "notes",
        "segments",
        "fillups",
    ] {
        assert!(
            !text.contains(&format!("\"{forbidden}\"")),
            "raw field {forbidden}"
        );
    }
    assert_eq!(stats(&app, &format!("?vehicle_id={id}")).await, data);
}

fn assert_performance_history_envelope(data: &Value) -> usize {
    assert_eq!(data["vehicles"].as_array().unwrap().len(), 1);
    for summary in [&data["fleet"], &data["vehicles"][0]] {
        assert_eq!(summary["fill_up_count"], 100_000);
        near(&summary["total_distance"], 999_990.0);
        near(&summary["total_fuel"], 1_000_000.0);
        // 14,285 complete cost cycles, followed by costs 10 through 14.
        near(&summary["total_cost"], 1_299_995.0);
        near(&summary["average_efficiency"], 1.0);
    }
    let series = &data["fleet_efficiency"];
    assert_eq!(series["source_count"], 99_999);
    assert_eq!(series["reduced"], true);
    let points = series["points"].as_array().unwrap();
    assert!(!points.is_empty() && points.len() <= 60);
    assert_eq!(points[0]["position"], 0.0);
    assert_eq!(points.last().unwrap()["position"], 1.0);
    for point in points {
        near(&point["value"], 100.0);
    }
    let text = data.to_string();
    for forbidden in [
        "fillup_id",
        "odometer",
        "fuel_amount",
        "fuel_unit",
        "start_date",
        "end_date",
        "is_valid",
        "station",
        "notes",
        "segments",
        "fillups",
    ] {
        assert!(
            !text.contains(&format!("\"{forbidden}\"")),
            "raw field {forbidden}"
        );
    }
    assert!(text.len() < 64 * 1024, "response has {} bytes", text.len());
    text.len()
}

#[tokio::test]
async fn hundred_thousand_fillups_keep_selected_and_fleet_responses_bounded_and_exact() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    // One batch, including same-day identities, spanning more than 27 years.
    let inserted = sqlx::query(
        "WITH RECURSIVE history(i) AS (
            SELECT 0 UNION ALL SELECT i + 1 FROM history WHERE i < 99999
         )
         INSERT INTO fillups
            (vehicle_id,date,odometer,fuel_amount,fuel_unit,cost,is_full_tank,is_missed)
         SELECT ?,date('2000-01-01','+' || (i / 10) || ' days'),
                1000 + 10 * i,10,'l',10 + (i % 7),1,0 FROM history",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(inserted.rows_affected(), 100_000);

    // Each separate current request asserts exactly one history_fetch event.
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    let selected_bytes = assert_performance_history_envelope(&data);
    let selected = &data["selected"];
    assert_eq!(selected["vehicle_id"], id);
    assert_eq!(selected["summary"], data["vehicles"][0]);
    for chart in ["cost", "distance"] {
        for mode in ["monthly", "yearly"] {
            let series = &selected["charts"][chart][mode];
            assert_eq!(series["points"].as_array().unwrap().len(), 12);
            assert_eq!(series["omitted_older_periods"], true);
            assert!(series["source_count"].as_u64().unwrap() > 12);
            assert_ne!(
                series["source_range"]["start"],
                series["displayed_range"]["start"]
            );
        }
    }
    let prices = &selected["charts"]["fuel_price"];
    assert_eq!(prices["source_count"], 100_000);
    assert_eq!(prices["reduced"], true);
    let points = prices["points"].as_array().unwrap();
    assert!(!points.is_empty() && points.len() <= 240);
    assert_eq!(points[0], json!({"date":"2000-01-01","value":1.0}));
    assert_eq!(
        points.last().unwrap(),
        &json!({"date":"2027-05-18","value":1.4})
    );
    assert_eq!(prices["source_range"], prices["displayed_range"]);
    for point in points {
        assert!(
            (10..=16).any(
                |cost| (point["value"].as_f64().unwrap() - f64::from(cost) / 10.0).abs() < 1e-9
            )
        );
    }
    assert!(points.iter().any(|p| p["value"] == 1.0));
    assert!(points.iter().any(|p| p["value"] == 1.6));
    let costs = &selected["cost_per_distance"];
    assert_eq!(costs["source_count"], 99_999);
    assert_eq!(costs["reduced"], true);
    let cost_points = costs["points"].as_array().unwrap();
    assert!(!cost_points.is_empty() && cost_points.len() <= 60);
    assert_eq!(cost_points[0]["position"], 0.0);
    assert_eq!(cost_points.last().unwrap()["position"], 1.0);

    let fleet = stats(&app, "").await;
    let fleet_bytes = assert_performance_history_envelope(&fleet);
    assert!(fleet["selected"].is_null());
    for field in ["units", "fleet", "vehicles", "fleet_efficiency"] {
        assert_eq!(fleet[field], data[field]);
    }
    println!(
        "100000 fillups: selected={selected_bytes} bytes, fleet={fleet_bytes} bytes; \
         prices={}, fleet sparkline={}, cost sparkline={}, period series=12 each",
        points.len(),
        data["fleet_efficiency"]["points"].as_array().unwrap().len(),
        cost_points.len()
    );
}

#[tokio::test]
async fn short_period_history_zero_fills_gaps_without_extending_to_today() {
    let (app, _) = setup().await;
    let id = vehicle(&app).await;
    fill(&app, id, "2020-01-01", 1000.0, 40.0, 80.0).await;
    fill(&app, id, "2020-02-01", 1500.0, 40.0, 80.0).await;
    fill(&app, id, "2022-04-01", 2000.0, 40.0, 80.0).await;
    let data = stats(&app, &format!("?vehicle_id={id}")).await;
    let cost = &data["selected"]["charts"]["cost"];
    assert_eq!(
        cost["yearly"]["points"],
        json!([{"period":"2020","value":80.0},{"period":"2021","value":0.0},{"period":"2022","value":80.0}])
    );
    assert_eq!(cost["yearly"]["omitted_older_periods"], false);
    assert_eq!(
        cost["monthly"]["displayed_range"],
        json!({"start":"2021-05","end":"2022-04"})
    );
    assert_eq!(cost["monthly"]["points"][0]["value"], 0.0);
    assert_eq!(cost["monthly"]["points"].as_array().unwrap().len(), 12);
}

#[tokio::test]
async fn fresh_requests_reflect_fillup_vehicle_settings_and_external_writes() {
    let (app, pool) = setup().await;
    let id = vehicle(&app).await;
    assert_eq!(stats(&app, "").await["fleet"]["fill_up_count"], 0);
    let fid = fill(&app, id, "2025-01-01", 1000.0, 40.0, 80.0).await;
    near(&stats(&app, "").await["fleet"]["total_cost"], 80.0);
    request(&app, "PUT", &format!("/api/vehicles/{id}/fillups/{fid}"), Some(json!({"date":"2025-01-01","odometer":1000,"fuel_amount":50,"cost":90,"is_full_tank":true})), StatusCode::OK).await;
    near(&stats(&app, "").await["fleet"]["total_cost"], 90.0);
    sqlx::query("UPDATE fillups SET cost = 123 WHERE id = ?")
        .bind(fid)
        .execute(&pool)
        .await
        .unwrap();
    near(&stats(&app, "").await["fleet"]["total_cost"], 123.0);
    request(
        &app,
        "PUT",
        "/api/settings",
        Some(json!({"distance_unit":"mi","volume_unit":"gal","currency":"USD"})),
        StatusCode::OK,
    )
    .await;
    let data = stats(&app, "").await;
    assert_eq!(data["units"]["currency"], "USD");
    near(&data["fleet"]["total_fuel"], 13.21);
    request(
        &app,
        "DELETE",
        &format!("/api/vehicles/{id}/fillups/{fid}"),
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_eq!(stats(&app, "").await["fleet"]["fill_up_count"], 0);
    let other = vehicle(&app).await;
    assert_eq!(
        stats(&app, "").await["vehicles"].as_array().unwrap().len(),
        2
    );
    request(
        &app,
        "PATCH",
        &format!("/api/vehicles/{other}"),
        Some(json!({"name":"Renamed"})),
        StatusCode::OK,
    )
    .await;
    assert_eq!(
        stats(&app, &format!("?vehicle_id={other}")).await["selected"]["vehicle_id"],
        other
    );
    request(
        &app,
        "DELETE",
        &format!("/api/vehicles/{other}"),
        None,
        StatusCode::NO_CONTENT,
    )
    .await;
    assert_eq!(
        stats(&app, "").await["vehicles"].as_array().unwrap().len(),
        1
    );
}

#[tokio::test]
async fn fresh_requests_reflect_import_merge_and_replace_with_reused_ids() {
    let (app, _) = setup().await;
    let id = vehicle(&app).await;
    fill(&app, id, "2025-01-01", 1000.0, 40.0, 80.0).await;
    let exported = request(&app, "GET", "/api/export", None, StatusCode::OK).await;
    near(&stats(&app, "").await["fleet"]["total_cost"], 80.0);
    let mut changed = exported.clone();
    changed["vehicles"][0]["fillups"][0]["cost"] = json!(222.0);
    request(
        &app,
        "POST",
        "/api/import?mode=replace",
        Some(changed),
        StatusCode::OK,
    )
    .await;
    near(
        &stats(&app, &format!("?vehicle_id={id}")).await["fleet"]["total_cost"],
        222.0,
    );
    let mut merged = exported;
    merged["vehicles"][0]["name"] = json!("Imported");
    request(
        &app,
        "POST",
        "/api/import?mode=merge",
        Some(merged),
        StatusCode::OK,
    )
    .await;
    let data = stats(&app, "").await;
    assert_eq!(data["vehicles"].as_array().unwrap().len(), 2);
    near(&data["fleet"]["total_cost"], 302.0);
}

#[tokio::test]
async fn database_and_calculation_failures_return_safe_uncached_errors() {
    for failure in ["table", "date", "infinite", "overflow"] {
        let (app, pool) = setup().await;
        let id = vehicle(&app).await;
        let fid = fill(&app, id, "2025-01-01", 1000.0, 40.0, 80.0).await;
        stats(&app, &format!("?vehicle_id={id}")).await;
        match failure {
            "table" => {
                sqlx::query("DROP TABLE fillups")
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            "date" => {
                sqlx::query("UPDATE fillups SET date = 'broken'")
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            "infinite" => {
                sqlx::query("UPDATE fillups SET cost = ? WHERE id = ?")
                    .bind(f64::INFINITY)
                    .bind(fid)
                    .execute(&pool)
                    .await
                    .unwrap();
            }
            _ => {
                sqlx::query("UPDATE fillups SET fuel_amount = 1e308, fuel_unit = 'gal'")
                    .execute(&pool)
                    .await
                    .unwrap();
            }
        }
        let response = app
            .clone()
            .oneshot(common::json_request(
                "GET",
                &format!("/api/stats?vehicle_id={id}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "{failure}"
        );
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(
            common::body_json(response).await,
            json!({"code":"INTERNAL_ERROR","message":"An unexpected error occurred."})
        );
    }
}

#[tokio::test]
async fn replaced_routes_and_dashboard_alias_never_serve_statistics_or_redirect() {
    let app = common::test_app().await;
    let id = vehicle(&app).await;
    for uri in [
        format!("/api/vehicles/{id}/stats"),
        format!("/api/vehicles/{id}/stats/history"),
        "/api/dashboard".into(),
        "/api/stats/prices".into(),
    ] {
        let response = app
            .clone()
            .oneshot(common::json_request("GET", &uri, None))
            .await
            .unwrap();
        assert!(!response.status().is_redirection());
        assert!(response.headers().get("location").is_none());
        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        if let Ok(json) = serde_json::from_slice::<Value>(&bytes) {
            assert!(json.get("fleet").is_none());
            assert!(json.get("total_distance").is_none());
            assert!(!json.is_array());
        }
    }
}
