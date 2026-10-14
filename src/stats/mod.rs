//! Pure statistics over a single vehicle's complete fill-up population.
//!
//! Stored distances are kilometers; fuel is normalized to liters. Calculations
//! sort a borrowed copy by date then ID and never mutate the caller's history.
//! SQL, date-range filtering, currency labels and HTTP contracts belong to adapters.

mod periods;
mod prices;
mod sampling;

pub use periods::{
    CalculationError, PERIOD_LIMIT, Period, PeriodPoint, PeriodRange, PeriodSeries,
    bounded_period_costs, bounded_period_distances, period_costs, period_distances,
};
pub use prices::{
    DateRange, FuelPrice, FuelPriceSeries, PriceTrendPoint, SampledFuelPrices, fuel_price_series,
    fuel_price_trend, fuel_prices, sample_fuel_prices,
};
pub use sampling::{
    PRICE_LIMIT, SPARKLINE_LIMIT, SparklinePoint, SparklineSeries, sample_sparkline,
};

const KM_PER_MI: f64 = 1.609_344;
const L_PER_GAL: f64 = 3.785_411_784;

/// Lightweight domain input, including rows ineligible for segment statistics.
#[derive(Clone, Debug)]
pub struct Fillup {
    pub id: i64,
    pub date: String,
    pub odometer: Option<f64>,
    pub fuel_amount: f64,
    pub fuel_unit: String,
    pub cost: Option<f64>,
    pub is_full_tank: bool,
    pub is_missed: bool,
}

/// Tank-to-tank interval in base units, retaining both boundary identities.
#[derive(Clone, Debug)]
pub struct Segment {
    pub start_fillup_id: i64,
    pub end_fillup_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub start_odometer: f64,
    pub end_odometer: f64,
    pub distance: f64,
    pub fuel: f64,
    pub cost: f64,
    pub efficiency: f64,
    pub cost_per_distance: f64,
    pub is_valid: bool,
}

/// Legacy summary: raw sums and mean raw valid-segment ratios before conversion.
#[derive(Clone, Debug, Default)]
pub struct AggregatedStats {
    pub total_distance: f64,
    pub total_fuel: f64,
    pub total_cost: f64,
    pub fill_up_count: i64,
    pub average_efficiency: Option<f64>,
    pub average_cost_per_distance: Option<f64>,
}

pub fn normalize_fuel_to_liters(amount: f64, fuel_unit: &str) -> f64 {
    if fuel_unit == "gal" {
        amount * L_PER_GAL
    } else {
        amount
    }
}

pub fn convert_distance(value: f64, to: &str) -> f64 {
    if to == "mi" { value / KM_PER_MI } else { value }
}

pub fn convert_volume(value: f64, to: &str) -> f64 {
    if to == "gal" {
        value / L_PER_GAL
    } else {
        value
    }
}

pub fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn ordered_rows(rows: &[Fillup]) -> Vec<&Fillup> {
    let mut ordered: Vec<_> = rows.iter().collect();
    ordered.sort_by(|a, b| a.date.cmp(&b.date).then(a.id.cmp(&b.id)));
    ordered
}

// Match the legacy SQL predicate: null odometers contribute fuel/cost/count,
// but zero/negative odometers do not. Null rows cannot form boundaries.
fn stats_eligible(row: &Fillup) -> bool {
    row.odometer.is_none_or(|odo| odo > 0.0)
}

/// Compute segments in date-then-ID order. Zero/negative-odometer rows are
/// excluded entirely; null-odometer rows contribute inside an interval but
/// cannot form its boundaries. The starting fill's fuel/cost/missed flag is excluded.
pub fn compute_segments(rows: &[Fillup]) -> Vec<Segment> {
    let rows: Vec<_> = ordered_rows(rows)
        .into_iter()
        .filter(|r| stats_eligible(r))
        .collect();
    let mut segments = Vec::new();
    let mut segment_start: Option<usize> = None;
    for (i, row) in rows.iter().enumerate() {
        let odo = row.odometer.unwrap_or(0.0);
        if odo <= 0.0 || !odo.is_finite() {
            continue;
        }
        if row.is_full_tank {
            if let Some(start_idx) = segment_start {
                let start_row = rows[start_idx];
                let start_odo = start_row.odometer.unwrap_or(0.0);
                let distance = odo - start_odo;
                if distance > 0.0 {
                    let mut total_fuel = 0.0;
                    let mut total_cost = 0.0;
                    let mut has_missed = false;
                    for fill in &rows[(start_idx + 1)..=i] {
                        total_fuel += normalize_fuel_to_liters(fill.fuel_amount, &fill.fuel_unit);
                        total_cost += fill.cost.unwrap_or(0.0);
                        has_missed |= fill.is_missed;
                    }
                    if total_fuel > 0.0 {
                        segments.push(Segment {
                            start_fillup_id: start_row.id,
                            end_fillup_id: row.id,
                            start_date: start_row.date.clone(),
                            end_date: row.date.clone(),
                            start_odometer: start_odo,
                            end_odometer: odo,
                            distance,
                            fuel: total_fuel,
                            cost: total_cost,
                            efficiency: distance / total_fuel,
                            cost_per_distance: total_cost / distance,
                            is_valid: !has_missed,
                        });
                    }
                }
            }
            segment_start = Some(i);
        }
    }
    segments
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, count) = values.fold((0.0, 0.0), |(sum, count), v| (sum + v, count + 1.0));
    (count > 0.0).then(|| sum / count)
}

/// Aggregate legacy raw statistics. `segments` must be the complete output of
/// `compute_segments(rows)` for the same vehicle and date range.
pub fn aggregate_stats(rows: &[Fillup], segments: &[Segment]) -> AggregatedStats {
    let rows: Vec<_> = ordered_rows(rows)
        .into_iter()
        .filter(|r| stats_eligible(r))
        .collect();
    let valid_odometers: Vec<_> = rows
        .iter()
        .filter_map(|r| r.odometer)
        .filter(|o| *o > 0.0)
        .collect();
    let total_distance = match (valid_odometers.first(), valid_odometers.last()) {
        (Some(first), Some(last)) => last - first,
        _ => 0.0,
    };
    AggregatedStats {
        total_distance,
        total_fuel: rows
            .iter()
            .map(|r| normalize_fuel_to_liters(r.fuel_amount, &r.fuel_unit))
            .sum(),
        total_cost: rows.iter().map(|r| r.cost.unwrap_or(0.0)).sum(),
        fill_up_count: i64::try_from(rows.len()).unwrap_or(i64::MAX),
        average_efficiency: mean(segments.iter().filter(|s| s.is_valid).map(|s| s.efficiency)),
        average_cost_per_distance: mean(
            segments
                .iter()
                .filter(|s| s.is_valid)
                .map(|s| s.cost_per_distance),
        ),
    }
}

/// Convert and round each segment exactly as the legacy history response does.
/// Ratios are converted from raw ratios, not recalculated from rounded fields.
pub fn displayed_segment(segment: &Segment, distance_unit: &str, volume_unit: &str) -> Segment {
    let dist = convert_distance(1.0, distance_unit);
    let vol = convert_volume(1.0, volume_unit);
    Segment {
        start_odometer: round2(convert_distance(segment.start_odometer, distance_unit)),
        end_odometer: round2(convert_distance(segment.end_odometer, distance_unit)),
        distance: round2(convert_distance(segment.distance, distance_unit)),
        fuel: round2(convert_volume(segment.fuel, volume_unit)),
        cost: round2(segment.cost),
        efficiency: round2(segment.efficiency * dist / vol),
        cost_per_distance: round2(segment.cost_per_distance / dist),
        ..segment.clone()
    }
}

/// Convert the legacy summary after raw aggregation (not a mean of rounded samples).
pub fn displayed_summary(
    stats: &AggregatedStats,
    distance_unit: &str,
    volume_unit: &str,
) -> AggregatedStats {
    let dist = convert_distance(1.0, distance_unit);
    let vol = convert_volume(1.0, volume_unit);
    AggregatedStats {
        total_distance: round2(convert_distance(stats.total_distance, distance_unit)),
        total_fuel: round2(convert_volume(stats.total_fuel, volume_unit)),
        total_cost: round2(stats.total_cost),
        average_efficiency: stats.average_efficiency.map(|v| round2(v * dist / vol)),
        average_cost_per_distance: stats.average_cost_per_distance.map(|v| round2(v / dist)),
        ..stats.clone()
    }
}

/// Dashboard average: equally weight every valid converted, ROUND2 history
/// sample in the complete population. Do not round the resulting mean again.
/// Can also consume concatenated fleet segments, provided their base units match.
pub fn displayed_average_efficiency(
    segments: &[Segment],
    distance_unit: &str,
    volume_unit: &str,
) -> Option<f64> {
    let dist = convert_distance(1.0, distance_unit);
    let vol = convert_volume(1.0, volume_unit);
    mean(
        segments
            .iter()
            .filter(|s| s.is_valid)
            .map(|s| round2(s.efficiency * dist / vol)),
    )
}

/// Fleet sums of already converted, ROUND2 vehicle summaries. Unlike the active
/// vehicle's mean valid-segment cost ratio, fleet ratios divide all-time totals.
#[derive(Debug, Default)]
pub struct FleetSummary {
    pub total_distance: f64,
    pub total_fuel: f64,
    pub total_cost: f64,
    pub fill_up_count: i64,
    pub cost_per_distance: Option<f64>,
    pub cost_per_volume: Option<f64>,
}

pub fn fleet_summary(vehicles: &[AggregatedStats]) -> FleetSummary {
    let mut fleet = FleetSummary::default();
    for vehicle in vehicles {
        fleet.total_distance += vehicle.total_distance;
        fleet.total_fuel += vehicle.total_fuel;
        fleet.total_cost += vehicle.total_cost;
        fleet.fill_up_count = fleet.fill_up_count.saturating_add(vehicle.fill_up_count);
    }
    fleet.cost_per_distance =
        (fleet.total_distance > 0.0).then(|| fleet.total_cost / fleet.total_distance);
    fleet.cost_per_volume = (fleet.total_fuel > 0.0).then(|| fleet.total_cost / fleet.total_fuel);
    fleet
}

#[cfg(test)]
mod bounded_tests;
#[cfg(test)]
mod tests;
