use chrono::{Duration, NaiveDate};

use super::tests::{close, fill, segment, values};
use super::*;

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

fn price_population(count: u32) -> Vec<FuelPrice> {
    (0..count)
        .map(|index| FuelPrice {
            fillup_id: i64::from(index),
            date: date("2020-01-01") + Duration::days(i64::from(index)),
            value: f64::from(index),
        })
        .collect()
}

#[test]
fn bounded_periods_anchor_to_latest_data_fill_gaps_and_omit_old_history() {
    let segments = [
        segment("1980-01-01", "1980-01-02", 50.0, 20.0),
        segment("2025-02-01", "2025-02-02", 100.0, 40.0),
    ];
    for period in [Period::Month, Period::Year] {
        for result in [
            bounded_period_costs(&segments, period).unwrap(),
            bounded_period_distances(&segments, period).unwrap(),
        ] {
            assert_eq!(result.points.len(), PERIOD_LIMIT);
            assert!(result.omitted_older_periods);
            assert_eq!(result.points[0].value, 0.0);
            assert!(result.points[1..11].iter().all(|point| point.value == 0.0));
            let range = result.displayed_range.unwrap();
            if period == Period::Month {
                assert_eq!(range.start, "2024-03");
                assert_eq!(range.end, "2025-02");
                assert_eq!(result.source_count, 542);
            } else {
                assert_eq!(range.start, "2014");
                assert_eq!(range.end, "2025");
                assert_eq!(result.source_count, 46);
            }
            assert_eq!(result.points[0].period, range.start);
            assert_eq!(result.points[11].period, range.end);
            assert_eq!(
                result.source_range.unwrap().start,
                if period == Period::Month {
                    "1980-01"
                } else {
                    "1980"
                }
            );
        }
    }
    close(
        bounded_period_costs(&segments, Period::Month)
            .unwrap()
            .points[11]
            .value,
        40.0,
    );
    close(
        bounded_period_distances(&segments, Period::Year)
            .unwrap()
            .points[11]
            .value,
        100.0,
    );
}

#[test]
fn bounded_periods_small_empty_and_exact_limit_do_not_pad_or_reduce() {
    for period in [Period::Month, Period::Year] {
        for result in [
            bounded_period_costs(&[], period).unwrap(),
            bounded_period_distances(&[], period).unwrap(),
        ] {
            assert!(result.points.is_empty());
            assert_eq!(result.source_count, 0);
            assert!(result.source_range.is_none());
            assert!(result.displayed_range.is_none());
            assert!(!result.omitted_older_periods);
        }
        let single = [segment("2025-03-01", "2025-03-02", 123.4, 4.56)];
        let costs = bounded_period_costs(&single, period).unwrap();
        let distance = bounded_period_distances(&single, period).unwrap();
        assert_eq!(costs.points.len(), 1);
        assert_eq!(distance.points.len(), 1);
        close(costs.points[0].value, 4.56);
        close(distance.points[0].value, 123.0);
        assert!(!costs.omitted_older_periods);
        assert_eq!(costs.source_range, costs.displayed_range);
    }
    let short = [
        segment("2023-02-01", "2023-02-01", 1.0, 1.0),
        segment("2025-02-01", "2025-02-01", 1.0, 1.0),
    ];
    values(
        &bounded_period_costs(&short, Period::Year).unwrap().points,
        &[("2023", 1.0), ("2024", 0.0), ("2025", 1.0)],
    );
    let twelve = [
        segment("2024-03-01", "2025-02-01", 20.0, 1.0),
        segment("2024-03-01", "2024-03-01", 1.0, 1.0),
    ];
    for series in [
        bounded_period_costs(&twelve, Period::Month).unwrap(),
        bounded_period_distances(&twelve, Period::Month).unwrap(),
    ] {
        assert_eq!(series.points.len(), 12);
        assert!(!series.omitted_older_periods);
    }
}

#[test]
fn clipped_segments_keep_full_denominator_and_round_only_final_totals() {
    let mut long = segment("1990-01-01", "2025-02-02", 100_000.01, 10.25);
    long.is_valid = false;
    let segments = [long.clone(), long];
    for period in [Period::Month, Period::Year] {
        let full = period_distances(&segments, period).unwrap();
        let bounded = bounded_period_distances(&segments, period).unwrap();
        assert_eq!(bounded.points, full[full.len() - 12..]);
        assert!(bounded.points.iter().map(|point| point.value).sum::<f64>() < 200_000.02);
        close(
            bounded_period_costs(&segments, period).unwrap().points[0].value,
            20.5,
        );
    }
    let small = [
        segment("2025-01-31", "2025-02-01", 0.6, 1.0),
        segment("2025-01-31", "2025-02-01", 0.6, 1.0),
    ];
    values(
        &bounded_period_distances(&small, Period::Month)
            .unwrap()
            .points,
        &[("2025-01", 1.0), ("2025-02", 1.0)],
    );
}

#[test]
fn bounding_does_not_mutate_inputs_or_change_exact_all_time_summaries() {
    let rows = [
        fill(2, "2025-02-02", Some(101000.0), 40.0, Some(80.0), true),
        fill(1, "1990-01-01", Some(1000.0), 40.0, Some(60.0), true),
    ];
    let segments = compute_segments(&rows);
    let before = format!(
        "{:?}",
        displayed_summary(&aggregate_stats(&rows, &segments), "mi", "gal")
    );
    let input = format!("{rows:?}{segments:?}");
    for period in [Period::Month, Period::Year] {
        bounded_period_costs(&segments, period).unwrap();
        bounded_period_distances(&segments, period).unwrap();
    }
    fuel_price_series(&fuel_prices(&rows, "gal")).unwrap();
    assert_eq!(
        before,
        format!(
            "{:?}",
            displayed_summary(&aggregate_stats(&rows, &segments), "mi", "gal")
        )
    );
    assert_eq!(input, format!("{rows:?}{segments:?}"));
    let stats = aggregate_stats(&rows, &segments);
    close(stats.total_distance, 100_000.0);
    close(stats.total_cost, 140.0);
}

#[test]
fn bounded_period_errors_include_omitted_invalid_input() {
    for period in [Period::Month, Period::Year] {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let segments = [
                segment("1900-01-01", "1900-02-01", bad, bad),
                segment("2025-01-01", "2025-02-01", 1.0, 1.0),
            ];
            assert!(bounded_period_costs(&segments, period).is_err());
            assert!(bounded_period_distances(&segments, period).is_err());
        }
        assert!(
            bounded_period_distances(&[segment("2025-03-01", "2025-02-01", 1.0, 1.0)], period)
                .is_err()
        );
        assert!(
            bounded_period_costs(&[segment("1900-01-01", "invalid", 1.0, 1.0)], period).is_err()
        );
        assert!(
            bounded_period_distances(&[segment("invalid", "2025-02-01", 1.0, 1.0)], period)
                .is_err()
        );
        let overflow = [
            segment("2025-01-01", "2025-01-01", f64::MAX, f64::MAX),
            segment("2025-01-01", "2025-01-01", f64::MAX, f64::MAX),
        ];
        assert!(bounded_period_costs(&overflow, period).is_err());
        assert!(bounded_period_distances(&overflow, period).is_err());
    }
}

#[test]
fn sampling_limit_boundaries_and_small_populations_return_real_values() {
    for count in [0, 1, 2, 17, 239, 240, 241, 10_000] {
        let prices = price_population(count);
        let result = sample_fuel_prices(&prices).unwrap();
        assert_eq!(result.source_count, prices.len());
        assert_eq!(result.reduced, count > 240);
        assert!(result.points.len() <= PRICE_LIMIT);
        assert_eq!(result.source_range, result.displayed_range);
        if count <= 240 {
            assert_eq!(result.points, prices);
        }
        if count == 0 {
            assert!(result.source_range.is_none());
        } else {
            assert_eq!(result.points.first(), prices.first());
            assert_eq!(result.points.last(), prices.last());
            assert_eq!(
                result.source_range.unwrap(),
                DateRange {
                    start: prices[0].date,
                    end: prices[prices.len() - 1].date
                }
            );
        }
        assert!(
            result
                .points
                .windows(2)
                .all(|pair| pair[0].fillup_id < pair[1].fillup_id)
        );
        for point in result.points {
            assert_eq!(point, prices[usize::try_from(point.fillup_id).unwrap()]);
        }
    }
    for count in [0, 1, 2, 17, 59, 60, 61, 10_000] {
        let values: Vec<_> = (0..count).map(f64::from).collect();
        let result = sample_sparkline(&values).unwrap();
        assert_eq!(result.source_count, values.len());
        assert_eq!(result.reduced, count > 60);
        assert!(result.points.len() <= SPARKLINE_LIMIT);
        if count <= 60 {
            assert_eq!(result.points.len(), values.len());
        }
        if count > 0 {
            close(result.points[0].position, 0.0);
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
}

#[test]
fn constant_sampling_uses_exact_bucket_boundaries_and_earliest_ties() {
    let mut prices = price_population(1000);
    for price in &mut prices {
        price.value = 1.234_567_89;
    }
    let result = sample_fuel_prices(&prices).unwrap();
    let mut expected = vec![0];
    expected.extend((0..119).map(|bucket| 1 + 998 * bucket / 119));
    expected.push(999);
    assert_eq!(
        result
            .points
            .iter()
            .map(|price| price.fillup_id)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(result.points.len(), 121); // min/max deduplicate only original indices
    assert!(
        result
            .points
            .iter()
            .all(|price| price.value == 1.234_567_89)
    );
    let spark = sample_sparkline(&vec![3.0; 1000]).unwrap();
    let mut expected = vec![0];
    expected.extend((0..29).map(|bucket| 1 + 998 * bucket / 29));
    expected.push(999);
    assert_eq!(spark.points.len(), 31);
    for (point, index) in spark.points.iter().zip(expected) {
        close(point.position, f64::from(index) / 999.0);
    }
}

#[test]
fn sampling_preserves_extrema_chronology_identity_positions_and_determinism() {
    let mut prices = price_population(10_000);
    prices[321].value = -99_000.0;
    prices[6789].value = 99_000.0;
    // Same-day values remain individual observations sorted by identity.
    prices[6788].date = prices[6789].date;
    prices.reverse();
    let original = prices.clone();
    let sampled = sample_fuel_prices(&prices).unwrap();
    assert_eq!(sampled, sample_fuel_prices(&prices).unwrap());
    assert_eq!(prices, original);
    assert!(sampled.points.iter().any(|price| price.fillup_id == 321));
    assert!(sampled.points.iter().any(|price| price.fillup_id == 6789));
    assert!(
        sampled
            .points
            .windows(2)
            .all(|pair| (pair[0].date, pair[0].fillup_id) < (pair[1].date, pair[1].fillup_id))
    );
    let same_date: Vec<_> = (0..1000)
        .rev()
        .map(|id| FuelPrice {
            fillup_id: id,
            date: date("2025-01-01"),
            value: 2.0,
        })
        .collect();
    let tied = fuel_price_series(&same_date).unwrap();
    assert!(tied.trend.is_none());
    assert_eq!(tied.samples.points[0].fillup_id, 0);
    assert_eq!(tied.samples.points.last().unwrap().fillup_id, 999);
    assert!(
        tied.samples
            .points
            .windows(2)
            .all(|pair| pair[0].fillup_id < pair[1].fillup_id)
    );
    let mut values: Vec<_> = (0..1000).map(f64::from).collect();
    values[123] = -10000.0;
    values[876] = 10000.0;
    let before = values.clone();
    let spark = sample_sparkline(&values).unwrap();
    assert_eq!(spark, sample_sparkline(&values).unwrap());
    assert_eq!(values, before);
    for index in [0, 123, 876, 999] {
        assert!(spark.points.iter().any(|point| point.value == values[index]
            && (point.position - f64::from(u32::try_from(index).unwrap()) / 999.0).abs() < 1e-12));
    }
    assert!(
        spark
            .points
            .windows(2)
            .all(|pair| pair[0].position < pair[1].position)
    );
}

#[test]
fn regression_known_line_constants_and_same_day_observations() {
    let mut prices = price_population(1000);
    for price in &mut prices {
        price.value = 0.123_456_789 + 0.01 * price.value;
    }
    let trend = fuel_price_series(&prices).unwrap().trend.unwrap();
    assert_eq!(trend[0].date, prices[0].date);
    assert_eq!(trend[1].date, prices[999].date);
    close(trend[0].value, prices[0].value);
    close(trend[1].value, prices[999].value);
    for value in [0.0, 1.234_567_89, f64::MAX] {
        for price in &mut prices {
            price.value = value;
        }
        let trend = fuel_price_trend(&prices).unwrap().unwrap();
        assert_eq!(trend[0].value, value);
        assert_eq!(trend[1].value, value);
    }
    let prices = [
        FuelPrice {
            fillup_id: 1,
            date: date("2025-01-01"),
            value: 1.0,
        },
        FuelPrice {
            fillup_id: 2,
            date: date("2025-01-01"),
            value: 3.0,
        },
        FuelPrice {
            fillup_id: 3,
            date: date("2025-01-02"),
            value: 4.0,
        },
    ];
    let fit = fuel_price_trend(&prices).unwrap().unwrap();
    close(fit[0].value, 2.0);
    close(fit[1].value, 4.0);
}

#[test]
fn regression_is_full_population_not_biased_extrema_subset() {
    let mut prices = price_population(1000);
    for price in &mut prices {
        price.value = 1.0;
    }
    for index in (10..490).step_by(17) {
        prices[index].value = 20.0;
    }
    let result = fuel_price_series(&prices).unwrap();
    let trend = result.trend.unwrap();
    let full = fuel_price_trend(&prices).unwrap().unwrap();
    assert_eq!(trend, full);
    let sampled_fit = fuel_price_trend(&result.samples.points).unwrap().unwrap();
    assert!((trend[0].value - sampled_fit[0].value).abs() > 0.1);
    // Independent textbook fit on this small fixture verifies the full mean/covariance.
    let mean_y = prices.iter().map(|price| price.value).sum::<f64>() / 1000.0;
    let covariance = prices
        .iter()
        .enumerate()
        .map(|(index, price)| {
            (f64::from(u32::try_from(index).unwrap()) - 499.5) * (price.value - mean_y)
        })
        .sum::<f64>();
    let variance = (0..1000)
        .map(|index| (f64::from(index) - 499.5).powi(2))
        .sum::<f64>();
    close(trend[0].value, mean_y - 499.5 * covariance / variance);
    close(trend[1].value, mean_y + 499.5 * covariance / variance);
}

#[test]
fn regression_empty_single_and_zero_date_domain_have_no_trend() {
    for prices in [
        Vec::new(),
        price_population(1),
        vec![
            FuelPrice {
                fillup_id: 1,
                date: date("2025-01-01"),
                value: 1.0,
            },
            FuelPrice {
                fillup_id: 2,
                date: date("2025-01-01"),
                value: 2.0,
            },
        ],
    ] {
        assert!(fuel_price_trend(&prices).unwrap().is_none());
        assert!(fuel_price_series(&prices).unwrap().trend.is_none());
    }
    let prices = price_population(2);
    let trend = fuel_price_trend(&prices).unwrap().unwrap();
    close(trend[0].value, 0.0);
    close(trend[1].value, 1.0);
}

#[test]
fn regression_centered_dates_and_scaled_values_stay_finite_and_precise() {
    for scale in [1e-300, 1e300] {
        let prices: Vec<_> = (0..100)
            .map(|index| FuelPrice {
                fillup_id: index,
                date: date("9999-01-01") + Duration::days(index),
                value: (1.0 + f64::from(u32::try_from(index).unwrap()) / 100.0) * scale,
            })
            .collect();
        let fit = fuel_price_trend(&prices).unwrap().unwrap();
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
    let fit = fuel_price_trend(&prices).unwrap().unwrap();
    close(fit[0].value, 1.0);
    close(fit[1].value, 2.0);
}

#[test]
fn public_series_helpers_reject_nonfinite_values_and_fit_overflow_safely() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let prices = [FuelPrice {
            fillup_id: 1,
            date: date("2025-01-01"),
            value: bad,
        }];
        assert!(sample_fuel_prices(&prices).is_err());
        assert!(fuel_price_trend(&prices).is_err());
        assert!(fuel_price_series(&prices).is_err());
        assert!(sample_sparkline(&[1.0, bad]).is_err());
    }
    let mut prices = price_population(3);
    prices[0].value = f64::MAX;
    prices[1].value = f64::MAX;
    prices[2].value = -f64::MAX;
    assert!(fuel_price_series(&prices).is_err());
}
