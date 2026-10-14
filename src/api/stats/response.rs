use serde::Serialize;

use crate::stats::{
    self, AggregatedStats, CalculationError, DateRange, Fillup, Period, PeriodSeries, Segment,
    SparklineSeries,
};

#[derive(Serialize, sqlx::FromRow)]
pub struct Units {
    pub distance_unit: String,
    pub volume_unit: String,
    pub currency: String,
}

#[derive(Serialize, Clone)]
pub struct VehicleSummary {
    pub vehicle_id: i64,
    pub total_distance: f64,
    pub total_fuel: f64,
    pub total_cost: f64,
    pub fill_up_count: i64,
    pub average_efficiency: Option<f64>,
    pub average_cost_per_distance: Option<f64>,
}

impl VehicleSummary {
    pub fn new(vehicle_id: i64, s: &AggregatedStats) -> Self {
        Self {
            vehicle_id,
            total_distance: s.total_distance,
            total_fuel: s.total_fuel,
            total_cost: s.total_cost,
            fill_up_count: s.fill_up_count,
            average_efficiency: s.average_efficiency,
            average_cost_per_distance: s.average_cost_per_distance,
        }
    }
}

#[derive(Serialize)]
pub struct Fleet {
    pub total_distance: f64,
    pub total_fuel: f64,
    pub total_cost: f64,
    pub fill_up_count: i64,
    pub average_efficiency: Option<f64>,
    pub cost_per_distance: Option<f64>,
    pub cost_per_volume: Option<f64>,
}

impl Fleet {
    pub fn new(s: &stats::FleetSummary, average_efficiency: Option<f64>) -> Self {
        Self {
            total_distance: s.total_distance,
            total_fuel: s.total_fuel,
            total_cost: s.total_cost,
            fill_up_count: s.fill_up_count,
            average_efficiency,
            cost_per_distance: s.cost_per_distance,
            cost_per_volume: s.cost_per_volume,
        }
    }
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub units: Units,
    pub fleet: Fleet,
    pub vehicles: Vec<VehicleSummary>,
    pub fleet_efficiency: SparklineSeries,
    pub selected: Option<SelectedStats>,
}

#[derive(Serialize)]
pub struct SelectedStats {
    pub vehicle_id: i64,
    pub summary: VehicleSummary,
    pub charts: Charts,
    pub cost_per_distance: SparklineSeries,
}

impl SelectedStats {
    pub fn new(
        summary: VehicleSummary,
        rows: &[Fillup],
        segments: &[Segment],
        units: &Units,
    ) -> Result<Self, CalculationError> {
        let prices = stats::fuel_prices(rows, &units.volume_unit);
        let prices = stats::fuel_price_series(&prices)?;
        let samples = prices.samples;
        Ok(Self {
            vehicle_id: summary.vehicle_id,
            summary,
            charts: Charts {
                cost: PeriodModes::cost(segments)?,
                distance: PeriodModes::distance(segments)?,
                fuel_price: FuelPrices {
                    points: samples
                        .points
                        .into_iter()
                        .map(|p| PricePoint {
                            date: p.date.to_string(),
                            value: p.value,
                        })
                        .collect(),
                    source_count: samples.source_count,
                    reduced: samples.reduced,
                    source_range: samples.source_range.map(WireDateRange::from),
                    displayed_range: samples.displayed_range.map(WireDateRange::from),
                    trend: prices.trend.map(|points| {
                        points.map(|p| PricePoint {
                            date: p.date.to_string(),
                            value: p.value,
                        })
                    }),
                },
            },
            cost_per_distance: stats::sample_sparkline(
                &segments
                    .iter()
                    .map(|s| s.cost_per_distance)
                    .collect::<Vec<_>>(),
            )?,
        })
    }
}

#[derive(Serialize)]
pub struct Charts {
    cost: PeriodModes,
    distance: PeriodModes,
    fuel_price: FuelPrices,
}

#[derive(Serialize)]
struct PeriodModes {
    monthly: PeriodSeries,
    yearly: PeriodSeries,
}

impl PeriodModes {
    fn cost(segments: &[Segment]) -> Result<Self, CalculationError> {
        Ok(Self {
            monthly: stats::bounded_period_costs(segments, Period::Month)?,
            yearly: stats::bounded_period_costs(segments, Period::Year)?,
        })
    }

    fn distance(segments: &[Segment]) -> Result<Self, CalculationError> {
        Ok(Self {
            monthly: stats::bounded_period_distances(segments, Period::Month)?,
            yearly: stats::bounded_period_distances(segments, Period::Year)?,
        })
    }
}

#[derive(Serialize)]
struct PricePoint {
    date: String,
    value: f64,
}

#[derive(Serialize)]
struct WireDateRange {
    start: String,
    end: String,
}

impl From<DateRange> for WireDateRange {
    fn from(range: DateRange) -> Self {
        Self {
            start: range.start.to_string(),
            end: range.end.to_string(),
        }
    }
}

#[derive(Serialize)]
struct FuelPrices {
    points: Vec<PricePoint>,
    source_count: usize,
    reduced: bool,
    source_range: Option<WireDateRange>,
    displayed_range: Option<WireDateRange>,
    trend: Option<[PricePoint; 2]>,
}
