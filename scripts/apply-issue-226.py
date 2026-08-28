#!/usr/bin/env python3
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


sweep_path = Path("crates/anthrosim-cli/src/sweep.rs")
s = sweep_path.read_text(encoding="utf-8")

s = replace_once(s, "const DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 5;\nconst DERIVED_POINT_ANALYSIS_SCHEMA_VERSION: u32 = 6;",
                 "const DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 6;\nconst DERIVED_POINT_ANALYSIS_SCHEMA_VERSION: u32 = 7;",
                 "schema versions")

s = replace_once(s,
'''    migration_total_distance_cells: Option<u64>,
    migration_mean_origin_resource_score_permille: Option<u16>,''',
'''    migration_total_distance_cells: Option<u64>,
    births_per_365_simulated_days: Option<f64>,
    deaths_per_365_simulated_days: Option<f64>,
    resource_scarcity_deaths_per_365_simulated_days: Option<f64>,
    resource_unmet_need_per_365_simulated_days: Option<f64>,
    migration_moves_per_365_simulated_days: Option<f64>,
    migration_distance_cells_per_365_simulated_days: Option<f64>,
    migration_mean_origin_resource_score_permille: Option<u16>,''',
                 "run rate fields")

s = replace_once(s,
'''    operationally_censored_runs: u64,
    living_condition_defined_runs_scientifically_eligible_only: u64,''',
'''    operationally_censored_runs: u64,
    population_extinction_fraction_scientifically_eligible_only: Option<f64>,
    mean_simulated_days_scientifically_eligible_only: Option<f64>,
    min_simulated_days_scientifically_eligible_only: Option<u64>,
    max_simulated_days_scientifically_eligible_only: Option<u64>,
    living_condition_defined_runs_scientifically_eligible_only: u64,''',
                 "point exposure fields")

s = replace_once(s,
'''    mean_migration_total_distance_cells_scientifically_eligible_only: Option<f64>,
    pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only: Option<f64>,''',
'''    mean_migration_total_distance_cells_scientifically_eligible_only: Option<f64>,
    mean_births_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    mean_deaths_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    mean_resource_scarcity_deaths_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    mean_resource_unmet_need_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    mean_migration_moves_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    mean_migration_distance_cells_per_365_simulated_days_scientifically_eligible_only: Option<f64>,
    pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only: Option<f64>,''',
                 "point rate fields")

s = replace_once(s,
'''        note: "Descriptive analysis only. Point means pool provenance-valid scientific outcomes: durationReached and populationExtinct, while undefined denominator-based values remain null and are averaged only where defined. personRecordLimitReached is an operational censoring event and is excluded from scientific aggregates while remaining explicit in run-level outputs.",''',
'''        note: "Descriptive analysis only. Raw cumulative point means intentionally retain provenance-valid scientific outcomes (durationReached and populationExtinct) and must be interpreted jointly with realized simulatedDays and extinction frequency. Per-365-simulated-day rates normalize only by realized simulation time; they are not person-time, household-opportunity, or population-exposure rates. Zero-day and otherwise undefined denominator-based values remain null. personRecordLimitReached is operational censoring and is excluded from scientific aggregates while remaining explicit in run-level outputs.",''',
                 "summary note")

s = replace_once(s,
'''                migration_moves_completed: None,
                migration_total_distance_cells: None,
                migration_mean_origin_resource_score_permille: None,''',
'''                migration_moves_completed: None,
                migration_total_distance_cells: None,
                births_per_365_simulated_days: None,
                deaths_per_365_simulated_days: None,
                resource_scarcity_deaths_per_365_simulated_days: None,
                resource_unmet_need_per_365_simulated_days: None,
                migration_moves_per_365_simulated_days: None,
                migration_distance_cells_per_365_simulated_days: None,
                migration_mean_origin_resource_score_permille: None,''',
                 "run row init")

s = replace_once(s,
'''                row.migration_total_distance_cells =
                    Some(run_manifest.migration.total_distance_cells);
                row.migration_mean_origin_resource_score_permille =''',
'''                row.migration_total_distance_cells =
                    Some(run_manifest.migration.total_distance_cells);
                row.births_per_365_simulated_days = per_365_simulated_days(
                    row.births_since_start,
                    row.simulated_days,
                );
                row.deaths_per_365_simulated_days = per_365_simulated_days(
                    row.deaths_since_start,
                    row.simulated_days,
                );
                row.resource_scarcity_deaths_per_365_simulated_days = per_365_simulated_days(
                    row.resource_scarcity_deaths,
                    row.simulated_days,
                );
                row.resource_unmet_need_per_365_simulated_days = per_365_simulated_days(
                    row.resource_unmet_need,
                    row.simulated_days,
                );
                row.migration_moves_per_365_simulated_days = per_365_simulated_days(
                    row.migration_moves_completed,
                    row.simulated_days,
                );
                row.migration_distance_cells_per_365_simulated_days = per_365_simulated_days(
                    row.migration_total_distance_cells,
                    row.simulated_days,
                );
                row.migration_mean_origin_resource_score_permille =''',
                 "populate run rates")

s = replace_once(s,
'''            let scientifically_eligible_count = scientifically_eligible.len() as u64;
            let migration_moves = scientifically_eligible''',
'''            let scientifically_eligible_count = scientifically_eligible.len() as u64;
            let scientifically_eligible_simulated_days = scientifically_eligible
                .iter()
                .filter_map(|row| row.simulated_days)
                .collect::<Vec<_>>();
            let migration_moves = scientifically_eligible''',
                 "collect point exposure")

s = replace_once(s,
'''                scientifically_eligible_runs: scientifically_eligible_count,
                operationally_censored_runs: operationally_censored.len() as u64,
                living_condition_defined_runs_scientifically_eligible_only:''',
'''                scientifically_eligible_runs: scientifically_eligible_count,
                operationally_censored_runs: operationally_censored.len() as u64,
                population_extinction_fraction_scientifically_eligible_only:
                    if scientifically_eligible_count == 0 {
                        None
                    } else {
                        Some(population_extinct_runs as f64 / scientifically_eligible_count as f64)
                    },
                mean_simulated_days_scientifically_eligible_only: mean_u64(
                    scientifically_eligible_simulated_days.iter().copied(),
                ),
                min_simulated_days_scientifically_eligible_only:
                    scientifically_eligible_simulated_days.iter().copied().min(),
                max_simulated_days_scientifically_eligible_only:
                    scientifically_eligible_simulated_days.iter().copied().max(),
                living_condition_defined_runs_scientifically_eligible_only:''',
                 "populate point exposure")

s = replace_once(s,
'''                mean_migration_total_distance_cells_scientifically_eligible_only: mean_u64(
                    scientifically_eligible
                        .iter()
                        .filter_map(|row| row.migration_total_distance_cells),
                ),
                pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only:''',
'''                mean_migration_total_distance_cells_scientifically_eligible_only: mean_u64(
                    scientifically_eligible
                        .iter()
                        .filter_map(|row| row.migration_total_distance_cells),
                ),
                mean_births_per_365_simulated_days_scientifically_eligible_only: mean_f64(
                    scientifically_eligible
                        .iter()
                        .filter_map(|row| row.births_per_365_simulated_days),
                ),
                mean_deaths_per_365_simulated_days_scientifically_eligible_only: mean_f64(
                    scientifically_eligible
                        .iter()
                        .filter_map(|row| row.deaths_per_365_simulated_days),
                ),
                mean_resource_scarcity_deaths_per_365_simulated_days_scientifically_eligible_only:
                    mean_f64(scientifically_eligible.iter().filter_map(|row| {
                        row.resource_scarcity_deaths_per_365_simulated_days
                    })),
                mean_resource_unmet_need_per_365_simulated_days_scientifically_eligible_only:
                    mean_f64(scientifically_eligible.iter().filter_map(|row| {
                        row.resource_unmet_need_per_365_simulated_days
                    })),
                mean_migration_moves_per_365_simulated_days_scientifically_eligible_only:
                    mean_f64(scientifically_eligible.iter().filter_map(|row| {
                        row.migration_moves_per_365_simulated_days
                    })),
                mean_migration_distance_cells_per_365_simulated_days_scientifically_eligible_only:
                    mean_f64(scientifically_eligible.iter().filter_map(|row| {
                        row.migration_distance_cells_per_365_simulated_days
                    })),
                pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only:''',
                 "populate point rates")

s = replace_once(s,
'''fn mean_u64(values: impl Iterator<Item = u64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let total = values
        .iter()
        .fold(0_u128, |sum, value| sum + u128::from(*value));
    Some(total as f64 / values.len() as f64)
}

fn write_runs_csv''',
'''fn mean_u64(values: impl Iterator<Item = u64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let total = values
        .iter()
        .fold(0_u128, |sum, value| sum + u128::from(*value));
    Some(total as f64 / values.len() as f64)
}

fn mean_f64(values: impl Iterator<Item = f64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

fn per_365_simulated_days(value: Option<u64>, simulated_days: Option<u64>) -> Option<f64> {
    match (value, simulated_days) {
        (Some(value), Some(days)) if days > 0 => Some(value as f64 * 365.0 / days as f64),
        _ => None,
    }
}

fn write_runs_csv''',
                 "rate helpers")

s = replace_once(s,
'''condition_mortality_deaths,resource_unmet_need,migration_moves_completed,migration_total_distance_cells,migration_mean_origin_resource_score_permille''',
'''condition_mortality_deaths,resource_unmet_need,migration_moves_completed,migration_total_distance_cells,births_per_365_simulated_days,deaths_per_365_simulated_days,condition_mortality_deaths_per_365_simulated_days,resource_unmet_need_per_365_simulated_days,migration_moves_per_365_simulated_days,migration_distance_cells_per_365_simulated_days,migration_mean_origin_resource_score_permille''',
                 "runs csv header")

s = replace_once(s,
'''            optional_to_string(row.migration_moves_completed),
            optional_to_string(row.migration_total_distance_cells),
            optional_to_string(row.migration_mean_origin_resource_score_permille),''',
'''            optional_to_string(row.migration_moves_completed),
            optional_to_string(row.migration_total_distance_cells),
            optional_to_string(row.births_per_365_simulated_days),
            optional_to_string(row.deaths_per_365_simulated_days),
            optional_to_string(row.resource_scarcity_deaths_per_365_simulated_days),
            optional_to_string(row.resource_unmet_need_per_365_simulated_days),
            optional_to_string(row.migration_moves_per_365_simulated_days),
            optional_to_string(row.migration_distance_cells_per_365_simulated_days),
            optional_to_string(row.migration_mean_origin_resource_score_permille),''',
                 "runs csv values")

s = replace_once(s,
'''scientifically_eligible_runs,operationally_censored_runs,living_condition_defined_runs_scientifically_eligible_only''',
'''scientifically_eligible_runs,operationally_censored_runs,population_extinction_fraction_scientifically_eligible_only,mean_simulated_days_scientifically_eligible_only,min_simulated_days_scientifically_eligible_only,max_simulated_days_scientifically_eligible_only,living_condition_defined_runs_scientifically_eligible_only''',
                 "points csv exposure header")

s = replace_once(s,
'''mean_migration_moves_scientifically_eligible_only,mean_migration_total_distance_cells_scientifically_eligible_only,pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only''',
'''mean_migration_moves_scientifically_eligible_only,mean_migration_total_distance_cells_scientifically_eligible_only,mean_births_per_365_simulated_days_scientifically_eligible_only,mean_deaths_per_365_simulated_days_scientifically_eligible_only,mean_condition_mortality_deaths_per_365_simulated_days_scientifically_eligible_only,mean_resource_unmet_need_per_365_simulated_days_scientifically_eligible_only,mean_migration_moves_per_365_simulated_days_scientifically_eligible_only,mean_migration_distance_cells_per_365_simulated_days_scientifically_eligible_only,pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only''',
                 "points csv rates header")

s = replace_once(s,
'''            row.scientifically_eligible_runs.to_string(),
            row.operationally_censored_runs.to_string(),
            row.living_condition_defined_runs_scientifically_eligible_only''',
'''            row.scientifically_eligible_runs.to_string(),
            row.operationally_censored_runs.to_string(),
            optional_to_string(row.population_extinction_fraction_scientifically_eligible_only),
            optional_to_string(row.mean_simulated_days_scientifically_eligible_only),
            optional_to_string(row.min_simulated_days_scientifically_eligible_only),
            optional_to_string(row.max_simulated_days_scientifically_eligible_only),
            row.living_condition_defined_runs_scientifically_eligible_only''',
                 "points csv exposure values")

s = replace_once(s,
'''            row.mean_migration_total_distance_cells_scientifically_eligible_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only''',
'''            row.mean_migration_total_distance_cells_scientifically_eligible_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            optional_to_string(row.mean_births_per_365_simulated_days_scientifically_eligible_only),
            optional_to_string(row.mean_deaths_per_365_simulated_days_scientifically_eligible_only),
            optional_to_string(
                row.mean_resource_scarcity_deaths_per_365_simulated_days_scientifically_eligible_only,
            ),
            optional_to_string(
                row.mean_resource_unmet_need_per_365_simulated_days_scientifically_eligible_only,
            ),
            optional_to_string(
                row.mean_migration_moves_per_365_simulated_days_scientifically_eligible_only,
            ),
            optional_to_string(
                row.mean_migration_distance_cells_per_365_simulated_days_scientifically_eligible_only,
            ),
            row.pooled_mean_migration_distance_cells_per_move_scientifically_eligible_only''',
                 "points csv rate values")

s = replace_once(s,
'''            migration_moves_completed: None,
            migration_total_distance_cells: None,
            migration_mean_origin_resource_score_permille: None,''',
'''            migration_moves_completed: None,
            migration_total_distance_cells: None,
            births_per_365_simulated_days: None,
            deaths_per_365_simulated_days: None,
            resource_scarcity_deaths_per_365_simulated_days: None,
            resource_unmet_need_per_365_simulated_days: None,
            migration_moves_per_365_simulated_days: None,
            migration_distance_cells_per_365_simulated_days: None,
            migration_mean_origin_resource_score_permille: None,''',
                 "test helper init")

anchor = '''    #[test]\n    fn person_record_safety_ceiling_cannot_inject_truncated_run_into_scientific_mean() {'''
new_tests = r'''    #[test]
    fn exposure_rates_distinguish_short_intense_extinction_from_larger_long_run_total() {
        let dimensions = SweepDimensions {
            population: vec![12],
            household_size: vec![],
            resource_productivity_scale_permille: vec![],
            resource_seasonality_scale_permille: vec![],
            annual_food_need: vec![],
            disable_migration: vec![],
            migration_radius: vec![],
        };
        let sweep = build_sweep_manifest(small_settings(), vec![1, 2], dimensions).expect("sweep");
        let point = &sweep.points[0];
        let mut long = derived_row(
            &sweep,
            point,
            1,
            "completed",
            Some(StopReason::DurationReached),
            (Some(100), Some(50), Some(20)),
        );
        long.simulated_days = Some(3_650);
        long.end_day = Some(3_650);
        long.resource_unmet_need = Some(10_000);
        long.resource_scarcity_deaths = Some(10);
        long.migration_moves_completed = Some(20);
        long.migration_total_distance_cells = Some(100);
        long.births_per_365_simulated_days = per_365_simulated_days(long.births_since_start, long.simulated_days);
        long.deaths_per_365_simulated_days = per_365_simulated_days(long.deaths_since_start, long.simulated_days);
        long.resource_scarcity_deaths_per_365_simulated_days = per_365_simulated_days(long.resource_scarcity_deaths, long.simulated_days);
        long.resource_unmet_need_per_365_simulated_days = per_365_simulated_days(long.resource_unmet_need, long.simulated_days);
        long.migration_moves_per_365_simulated_days = per_365_simulated_days(long.migration_moves_completed, long.simulated_days);
        long.migration_distance_cells_per_365_simulated_days = per_365_simulated_days(long.migration_total_distance_cells, long.simulated_days);

        let mut extinct = derived_row(
            &sweep,
            point,
            2,
            "completed",
            Some(StopReason::PopulationExtinct),
            (Some(0), Some(20), Some(12)),
        );
        extinct.simulated_days = Some(365);
        extinct.end_day = Some(365);
        extinct.resource_unmet_need = Some(2_000);
        extinct.resource_scarcity_deaths = Some(5);
        extinct.migration_moves_completed = Some(8);
        extinct.migration_total_distance_cells = Some(60);
        extinct.births_per_365_simulated_days = per_365_simulated_days(extinct.births_since_start, extinct.simulated_days);
        extinct.deaths_per_365_simulated_days = per_365_simulated_days(extinct.deaths_since_start, extinct.simulated_days);
        extinct.resource_scarcity_deaths_per_365_simulated_days = per_365_simulated_days(extinct.resource_scarcity_deaths, extinct.simulated_days);
        extinct.resource_unmet_need_per_365_simulated_days = per_365_simulated_days(extinct.resource_unmet_need, extinct.simulated_days);
        extinct.migration_moves_per_365_simulated_days = per_365_simulated_days(extinct.migration_moves_completed, extinct.simulated_days);
        extinct.migration_distance_cells_per_365_simulated_days = per_365_simulated_days(extinct.migration_total_distance_cells, extinct.simulated_days);

        assert!(long.resource_unmet_need.unwrap() > extinct.resource_unmet_need.unwrap());
        assert!(
            long.resource_unmet_need_per_365_simulated_days.unwrap()
                < extinct.resource_unmet_need_per_365_simulated_days.unwrap()
        );

        let summary = &build_point_rows(&sweep, &[long, extinct])[0];
        assert_eq!(summary.population_extinct_runs, 1);
        assert_eq!(
            summary.population_extinction_fraction_scientifically_eligible_only,
            Some(0.5)
        );
        assert_eq!(summary.mean_simulated_days_scientifically_eligible_only, Some(2_007.5));
        assert_eq!(summary.min_simulated_days_scientifically_eligible_only, Some(365));
        assert_eq!(summary.max_simulated_days_scientifically_eligible_only, Some(3_650));
        assert_eq!(
            summary.mean_resource_unmet_need_scientifically_eligible_only,
            Some(6_000.0)
        );
        assert_eq!(
            summary.mean_resource_unmet_need_per_365_simulated_days_scientifically_eligible_only,
            Some(1_500.0)
        );
    }

    #[test]
    fn zero_simulated_day_rates_are_undefined_not_zero() {
        assert_eq!(per_365_simulated_days(Some(10), Some(0)), None);
        assert_eq!(per_365_simulated_days(Some(0), Some(0)), None);
        assert_eq!(per_365_simulated_days(Some(365), Some(365)), Some(365.0));
    }

'''
s = replace_once(s, anchor, new_tests + anchor, "new exposure tests")

sweep_path.write_text(s, encoding="utf-8")

m8_path = Path("scripts/aggregate-m8-spatial-benchmark.py")
m = m8_path.read_text(encoding="utf-8")
m = replace_once(m,
'''        if flat_value is None or arm_value is None:
            unavailable += 1
            pair["reason"] = "primary metric unavailable because one or both paired runs were not analyzable"
            pairs.append(pair)
            continue
''',
'''        if flat_record.get("terminalDegenerate") or arm_record.get("terminalDegenerate"):
            unavailable += 1
            pair["reason"] = (
                "fixed-horizon primary metric unavailable because one or both paired runs "
                "did not reach the declared duration"
            )
            pairs.append(pair)
            continue
        if flat_value is None or arm_value is None:
            unavailable += 1
            pair["reason"] = "primary metric unavailable because one or both paired runs were not analyzable"
            pairs.append(pair)
            continue
''', "M8 fixed-horizon eligibility")
m8_path.write_text(m, encoding="utf-8")
