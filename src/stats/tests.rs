use super::*;

pub(super) fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}

pub(super) fn fill(
    id: i64,
    date: &str,
    odo: Option<f64>,
    fuel: f64,
    cost: Option<f64>,
    is_full_tank: bool,
) -> Fillup {
    Fillup {
        id,
        date: date.into(),
        odometer: odo,
        fuel_amount: fuel,
        fuel_unit: "l".into(),
        cost,
        is_full_tank,
        is_missed: false,
    }
}

pub(super) fn segment(start: &str, end: &str, distance: f64, cost: f64) -> Segment {
    Segment {
        start_fillup_id: 1,
        end_fillup_id: 2,
        start_date: start.into(),
        end_date: end.into(),
        start_odometer: 1000.0,
        end_odometer: 1000.0 + distance,
        distance,
        fuel: 40.0,
        cost,
        efficiency: distance / 40.0,
        cost_per_distance: cost / distance,
        is_valid: true,
    }
}

pub(super) fn values(points: &[PeriodPoint], expected: &[(&str, f64)]) {
    assert_eq!(points.len(), expected.len());
    for (point, (period, value)) in points.iter().zip(expected) {
        assert_eq!(point.period, *period);
        close(point.value, *value);
    }
}

#[test]
fn tank_to_tank_standard_partial_and_same_day_identity() {
    let rows = vec![
        fill(3, "2024-01-01", Some(1500.0), 25.0, Some(50.0), true),
        fill(1, "2024-01-01", Some(1000.0), 40.0, Some(80.0), true),
        fill(2, "2024-01-01", Some(1200.0), 15.0, Some(30.0), false),
    ];
    let segments = compute_segments(&rows);
    assert_eq!(segments.len(), 1);
    let s = &segments[0];
    assert_eq!((s.start_fillup_id, s.end_fillup_id), (1, 3));
    close(s.distance, 500.0);
    close(s.fuel, 40.0);
    close(s.cost, 80.0);
    close(s.efficiency, 12.5);
    close(s.cost_per_distance, 0.16);
    assert!(s.is_valid);
    let agg = aggregate_stats(&rows, &segments);
    close(agg.total_fuel, 80.0);
    close(agg.total_cost, 160.0);
    close(agg.total_distance, 500.0);
    assert_eq!(agg.fill_up_count, 3);
    assert_eq!(rows[0].id, 3); // sorting does not mutate input
}

#[test]
fn null_rows_accumulate_but_zero_and_negative_rows_are_excluded() {
    let mut rows = vec![
        fill(1, "2024-01-01", Some(1000.0), 40.0, Some(80.0), true),
        fill(2, "2024-01-02", Some(0.0), 999.0, Some(999.0), true),
        fill(3, "2024-01-03", Some(-1.0), 999.0, Some(999.0), false),
        fill(4, "2024-01-04", None, 15.0, Some(30.0), true),
        fill(5, "2024-01-05", Some(1500.0), 25.0, Some(50.0), true),
    ];
    rows[1].is_missed = true;
    let segments = compute_segments(&rows);
    assert!(segments[0].is_valid);
    close(segments[0].fuel, 40.0);
    let agg = aggregate_stats(&rows, &segments);
    assert_eq!(agg.fill_up_count, 3);
    close(agg.total_cost, 160.0);
    close(agg.total_distance, 500.0);
    rows[3].is_missed = true;
    let invalid = compute_segments(&rows);
    assert!(!invalid[0].is_valid);
    assert!(
        aggregate_stats(&rows, &invalid)
            .average_efficiency
            .is_none()
    );
    close(invalid[0].cost_per_distance, 0.16);
}

#[test]
fn empty_single_nonincreasing_and_zero_fuel_populations() {
    let empty = aggregate_stats(&[], &[]);
    assert_eq!(empty.fill_up_count, 0);
    close(empty.total_distance, 0.0);
    assert!(empty.average_efficiency.is_none());
    let rows = vec![fill(1, "2024-01-01", Some(1000.0), 40.0, None, true)];
    assert!(compute_segments(&rows).is_empty());
    close(aggregate_stats(&rows, &[]).total_fuel, 40.0);
    for (odo, fuel) in [(1000.0, 40.0), (900.0, 40.0), (1500.0, 0.0)] {
        let mut two = rows.clone();
        two.push(fill(2, "2024-02-01", Some(odo), fuel, None, true));
        assert!(compute_segments(&two).is_empty());
    }
}

#[test]
fn summary_sums_raw_before_rounding_and_converts_mixed_fuel() {
    let mut rows = vec![
        fill(1, "2024-01-01", Some(1000.0), 1.004, Some(0.004), true),
        fill(2, "2024-02-01", Some(1500.0), 1.0, Some(0.004), true),
    ];
    rows[1].fuel_unit = "gal".into();
    let segments = compute_segments(&rows);
    close(segments[0].fuel, L_PER_GAL);
    let agg = aggregate_stats(&rows, &segments);
    let metric = displayed_summary(&agg, "km", "l");
    close(metric.total_cost, 0.01); // not sum(round2(each cost))
    close(metric.total_fuel, 4.79);
    let imperial = displayed_summary(&agg, "mi", "gal");
    close(imperial.total_distance, round2(500.0 / KM_PER_MI));
    close(imperial.total_fuel, round2((1.004 + L_PER_GAL) / L_PER_GAL));
    close(
        imperial.average_efficiency.unwrap(),
        round2(500.0 / KM_PER_MI),
    );
}

#[test]
fn displayed_average_rounds_converted_samples_and_weights_segments_not_vehicles() {
    let mut a = segment("2024-01-01", "2024-02-01", 400.0, 60.0);
    a.efficiency = 1.004;
    let mut b = a.clone();
    b.efficiency = 1.0051;
    let mut c = a.clone();
    c.efficiency = 9.0;
    let mut invalid = a.clone();
    invalid.efficiency = 999.0;
    invalid.is_valid = false;
    // Vehicle A has two samples; vehicle B has one.
    let all = vec![a.clone(), b.clone(), c, invalid];
    close(
        displayed_average_efficiency(&all, "km", "l").unwrap(),
        (1.0 + 1.01 + 9.0) / 3.0,
    );
    let active = [a, b];
    close(
        displayed_average_efficiency(&active, "km", "l").unwrap(),
        1.005,
    );
    let factor = L_PER_GAL / KM_PER_MI;
    close(
        displayed_average_efficiency(&active, "mi", "gal").unwrap(),
        round2(1.004 * factor).midpoint(round2(1.0051 * factor)),
    );
    assert!(displayed_average_efficiency(&[], "km", "l").is_none());
    assert!(displayed_average_efficiency(&[all[3].clone()], "km", "l").is_none());
}

#[test]
fn legacy_summary_averages_raw_ratios_before_conversion_and_rounding() {
    let mut first = segment("2024-01-01", "2024-02-01", 400.0, 60.0);
    first.efficiency = 1.004;
    first.cost_per_distance = 0.104;
    let mut second = first.clone();
    second.efficiency = 1.0051;
    second.cost_per_distance = 0.1051;
    let mut invalid = first.clone();
    invalid.efficiency = 999.0;
    invalid.cost_per_distance = 999.0;
    invalid.is_valid = false;
    let segments = [first, second, invalid];
    let raw = aggregate_stats(&[], &segments);
    close(raw.average_efficiency.unwrap(), 1.00455);
    close(raw.average_cost_per_distance.unwrap(), 0.10455);
    let display = displayed_summary(&raw, "km", "l");
    close(display.average_efficiency.unwrap(), 1.0);
    close(display.average_cost_per_distance.unwrap(), 0.1);
    close(
        displayed_average_efficiency(&segments, "km", "l").unwrap(),
        1.005,
    );
    let imperial = displayed_summary(&raw, "mi", "gal");
    close(
        imperial.average_efficiency.unwrap(),
        round2(1.00455 * L_PER_GAL / KM_PER_MI),
    );
    close(
        imperial.average_cost_per_distance.unwrap(),
        round2(0.10455 * KM_PER_MI),
    );
}

#[test]
fn fleet_sums_rounded_vehicle_totals_and_uses_total_ratios_not_active_mean() {
    let rows = [
        fill(1, "2024-01-01", Some(1000.0), 10.004, Some(10.004), true),
        fill(2, "2024-02-01", Some(1100.0), 10.0, Some(10.0), true),
        fill(3, "2024-03-01", Some(2000.004), 90.0, Some(180.0), true),
    ];
    let segs = compute_segments(&rows);
    let vehicle = displayed_summary(&aggregate_stats(&rows, &segs), "km", "l");
    close(vehicle.average_cost_per_distance.unwrap(), 0.15);
    let other = displayed_summary(
        &AggregatedStats {
            total_cost: 0.004,
            total_distance: 0.004,
            ..AggregatedStats::default()
        },
        "km",
        "l",
    );
    let fleet = fleet_summary(&[vehicle, other, AggregatedStats::default()]);
    close(fleet.total_cost, 200.0);
    close(fleet.total_distance, 1000.0);
    close(fleet.cost_per_distance.unwrap(), 0.2);
    close(fleet.cost_per_volume.unwrap(), 200.0 / 110.0);
    let empty = fleet_summary(&[]);
    assert!(empty.cost_per_distance.is_none());
    assert!(empty.cost_per_volume.is_none());
    assert!(
        fleet_summary(&[AggregatedStats::default()])
            .cost_per_distance
            .is_none()
    );
}

#[test]
fn monthly_costs_end_period_invalid_segments_and_year_gaps_match_frontend() {
    let mut first = segment("2024-10-01", "2024-11-01", 500.0, 10.0);
    first.is_valid = false;
    let later = segment("2025-01-01", "2025-02-01", 600.0, 20.0);
    values(
        &period_costs(&[later, first], Period::Month).unwrap(),
        &[
            ("2024-11", 10.0),
            ("2024-12", 0.0),
            ("2025-01", 0.0),
            ("2025-02", 20.0),
        ],
    );
    values(
        &period_costs(
            &[
                segment("2026-01-01", "2026-02-01", 10.0, 20.0),
                segment("2024-01-01", "2024-02-01", 10.0, 10.0),
                segment("2024-02-01", "2024-03-01", 10.0, 5.0),
            ],
            Period::Year,
        )
        .unwrap(),
        &[("2024", 15.0), ("2026", 20.0)],
    );
}

#[test]
fn inclusive_distance_month_year_boundaries_and_leap_days() {
    values(
        &period_distances(
            &[segment("2024-02-28", "2024-03-01", 300.0, 1.0)],
            Period::Month,
        )
        .unwrap(),
        &[("2024-02", 200.0), ("2024-03", 100.0)],
    );
    values(
        &period_distances(
            &[segment("2024-12-31", "2025-01-01", 200.0, 1.0)],
            Period::Year,
        )
        .unwrap(),
        &[("2024", 100.0), ("2025", 100.0)],
    );
    values(
        &period_distances(
            &[segment("2025-03-01", "2025-03-01", 100.0, 1.0)],
            Period::Month,
        )
        .unwrap(),
        &[("2025-03", 100.0)],
    );
}

#[test]
fn distance_month_gaps_unordered_and_multi_month_proportions() {
    let mut first = segment("2025-01-01", "2025-01-15", 500.0, 1.0);
    first.is_valid = false;
    values(
        &period_distances(
            &[segment("2025-03-01", "2025-03-15", 600.0, 1.0), first],
            Period::Month,
        )
        .unwrap(),
        &[("2025-01", 500.0), ("2025-02", 0.0), ("2025-03", 600.0)],
    );
    values(
        &period_distances(
            &[segment("2025-01-01", "2025-03-01", 600.0, 1.0)],
            Period::Month,
        )
        .unwrap(),
        &[("2025-01", 310.0), ("2025-02", 280.0), ("2025-03", 10.0)],
    );
}

#[test]
fn calendar_allocation_ignores_spring_and_fall_clock_changes() {
    values(
        &period_distances(
            &[segment("2025-03-08", "2025-04-01", 250.0, 1.0)],
            Period::Month,
        )
        .unwrap(),
        &[("2025-03", 240.0), ("2025-04", 10.0)],
    );
    values(
        &period_distances(
            &[segment("2025-10-31", "2025-11-03", 400.0, 1.0)],
            Period::Month,
        )
        .unwrap(),
        &[("2025-10", 100.0), ("2025-11", 300.0)],
    );
}

#[test]
fn yearly_distance_allocates_directly_instead_of_summing_monthly_rounding() {
    let segments = [segment("2025-01-31", "2025-02-01", 1.0, 1.0)];
    values(
        &period_distances(&segments, Period::Month).unwrap(),
        &[("2025-01", 1.0), ("2025-02", 1.0)],
    );
    values(
        &period_distances(&segments, Period::Year).unwrap(),
        &[("2025", 1.0)],
    );
    values(
        &period_distances(
            &[
                segment("2024-12-31", "2025-01-02", 300.0, 1.0),
                segment("2027-01-01", "2027-01-01", 40.0, 1.0),
            ],
            Period::Year,
        )
        .unwrap(),
        &[("2024", 100.0), ("2025", 200.0), ("2027", 40.0)],
    );
}

#[test]
fn period_empty_invalid_dates_and_arithmetic_edges_are_safe() {
    for period in [Period::Month, Period::Year] {
        assert_eq!(period_costs(&[], period).unwrap(), []);
        assert_eq!(period_distances(&[], period).unwrap(), []);
        assert!(period_distances(&[segment("banana", "2025-01-01", 10.0, 1.0)], period).is_err());
        assert!(period_costs(&[segment("2025-01-01", "2025-02-30", 10.0, 1.0)], period).is_err());
        assert!(
            period_distances(&[segment("2025-02-01", "2025-01-01", 10.0, 1.0)], period).is_err()
        );
        assert!(
            period_distances(
                &[segment("2025-01-01", "2025-02-01", f64::INFINITY, 1.0)],
                period
            )
            .is_err()
        );
        assert!(
            period_costs(
                &[segment("2025-01-01", "2025-02-01", 10.0, f64::INFINITY)],
                period
            )
            .is_err()
        );
        let max_date = chrono::NaiveDate::MAX.format("%Y-%m-%d").to_string();
        assert!(period_distances(&[segment(&max_date, &max_date, 10.0, 1.0)], period).is_err());
    }
}

#[test]
fn prices_include_first_partial_missed_zero_null_odometer_free_fuel_and_identity() {
    let mut missed = fill(2, "2025-08-15", None, 30.0, Some(60.0), false);
    missed.is_missed = true;
    let rows = [
        fill(3, "2025-09-01", Some(-1.0), 30.0, Some(0.0), false),
        missed,
        fill(1, "2025-08-15", Some(0.0), 30.0, Some(45.0), false),
    ];
    assert!(compute_segments(&rows).is_empty());
    let prices = fuel_prices(&rows, "l");
    assert_eq!(
        prices.iter().map(|p| p.fillup_id).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    for (price, expected) in prices.iter().zip([1.5, 2.0, 0.0]) {
        close(price.value, expected);
    }
    assert_eq!(prices[0].date.to_string(), "2025-08-15");
    assert_eq!(rows[0].id, 3);
}

#[test]
fn prices_normalize_mixed_units_in_both_directions_without_rounding() {
    let mut gallon = fill(2, "2025-08-15", None, 1.0, Some(8.0), false);
    gallon.fuel_unit = "gal".into();
    let rows = [
        fill(1, "2025-08-15", None, L_PER_GAL, Some(8.0), false),
        gallon,
    ];
    for price in fuel_prices(&rows, "l") {
        close(price.value, 8.0 / L_PER_GAL);
    }
    for price in fuel_prices(&rows, "gal") {
        close(price.value, 8.0);
    }
    let precise = fuel_prices(&[fill(1, "2025-08-15", None, 3.0, Some(1.0), false)], "l");
    close(precise[0].value, 1.0 / 3.0);
    assert!((precise[0].value - round2(precise[0].value)).abs() > 0.001);
}

#[test]
fn prices_skip_invalid_numbers_dates_and_normalization_overflow() {
    assert_eq!(fuel_prices(&[], "l"), []);
    for fuel in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            fuel_prices(&[fill(1, "2025-08-15", None, fuel, Some(1.0), false)], "l"),
            []
        );
    }
    for cost in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        assert_eq!(
            fuel_prices(&[fill(1, "2025-08-15", None, 1.0, cost, false)], "l"),
            []
        );
    }
    let mut overflow = fill(1, "2025-08-15", None, f64::MAX, Some(1.0), false);
    overflow.fuel_unit = "gal".into();
    assert_eq!(fuel_prices(&[overflow], "l"), []);
    assert_eq!(
        fuel_prices(
            &[fill(
                1,
                "2025-08-15",
                None,
                f64::MIN_POSITIVE,
                Some(f64::MAX),
                false
            )],
            "l"
        ),
        []
    );
    assert_eq!(
        fuel_prices(&[fill(1, "2025-02-30", None, 1.0, Some(1.0), false)], "l"),
        []
    );
}
