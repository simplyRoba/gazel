use super::tests::{close, segment, values};
use super::*;
use chrono::{Duration, NaiveDate};

fn price_population(count: u32) -> Vec<FuelPrice> {
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    (0..count)
        .map(|index| FuelPrice {
            fillup_id: i64::from(index),
            date: start + Duration::days(i64::from(index)),
            value: f64::from(index),
        })
        .collect()
}

#[test]
fn full_periods_fill_gaps_without_padding() {
    let segments = [
        segment("2000-01-01", "2000-01-02", 50.0, 20.0),
        segment("2025-02-01", "2025-02-02", 100.0, 40.0),
    ];
    for period in [Period::Month, Period::Year] {
        for (points, first) in [
            (period_costs(&segments, period).unwrap(), 20.0),
            (period_distances(&segments, period).unwrap(), 50.0),
        ] {
            assert_eq!(points.len(), if period == Period::Month { 302 } else { 26 });
            close(points.first().unwrap().value, first);
            assert!(points[1..points.len() - 1].iter().all(|p| p.value == 0.0));
        }
        assert_eq!(period_costs(&[], period).unwrap(), []);
        assert_eq!(period_distances(&[], period).unwrap(), []);
    }
    values(
        &period_costs(&segments, Period::Year).unwrap()[..3],
        &[("2000", 20.0), ("2001", 0.0), ("2002", 0.0)],
    );
}

#[test]
fn long_cross_year_distance_uses_full_interval_and_direct_rounding() {
    let s = segment("2000-02-29", "2025-03-01", 10000.0, 2.0);
    let monthly = period_distances(std::slice::from_ref(&s), Period::Month).unwrap();
    let yearly = period_distances(&[s], Period::Year).unwrap();
    assert_eq!(monthly.first().unwrap().period, "2000-02");
    assert_eq!(monthly.last().unwrap().period, "2025-03");
    assert_eq!(yearly.len(), 26);
    let short = [segment("2024-12-31", "2025-01-01", 1.0, 1.0)];
    values(
        &period_distances(&short, Period::Year).unwrap(),
        &[("2024", 1.0), ("2025", 1.0)],
    );
}

#[test]
fn all_prices_are_ordered_and_trend_uses_full_population() {
    let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let mut prices: Vec<_> = (0..500)
        .map(|i| FuelPrice {
            fillup_id: i,
            date: start + Duration::days(i / 2),
            value: if i == 17 { 1000.0 } else { 1.0 },
        })
        .collect();
    prices.reverse();
    let series = fuel_price_series(&prices).unwrap();
    assert_eq!(series.points.len(), 500);
    assert!(
        series
            .points
            .windows(2)
            .all(|p| (p[0].date, p[0].fillup_id) < (p[1].date, p[1].fillup_id))
    );
    let same = vec![
        FuelPrice {
            fillup_id: 2,
            date: start,
            value: 1.0,
        },
        FuelPrice {
            fillup_id: 1,
            date: start,
            value: 1.0,
        },
    ];
    let result = fuel_price_series(&same).unwrap();
    assert!(result.trend.is_none());
    assert_eq!(result.points[0].fillup_id, 1);
    assert_eq!(result.points.len(), 2);
    assert!(fuel_price_series(&[]).unwrap().trend.is_none());
}

#[test]
fn sparkline_sampling_stays_bounded_and_preserves_positions() {
    for count in [0, 1, 2, 59, 60, 61, 10000] {
        let values: Vec<_> = (0..count).map(f64::from).collect();
        let result = sample_sparkline(&values).unwrap();
        assert_eq!(result.source_count, values.len());
        assert_eq!(result.reduced, count > 60);
        assert!(result.points.len() <= SPARKLINE_LIMIT);
        if count <= 60 {
            assert_eq!(result.points.len(), values.len());
        }
        if count > 0 {
            close(result.points.first().unwrap().position, 0.0);
            if count > 1 {
                close(result.points.last().unwrap().position, 1.0);
            }
        }
        for point in result.points {
            close(
                point.position,
                if count <= 1 {
                    0.0
                } else {
                    point.value / f64::from(count - 1)
                },
            );
        }
    }
    let spark = sample_sparkline(&vec![3.0; 1000]).unwrap();
    assert_eq!(spark.points.len(), 31);
    let mut expected = vec![0];
    expected.extend((0..29).map(|bucket| 1 + 998 * bucket / 29));
    expected.push(999);
    for (point, index) in spark.points.iter().zip(expected) {
        close(point.position, f64::from(index) / 999.0);
        close(point.value, 3.0);
    }
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(sample_sparkline(&[1.0, bad]).is_err());
    }
}

#[test]
fn sparkline_extrema_keep_original_positions_and_are_deterministic() {
    let mut input: Vec<_> = (0..1000).map(f64::from).collect();
    input[123] = -10000.0;
    input[876] = 10000.0;
    let before = input.clone();
    let sampled = sample_sparkline(&input).unwrap();
    assert_eq!(sampled, sample_sparkline(&input).unwrap());
    assert_eq!(input, before);
    for index in [0, 123, 876, 999] {
        assert!(
            sampled
                .points
                .iter()
                .any(|point| (point.value - input[index]).abs() < 1e-12
                    && (point.position - f64::from(u32::try_from(index).unwrap()) / 999.0).abs()
                        < 1e-12)
        );
    }
    assert!(
        sampled
            .points
            .windows(2)
            .all(|pair| pair[0].position < pair[1].position)
    );
}

#[test]
fn price_regression_known_lines_and_same_day_observations() {
    let mut prices = price_population(1000);
    for price in &mut prices {
        price.value = 0.123_456_789 + 0.01 * price.value;
    }
    let trend = fuel_price_series(&prices).unwrap().trend.unwrap();
    assert_eq!(
        (trend[0].date, trend[1].date),
        (prices[0].date, prices[999].date)
    );
    close(trend[0].value, prices[0].value);
    close(trend[1].value, prices[999].value);
    for value in [0.0, 1.234_567_89, f64::MAX] {
        for price in &mut prices {
            price.value = value;
        }
        let trend = fuel_price_series(&prices).unwrap().trend.unwrap();
        assert_eq!([trend[0].value, trend[1].value], [value, value]);
    }
    let mut prices = price_population(3);
    prices[1].date = prices[0].date;
    prices[2].date = prices[0].date + Duration::days(1);
    for (price, value) in prices.iter_mut().zip([1.0, 3.0, 4.0]) {
        price.value = value;
    }
    let fit = fuel_price_series(&prices).unwrap().trend.unwrap();
    close(fit[0].value, 2.0);
    close(fit[1].value, 4.0);
    for input in [Vec::new(), price_population(1), prices[..2].to_vec()] {
        assert!(fuel_price_series(&input).unwrap().trend.is_none());
    }
}

#[test]
fn price_regression_uses_full_population_independent_fit() {
    let mut prices = price_population(1000);
    for price in &mut prices {
        price.value = 1.0;
    }
    for index in (10..490).step_by(17) {
        prices[index].value = 20.0;
    }
    let fit = fuel_price_series(&prices).unwrap().trend.unwrap();
    let mean = prices.iter().map(|price| price.value).sum::<f64>() / 1000.0;
    let covariance = prices
        .iter()
        .enumerate()
        .map(|(index, price)| {
            (f64::from(u32::try_from(index).unwrap()) - 499.5) * (price.value - mean)
        })
        .sum::<f64>();
    let variance = (0..1000)
        .map(|index| (f64::from(index) - 499.5).powi(2))
        .sum::<f64>();
    close(fit[0].value, mean - 499.5 * covariance / variance);
    close(fit[1].value, mean + 499.5 * covariance / variance);
}

#[test]
fn price_regression_extreme_dates_and_values_are_finite() {
    for scale in [1e-300, 1e300] {
        let prices: Vec<_> = (0..100)
            .map(|index| FuelPrice {
                fillup_id: index,
                date: NaiveDate::from_ymd_opt(9999, 1, 1).unwrap() + Duration::days(index),
                value: (1.0 + f64::from(u32::try_from(index).unwrap()) / 100.0) * scale,
            })
            .collect();
        let fit = fuel_price_series(&prices).unwrap().trend.unwrap();
        assert!(fit.iter().all(|point| point.value.is_finite()));
        close(fit[0].value / scale, 1.0);
        close(fit[1].value / scale, 1.99);
    }
    let prices = [
        FuelPrice {
            fillup_id: 1,
            date: NaiveDate::MIN,
            value: 1.0,
        },
        FuelPrice {
            fillup_id: 2,
            date: NaiveDate::MAX,
            value: 2.0,
        },
    ];
    let fit = fuel_price_series(&prices).unwrap().trend.unwrap();
    close(fit[0].value, 1.0);
    close(fit[1].value, 2.0);
}

#[test]
fn price_regression_rejects_nonfinite_and_overflow() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut prices = price_population(1);
        prices[0].value = bad;
        assert!(fuel_price_trend(&prices).is_err());
        assert!(fuel_price_series(&prices).is_err());
    }
    let mut prices = price_population(3);
    prices[0].value = f64::MAX;
    prices[1].value = f64::MAX;
    prices[2].value = -f64::MAX;
    assert!(fuel_price_series(&prices).is_err());
}
