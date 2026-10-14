//! Page-bounded segment context: SQLite aggregates intervals; raw history never
//! leaves the database and only returned full-tank endings need lookups.

use std::collections::HashMap;

use sqlx::{QueryBuilder, Sqlite, SqliteConnection};

use super::{FillupEfficiency, FillupPageEfficiency, FillupRow};
use crate::api::error::{ApiError, db_error};
use crate::stats;

#[derive(sqlx::FromRow)]
struct Interval {
    end_id: i64,
    distance: f64,
    fuel: Option<f64>,
    has_missed: i32,
}

impl Interval {
    fn efficiency(&self, distance_unit: &str, volume_unit: &str) -> Option<f64> {
        let fuel = self.fuel?;
        if self.distance <= 0.0
            || !self.distance.is_finite()
            || fuel <= 0.0
            || !fuel.is_finite()
            || self.has_missed != 0
        {
            return None;
        }
        // Preserve displayed_segment's operation and rounding order, without
        // rounding distance/fuel first or constructing a complete Segment.
        let ratio = stats::round2(
            (self.distance / fuel) * stats::convert_distance(1.0, distance_unit)
                / stats::convert_volume(1.0, volume_unit),
        );
        ratio.is_finite().then_some(ratio)
    }
}

pub(super) async fn for_page(
    connection: &mut SqliteConnection,
    vehicle_id: i64,
    rows: &[FillupRow],
) -> Result<FillupPageEfficiency, ApiError> {
    let (distance_unit, volume_unit): (String, String) =
        sqlx::query_as("SELECT distance_unit, volume_unit FROM settings WHERE id = 1")
            .fetch_one(&mut *connection)
            .await
            .map_err(db_error)?;
    let endings: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.is_full_tank != 0 && row.odometer.is_some_and(|odo| odo > 0.0 && odo.is_finite())
        })
        .collect();
    let mut efficiencies = HashMap::new();
    if !endings.is_empty() {
        // The VALUES population is bounded by the validated page limit (100).
        // Each boundary seek uses vehicle/date chronology, including same-day IDs.
        // A full tank always resets the boundary, even after an invalid interval.
        let mut query = QueryBuilder::<Sqlite>::new("WITH endings(id, date, odometer) AS (");
        query.push_values(endings, |mut values, row| {
            values
                .push_bind(row.id)
                .push_bind(&row.date)
                .push_bind(row.odometer);
        });
        query.push(
            "), boundaries AS (SELECT e.*, (SELECT b.id FROM fillups b \
             WHERE b.vehicle_id = ",
        );
        query.push_bind(vehicle_id);
        query.push(" AND b.is_full_tank != 0 AND b.odometer > 0 AND b.odometer <= ");
        query.push_bind(f64::MAX);
        query.push(
            " AND (b.date, b.id) < (e.date, e.id) \
             ORDER BY b.date DESC, b.id DESC LIMIT 1) AS start_id FROM endings e) \
             SELECT e.id AS end_id, e.odometer - b.odometer AS distance, \
             SUM(f.fuel_amount * CASE WHEN f.fuel_unit = 'gal' THEN ",
        );
        query.push_bind(stats::normalize_fuel_to_liters(1.0, "gal"));
        query.push(
            " ELSE 1.0 END) AS fuel, MAX(f.is_missed != 0) AS has_missed \
             FROM boundaries e JOIN fillups b ON b.id = e.start_id \
             JOIN fillups f ON f.vehicle_id = ",
        );
        query.push_bind(vehicle_id);
        // Starting fuel/missed is excluded, ending included. Null odometers
        // contribute fuel/missed but cannot be boundaries; zero/negative rows
        // are excluded entirely, matching the shared statistics engine.
        query.push(
            " AND (f.date, f.id) > (b.date, b.id) AND (f.date, f.id) <= (e.date, e.id) \
             AND (f.odometer IS NULL OR f.odometer > 0) \
             GROUP BY e.id, e.odometer, b.odometer",
        );
        for interval in query
            .build_query_as::<Interval>()
            .fetch_all(&mut *connection)
            .await
            .map_err(db_error)?
        {
            efficiencies.insert(
                interval.end_id,
                interval.efficiency(&distance_unit, &volume_unit),
            );
        }
    }
    Ok(FillupPageEfficiency {
        distance_unit,
        volume_unit,
        values: rows
            .iter()
            .map(|row| FillupEfficiency {
                fillup_id: row.id,
                efficiency: efficiencies.get(&row.id).copied().flatten(),
            })
            .collect(),
    })
}
