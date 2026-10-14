use chrono::NaiveDate;

use super::sampling::retained_indices;
use super::{
    CalculationError, Fillup, PRICE_LIMIT, convert_volume, normalize_fuel_to_liters, ordered_rows,
};

/// An actual recorded, unrounded price, retaining date and identity.
#[derive(Clone, Debug, PartialEq)]
pub struct FuelPrice {
    pub fillup_id: i64,
    pub date: NaiveDate,
    pub value: f64,
}

/// Prices from all fills, independent of segment/full-tank/missed/odometer
/// eligibility. Normalize volume before division; retain free fuel. Invalid
/// dates, missing costs, nonfinite arithmetic and nonpositive fuel are skipped.
/// Sort a copy by date then ID. Values are deliberately never rounded.
pub fn fuel_prices(rows: &[Fillup], volume_unit: &str) -> Vec<FuelPrice> {
    ordered_rows(rows)
        .into_iter()
        .filter_map(|row| {
            let cost = row.cost.filter(|v| v.is_finite())?;
            if !row.fuel_amount.is_finite() || row.fuel_amount <= 0.0 {
                return None;
            }
            let date = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").ok()?;
            let volume = convert_volume(
                normalize_fuel_to_liters(row.fuel_amount, &row.fuel_unit),
                volume_unit,
            );
            if !volume.is_finite() || volume <= 0.0 {
                return None;
            }
            let value = cost / volume;
            value.is_finite().then_some(FuelPrice {
                fillup_id: row.id,
                date,
                value,
            })
        })
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, PartialEq)]
pub struct SampledFuelPrices {
    pub points: Vec<FuelPrice>,
    pub source_count: usize,
    pub reduced: bool,
    pub source_range: Option<DateRange>,
    pub displayed_range: Option<DateRange>,
}

#[derive(Debug, PartialEq)]
pub struct PriceTrendPoint {
    pub date: NaiveDate,
    pub value: f64,
}

#[derive(Debug, PartialEq)]
pub struct FuelPriceSeries {
    pub samples: SampledFuelPrices,
    pub trend: Option<[PriceTrendPoint; 2]>,
}

fn ordered_prices(prices: &[FuelPrice]) -> Result<Vec<&FuelPrice>, CalculationError> {
    if prices.iter().any(|price| !price.value.is_finite()) {
        return Err(CalculationError);
    }
    let mut ordered: Vec<_> = prices.iter().collect();
    ordered.sort_by_key(|price| (price.date, price.fillup_id));
    Ok(ordered)
}

fn sample_ordered(prices: &[&FuelPrice]) -> SampledFuelPrices {
    let values: Vec<_> = prices.iter().map(|price| price.value).collect();
    let points = retained_indices(&values, PRICE_LIMIT)
        .into_iter()
        .map(|index| prices[index].clone())
        .collect();
    let range = || {
        prices
            .first()
            .zip(prices.last())
            .map(|(first, last)| DateRange {
                start: first.date,
                end: last.date,
            })
    };
    SampledFuelPrices {
        points,
        source_count: prices.len(),
        reduced: prices.len() > PRICE_LIMIT,
        source_range: range(),
        displayed_range: range(),
    }
}

/// Retain all prices through 240; otherwise retain endpoints and min/max of
/// 119 contiguous chronological interior index buckets. Ties retain the earliest
/// original date/ID index. Deduplicate indices, never identities or values, and
/// preserve original unrounded prices. Sort a copy without mutating the input.
/// For chart data with a full-population trend, use `fuel_price_series` instead.
///
/// # Errors
/// Returns an error for nonfinite input prices.
pub fn sample_fuel_prices(prices: &[FuelPrice]) -> Result<SampledFuelPrices, CalculationError> {
    Ok(sample_ordered(&ordered_prices(prices)?))
}

fn trend_ordered(prices: &[&FuelPrice]) -> Result<Option<[PriceTrendPoint; 2]>, CalculationError> {
    let Some((first, last)) = prices.first().zip(prices.last()) else {
        return Ok(None);
    };
    if prices.len() < 2 || first.date == last.date {
        return Ok(None);
    }
    let count = f64::from(u32::try_from(prices.len()).map_err(|_| CalculationError)?);
    let width = f64::from(
        i32::try_from((last.date - first.date).num_days()).map_err(|_| CalculationError)?,
    );
    // Relative calendar coordinates avoid cancellation from absolute timestamps.
    // Scale values as well to keep squares/products safe at tiny or huge scales.
    let scale = prices
        .iter()
        .map(|price| price.value.abs())
        .fold(0.0, f64::max);
    let x = |price: &FuelPrice| -> Result<f64, CalculationError> {
        Ok(f64::from(
            i32::try_from((price.date - first.date).num_days()).map_err(|_| CalculationError)?,
        ) / width
            - 0.5)
    };
    let y = |price: &FuelPrice| {
        if scale == 0.0 {
            0.0
        } else {
            price.value / scale
        }
    };
    let (mut mean_x, mut mean_y, mut seen) = (0.0, 0.0, 0.0);
    for price in prices {
        seen += 1.0;
        mean_x += (x(price)? - mean_x) / seen;
        mean_y += (y(price) - mean_y) / seen;
    }
    let (mut covariance, mut variance) = (0.0, 0.0);
    for price in prices {
        let dx = x(price)? - mean_x;
        covariance += dx * (y(price) - mean_y) / count;
        variance += dx * dx / count;
    }
    let slope = covariance / variance;
    let endpoint =
        |date: NaiveDate, coordinate: f64| -> Result<PriceTrendPoint, CalculationError> {
            let value = (mean_y + slope * (coordinate - mean_x)) * scale;
            if !value.is_finite() {
                return Err(CalculationError);
            }
            Ok(PriceTrendPoint { date, value })
        };
    Ok(Some([
        endpoint(first.date, -0.5)?,
        endpoint(last.date, 0.5)?,
    ]))
}

/// Ordinary least squares over every input observation, including same-date
/// identities individually. Return only fitted endpoints at the source dates;
/// fewer than two observations or a zero-width date domain have no trend.
///
/// # Errors
/// Returns an error for nonfinite prices, unrepresentable population size or
/// nonfinite fitted endpoints (including arithmetic overflow).
pub fn fuel_price_trend(
    prices: &[FuelPrice],
) -> Result<Option<[PriceTrendPoint; 2]>, CalculationError> {
    trend_ordered(&ordered_prices(prices)?)
}

/// Build fuel-price chart data safely: fit the complete eligible population
/// FIRST, then reduce recorded points. Neither operation rounds price precision.
///
/// # Errors
/// Returns an error for nonfinite prices, unrepresentable population size or
/// nonfinite fitted endpoints.
pub fn fuel_price_series(prices: &[FuelPrice]) -> Result<FuelPriceSeries, CalculationError> {
    let ordered = ordered_prices(prices)?;
    let trend = trend_ordered(&ordered)?;
    let samples = sample_ordered(&ordered);
    Ok(FuelPriceSeries { samples, trend })
}
