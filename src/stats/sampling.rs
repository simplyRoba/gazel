use super::CalculationError;

pub const PRICE_LIMIT: usize = 240;
pub const SPARKLINE_LIMIT: usize = 60;

/// A retained value at its original normalized population index (0 through 1).
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct SparklinePoint {
    pub position: f64,
    pub value: f64,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct SparklineSeries {
    pub points: Vec<SparklinePoint>,
    pub source_count: usize,
    pub reduced: bool,
}

// Fixed-budget min/max selection. Endpoints plus extrema of contiguous interior
// index buckets retain real observations and the global extrema. Strict
// comparisons keep the earliest original index on ties. Bucket boundaries use
// quotient/remainder arithmetic to avoid multiplying the full input length.
pub(super) fn retained_indices(values: &[f64], limit: usize) -> Vec<usize> {
    if values.len() <= limit {
        return (0..values.len()).collect();
    }
    let buckets = (limit - 2) / 2;
    let interior = values.len() - 2;
    let boundary =
        |bucket: usize| 1 + (interior / buckets) * bucket + (interior % buckets) * bucket / buckets;
    let mut indices = Vec::with_capacity(limit);
    indices.push(0);
    for bucket in 0..buckets {
        let start = boundary(bucket);
        let end = boundary(bucket + 1);
        let (mut min, mut max) = (start, start);
        for index in start + 1..end {
            if values[index] < values[min] {
                min = index;
            }
            if values[index] > values[max] {
                max = index;
            }
        }
        indices.push(min);
        indices.push(max);
    }
    indices.push(values.len() - 1);
    indices.sort_unstable();
    indices.dedup();
    indices
}

/// Sample already chronologically ordered values using 29 interior min/max
/// buckets. Preserve endpoints, extrema, and original relative positions; never
/// average or renumber retained observations. The caller owns eligibility and
/// date/identity ordering of segment populations.
///
/// # Errors
/// Returns an error for nonfinite values or an unrepresentable population size.
pub fn sample_sparkline(values: &[f64]) -> Result<SparklineSeries, CalculationError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CalculationError);
    }
    let denominator =
        f64::from(u32::try_from(values.len().saturating_sub(1)).map_err(|_| CalculationError)?);
    let points = retained_indices(values, SPARKLINE_LIMIT)
        .into_iter()
        .map(|index| {
            Ok(SparklinePoint {
                position: if denominator == 0.0 {
                    0.0
                } else {
                    f64::from(u32::try_from(index).map_err(|_| CalculationError)?) / denominator
                },
                value: values[index],
            })
        })
        .collect::<Result<_, CalculationError>>()?;
    Ok(SparklineSeries {
        points,
        source_count: values.len(),
        reduced: values.len() > SPARKLINE_LIMIT,
    })
}
