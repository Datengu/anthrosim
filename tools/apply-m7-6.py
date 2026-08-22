#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path


def replace(path: str, old: str, new: str, expected: int = 1) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} occurrence(s), found {count}: {old[:80]!r}")
    file.write_text(text.replace(old, new), encoding="utf-8")


def sub(path: str, pattern: str, replacement: str, expected: int = 1) -> None:
    file = Path(path)
    text = file.read_text(encoding="utf-8")
    updated, count = re.subn(pattern, replacement, text, flags=re.S)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} regex replacement(s), found {count}: {pattern[:80]!r}")
    file.write_text(updated, encoding="utf-8")


# Version the exact experiment/resource/checkpoint/manifest shapes that gain the
# explicit temporal-variability control.
replace(
    "crates/anthrosim-core/src/config.rs",
    "impl ExperimentConfig {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 6;",
    "impl ExperimentConfig {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 7;",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    pub productivity_scale_permille: u16,\n    pub cell_stock_capacity_years: u16,",
    "    pub productivity_scale_permille: u16,\n    /// Scales the generated cell seasonal amplitude, 0..=1000.\n    /// 0 removes the seasonal swing; 1000 preserves the synthetic v0.1 baseline.\n    pub seasonality_scale_permille: u16,\n    pub cell_stock_capacity_years: u16,",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "impl ResourceConfig {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;",
    "impl ResourceConfig {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "            productivity_scale_permille: 1_000,\n            cell_stock_capacity_years: 10,",
    "            productivity_scale_permille: 1_000,\n            seasonality_scale_permille: 1_000,\n            cell_stock_capacity_years: 10,",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    pub const fn with_productivity_scale_permille(mut self, value: u16) -> Self {\n        self.productivity_scale_permille = value;\n        self\n    }\n\n    #[must_use]\n    pub const fn with_annual_need_units_per_person",
    "    pub const fn with_productivity_scale_permille(mut self, value: u16) -> Self {\n        self.productivity_scale_permille = value;\n        self\n    }\n\n    #[must_use]\n    pub const fn with_seasonality_scale_permille(mut self, value: u16) -> Self {\n        self.seasonality_scale_permille = value;\n        self\n    }\n\n    #[must_use]\n    pub const fn with_annual_need_units_per_person",
)
replace(
    "crates/anthrosim-core/src/manifest.rs",
    "impl RunManifest {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 7;",
    "impl RunManifest {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 8;",
)
replace(
    "crates/anthrosim-core/src/checkpoint.rs",
    "impl SimulationCheckpoint {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 1;",
    "impl SimulationCheckpoint {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;",
)

# Apply temporal variability in the resource process while keeping the same
# generated world across otherwise-equal sweep points.
replace(
    "crates/anthrosim-core/src/resources.rs",
    "            let seasonal = seasonal_factor_permille(\n                day_of_year,\n                cell.season_phase_days,\n                cell.season_amplitude,\n            );",
    "            let seasonal = scaled_seasonal_factor_permille(\n                day_of_year,\n                cell.season_phase_days,\n                cell.season_amplitude,\n                config.seasonality_scale_permille,\n            );",
)
replace(
    "crates/anthrosim-core/src/resources.rs",
    "    if config.productivity_scale_permille > PERMILLE_MAX {\n        return Err(ResourceConfigError::InvalidProductivityScale {\n            value: config.productivity_scale_permille,\n        });\n    }",
    "    if config.productivity_scale_permille > PERMILLE_MAX {\n        return Err(ResourceConfigError::InvalidProductivityScale {\n            value: config.productivity_scale_permille,\n        });\n    }\n    if config.seasonality_scale_permille > PERMILLE_MAX {\n        return Err(ResourceConfigError::InvalidSeasonalityScale {\n            value: config.seasonality_scale_permille,\n        });\n    }",
)
replace(
    "crates/anthrosim-core/src/resources.rs",
    "    #[error(\"resource productivity scale {value} permille is outside 0..=1000\")]\n    InvalidProductivityScale { value: u16 },",
    "    #[error(\"resource productivity scale {value} permille is outside 0..=1000\")]\n    InvalidProductivityScale { value: u16 },\n    #[error(\"resource seasonality scale {value} permille is outside 0..=1000\")]\n    InvalidSeasonalityScale { value: u16 },",
)
replace(
    "crates/anthrosim-core/src/resources.rs",
    "/// Integer triangular seasonal factor centred on a cell's phase.\n///\n/// A zero amplitude returns 1000.",
    "fn scaled_seasonal_factor_permille(\n    day_of_year: u16,\n    phase: u16,\n    cell_amplitude: u16,\n    seasonality_scale_permille: u16,\n) -> u16 {\n    let scaled_amplitude = scale_permille(\n        u64::from(cell_amplitude),\n        seasonality_scale_permille,\n    );\n    seasonal_factor_permille(\n        day_of_year,\n        phase,\n        u16::try_from(scaled_amplitude).unwrap_or(PERMILLE_MAX),\n    )\n}\n\n/// Integer triangular seasonal factor centred on a cell's phase.\n///\n/// A zero amplitude returns 1000.",
)
replace(
    "crates/anthrosim-core/src/resources.rs",
    "    fn seasonal_factor_has_expected_peak_and_trough() {\n        assert_eq!(seasonal_factor_permille(0, 0, 1_000), 2_000);\n        assert_eq!(seasonal_factor_permille(182, 0, 1_000), 0);\n        assert_eq!(seasonal_factor_permille(100, 100, 0), 1_000);\n    }",
    "    fn seasonal_factor_has_expected_peak_and_trough() {\n        assert_eq!(seasonal_factor_permille(0, 0, 1_000), 2_000);\n        assert_eq!(seasonal_factor_permille(182, 0, 1_000), 0);\n        assert_eq!(seasonal_factor_permille(100, 100, 0), 1_000);\n        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 0), 1_000);\n        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 500), 1_400);\n        assert_eq!(scaled_seasonal_factor_permille(0, 0, 800, 1_000), 1_800);\n\n        let invalid = ResourceConfig::synthetic_validation_v1()\n            .with_seasonality_scale_permille(1_001);\n        assert!(matches!(\n            validate_resource_config(&invalid),\n            Err(ResourceConfigError::InvalidSeasonalityScale { .. })\n        ));\n    }",
)

# Thread the new resource control through the existing ordinary run/ensemble path.
replace(
    "crates/anthrosim-cli/src/ensemble.rs",
    "    pub(crate) resource_productivity_scale_permille: u16,\n    pub(crate) annual_food_need: u32,",
    "    pub(crate) resource_productivity_scale_permille: u16,\n    pub(crate) resource_seasonality_scale_permille: u16,\n    pub(crate) annual_food_need: u32,",
)
replace(
    "crates/anthrosim-cli/src/ensemble.rs",
    "    let resources = ResourceConfig::synthetic_validation_v1()\n        .with_productivity_scale_permille(settings.resource_productivity_scale_permille)\n        .with_annual_need_units_per_person(settings.annual_food_need);",
    "    let resources = ResourceConfig::synthetic_validation_v1()\n        .with_productivity_scale_permille(settings.resource_productivity_scale_permille)\n        .with_seasonality_scale_permille(settings.resource_seasonality_scale_permille)\n        .with_annual_need_units_per_person(settings.annual_food_need);",
)
replace(
    "crates/anthrosim-cli/src/ensemble.rs",
    "            resource_productivity_scale_permille: 1_000,\n            annual_food_need: 100,",
    "            resource_productivity_scale_permille: 1_000,\n            resource_seasonality_scale_permille: 1_000,\n            annual_food_need: 100,",
)

# CLI declarations: run, ensemble and sweep all share the same base control.
replace(
    "crates/anthrosim-cli/src/main.rs",
    "        #[arg(long, default_value_t = 1_000)]\n        resource_productivity_scale_permille: u16,",
    "        #[arg(long, default_value_t = 1_000)]\n        resource_productivity_scale_permille: u16,\n\n        /// Synthetic seasonal-amplitude scale for renewable productivity, in permille (0..=1000).\n        #[arg(long, default_value_t = 1_000)]\n        resource_seasonality_scale_permille: u16,",
    expected=3,
)
replace(
    "crates/anthrosim-cli/src/main.rs",
    "        /// Explicit annual-resource-need values for the Cartesian parameter grid.\n        #[arg(long, value_delimiter = ',', num_args = 1..)]\n        sweep_annual_food_need: Vec<u32>,",
    "        /// Explicit seasonal-amplitude scales for the Cartesian parameter grid.\n        #[arg(long, value_delimiter = ',', num_args = 1..)]\n        sweep_resource_seasonality_scale_permille: Vec<u16>,\n\n        /// Explicit annual-resource-need values for the Cartesian parameter grid.\n        #[arg(long, value_delimiter = ',', num_args = 1..)]\n        sweep_annual_food_need: Vec<u32>,",
)
replace(
    "crates/anthrosim-cli/src/main.rs",
    "            resource_productivity_scale_permille,\n            annual_food_need,",
    "            resource_productivity_scale_permille,\n            resource_seasonality_scale_permille,\n            annual_food_need,",
    expected=6,
)
replace(
    "crates/anthrosim-cli/src/main.rs",
    "            sweep_resource_productivity_scale_permille,\n            sweep_annual_food_need,",
    "            sweep_resource_productivity_scale_permille,\n            sweep_resource_seasonality_scale_permille,\n            sweep_annual_food_need,",
)
replace(
    "crates/anthrosim-cli/src/main.rs",
    "                resource_productivity_scale_permille: sweep_resource_productivity_scale_permille,\n                annual_food_need: sweep_annual_food_need,",
    "                resource_productivity_scale_permille: sweep_resource_productivity_scale_permille,\n                resource_seasonality_scale_permille: sweep_resource_seasonality_scale_permille,\n                annual_food_need: sweep_annual_food_need,",
)

# Sweep manifest/analysis formats gain a new exact dimension and observables.
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "const SWEEP_MANIFEST_SCHEMA_VERSION: u32 = 1;\nconst DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 1;",
    "const SWEEP_MANIFEST_SCHEMA_VERSION: u32 = 2;\nconst DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 2;",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "    pub(crate) resource_productivity_scale_permille: Vec<u16>,\n    pub(crate) annual_food_need: Vec<u32>,",
    "    pub(crate) resource_productivity_scale_permille: Vec<u16>,\n    pub(crate) resource_seasonality_scale_permille: Vec<u16>,\n    pub(crate) annual_food_need: Vec<u32>,",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "            || !self.resource_productivity_scale_permille.is_empty()\n            || !self.annual_food_need.is_empty()",
    "            || !self.resource_productivity_scale_permille.is_empty()\n            || !self.resource_seasonality_scale_permille.is_empty()\n            || !self.annual_food_need.is_empty()",
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"#\[derive\(Debug, Clone, Serialize, Deserialize\)\]\n#\[serde\(rename_all = \"camelCase\"\)\]\nstruct DerivedRunRow \{.*?\n\}\n\n#\[derive\(Debug, Clone, Serialize, Deserialize\)\]\n#\[serde\(rename_all = \"camelCase\"\)\]\nstruct DerivedPointRow \{.*?\n\}",
    '''#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedRunRow {
    schema_version: u32,
    provenance: String,
    sweep_id: String,
    point_id: String,
    experiment_id: Option<String>,
    run_id: String,
    seed: u64,
    state: String,
    attempt: u32,
    status_relative_path: String,
    manifest_relative_path: Option<String>,
    world_width: u32,
    world_height: u32,
    initial_population: u32,
    household_size: u16,
    max_person_records: u64,
    resource_productivity_scale_permille: u16,
    resource_seasonality_scale_permille: u16,
    annual_food_need: u32,
    disable_migration: bool,
    migration_radius: u16,
    stop_reason: Option<String>,
    state_digest64: Option<u64>,
    final_living_population: Option<u64>,
    births_since_start: Option<u64>,
    deaths_since_start: Option<u64>,
    household_count: Option<u64>,
    mean_living_condition_permille: Option<u16>,
    authoritative_event_count: Option<u64>,
    final_living_occupied_cell_count: Option<u64>,
    resource_scarcity_deaths: Option<u64>,
    resource_unmet_need: Option<u64>,
    migration_moves_completed: Option<u64>,
    migration_total_distance_cells: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedPointRow {
    schema_version: u32,
    provenance: String,
    sweep_id: String,
    point_id: String,
    experiment_id: Option<String>,
    initial_population: u32,
    resource_productivity_scale_permille: u16,
    resource_seasonality_scale_permille: u16,
    disable_migration: bool,
    migration_radius: u16,
    planned_runs: u64,
    completed_runs: u64,
    failed_runs: u64,
    incomplete_runs: u64,
    other_non_completed_runs: u64,
    duration_reached_runs: u64,
    population_extinct_runs: u64,
    person_record_limit_reached_runs: u64,
    mean_final_living_population_completed_only: Option<f64>,
    mean_final_living_occupied_cell_count_completed_only: Option<f64>,
    mean_births_since_start_completed_only: Option<f64>,
    mean_deaths_since_start_completed_only: Option<f64>,
    mean_living_condition_permille_completed_only: Option<f64>,
    mean_resource_scarcity_deaths_completed_only: Option<f64>,
    mean_resource_unmet_need_completed_only: Option<f64>,
    mean_migration_moves_completed_only: Option<f64>,
    mean_migration_total_distance_cells_completed_only: Option<f64>,
    pooled_mean_migration_distance_cells_per_move_completed_only: Option<f64>,
    source_completed_run_ids: Vec<String>,
}''',
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "    validate_unique(\n        \"resource-productivity-scale-permille\",\n        &dimensions.resource_productivity_scale_permille,\n    )?;\n    validate_unique(\"annual-food-need\", &dimensions.annual_food_need)?;",
    "    validate_unique(\n        \"resource-productivity-scale-permille\",\n        &dimensions.resource_productivity_scale_permille,\n    )?;\n    validate_unique(\n        \"resource-seasonality-scale-permille\",\n        &dimensions.resource_seasonality_scale_permille,\n    )?;\n    validate_unique(\"annual-food-need\", &dimensions.annual_food_need)?;",
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"fn expand_sweep_points\(\n    base: &EnsembleRunSettings,\n    dimensions: &SweepDimensions,\n\) -> Result<Vec<SweepPoint>, io::Error> \{.*?\n\}\n\nfn values_or_base",
    '''fn expand_sweep_points(
    base: &EnsembleRunSettings,
    dimensions: &SweepDimensions,
) -> Result<Vec<SweepPoint>, io::Error> {
    let populations = values_or_base(&dimensions.population, base.population);
    let household_sizes = values_or_base(&dimensions.household_size, base.household_size);
    let productivities = values_or_base(
        &dimensions.resource_productivity_scale_permille,
        base.resource_productivity_scale_permille,
    );
    let seasonalities = values_or_base(
        &dimensions.resource_seasonality_scale_permille,
        base.resource_seasonality_scale_permille,
    );
    let annual_food_needs = values_or_base(&dimensions.annual_food_need, base.annual_food_need);
    let migration_switches = values_or_base(&dimensions.disable_migration, base.disable_migration);
    let migration_radii = values_or_base(&dimensions.migration_radius, base.migration_radius);

    let point_count = [
        populations.len(),
        household_sizes.len(),
        productivities.len(),
        seasonalities.len(),
        annual_food_needs.len(),
        migration_switches.len(),
        migration_radii.len(),
    ]
    .into_iter()
    .try_fold(1_usize, |total, count| total.checked_mul(count))
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "sweep grid size overflow"))?;
    if point_count > MAX_SWEEP_POINTS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "sweep expands to {point_count} parameter points; maximum supported planning size is {MAX_SWEEP_POINTS}"
            ),
        ));
    }

    let mut points = Vec::with_capacity(point_count);
    for &population in &populations {
        for &household_size in &household_sizes {
            for &resource_productivity_scale_permille in &productivities {
                for &resource_seasonality_scale_permille in &seasonalities {
                    for &annual_food_need in &annual_food_needs {
                        for &disable_migration in &migration_switches {
                            for &migration_radius in &migration_radii {
                                let index = points.len();
                                let point_id = format!("point-{index:06}");
                                let mut settings = base.clone();
                                settings.population = population;
                                settings.household_size = household_size;
                                settings.resource_productivity_scale_permille =
                                    resource_productivity_scale_permille;
                                settings.resource_seasonality_scale_permille =
                                    resource_seasonality_scale_permille;
                                settings.annual_food_need = annual_food_need;
                                settings.disable_migration = disable_migration;
                                settings.migration_radius = migration_radius;
                                points.push(SweepPoint {
                                    relative_experiment_dir: format!("experiments/{point_id}"),
                                    point_id,
                                    settings,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(points)
}

fn values_or_base''',
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "                resource_productivity_scale_permille: point\n                    .settings\n                    .resource_productivity_scale_permille,\n                annual_food_need: point.settings.annual_food_need,",
    "                resource_productivity_scale_permille: point\n                    .settings\n                    .resource_productivity_scale_permille,\n                resource_seasonality_scale_permille: point\n                    .settings\n                    .resource_seasonality_scale_permille,\n                annual_food_need: point.settings.annual_food_need,",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "                authoritative_event_count: None,\n            };",
    "                authoritative_event_count: None,\n                final_living_occupied_cell_count: None,\n                resource_scarcity_deaths: None,\n                resource_unmet_need: None,\n                migration_moves_completed: None,\n                migration_total_distance_cells: None,\n            };",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "                    || run_manifest\n                        .experiment\n                        .resources\n                        .annual_need_units_per_person\n                        != point.settings.annual_food_need",
    "                    || run_manifest\n                        .experiment\n                        .resources\n                        .seasonality_scale_permille\n                        != point.settings.resource_seasonality_scale_permille\n                    || run_manifest\n                        .experiment\n                        .resources\n                        .annual_need_units_per_person\n                        != point.settings.annual_food_need",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "                row.authoritative_event_count =\n                    Some(run_manifest.statistics.authoritative_event_count);",
    "                row.authoritative_event_count =\n                    Some(run_manifest.statistics.authoritative_event_count);\n                row.final_living_occupied_cell_count =\n                    Some(run_manifest.population.living_occupied_cell_count);\n                row.resource_scarcity_deaths = Some(run_manifest.resources.scarcity_deaths);\n                row.resource_unmet_need = Some(run_manifest.resources.unmet_need);\n                row.migration_moves_completed = Some(run_manifest.migration.moves_completed);\n                row.migration_total_distance_cells =\n                    Some(run_manifest.migration.total_distance_cells);",
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"fn build_point_rows\(sweep: &SweepManifest, runs: &\[DerivedRunRow\]\) -> Vec<DerivedPointRow> \{.*?\n\}\n\nfn mean_u64",
    '''fn build_point_rows(sweep: &SweepManifest, runs: &[DerivedRunRow]) -> Vec<DerivedPointRow> {
    sweep
        .points
        .iter()
        .map(|point| {
            let point_runs = runs
                .iter()
                .filter(|row| row.point_id == point.point_id)
                .collect::<Vec<_>>();
            let completed = point_runs
                .iter()
                .copied()
                .filter(|row| row.state == "completed")
                .collect::<Vec<_>>();
            let completed_runs = completed.len() as u64;
            let failed_runs = point_runs.iter().filter(|row| row.state == "failed").count() as u64;
            let incomplete_runs = point_runs
                .iter()
                .filter(|row| row.state == "incomplete")
                .count() as u64;
            let other_non_completed_runs =
                point_runs.len() as u64 - completed_runs - failed_runs - incomplete_runs;
            let duration_reached_runs = completed
                .iter()
                .filter(|row| row.stop_reason.as_deref() == Some("durationReached"))
                .count() as u64;
            let population_extinct_runs = completed
                .iter()
                .filter(|row| row.stop_reason.as_deref() == Some("populationExtinct"))
                .count() as u64;
            let person_record_limit_reached_runs = completed
                .iter()
                .filter(|row| row.stop_reason.as_deref() == Some("personRecordLimitReached"))
                .count() as u64;
            let migration_moves = completed
                .iter()
                .filter_map(|row| row.migration_moves_completed)
                .fold(0_u128, |sum, value| sum + u128::from(value));
            let migration_distance = completed
                .iter()
                .filter_map(|row| row.migration_total_distance_cells)
                .fold(0_u128, |sum, value| sum + u128::from(value));

            DerivedPointRow {
                schema_version: DERIVED_ANALYSIS_SCHEMA_VERSION,
                provenance: "derived".to_owned(),
                sweep_id: sweep.sweep_id.clone(),
                point_id: point.point_id.clone(),
                experiment_id: point_runs.first().and_then(|row| row.experiment_id.clone()),
                initial_population: point.settings.population,
                resource_productivity_scale_permille: point
                    .settings
                    .resource_productivity_scale_permille,
                resource_seasonality_scale_permille: point
                    .settings
                    .resource_seasonality_scale_permille,
                disable_migration: point.settings.disable_migration,
                migration_radius: point.settings.migration_radius,
                planned_runs: point_runs.len() as u64,
                completed_runs,
                failed_runs,
                incomplete_runs,
                other_non_completed_runs,
                duration_reached_runs,
                population_extinct_runs,
                person_record_limit_reached_runs,
                mean_final_living_population_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.final_living_population),
                ),
                mean_final_living_occupied_cell_count_completed_only: mean_u64(
                    completed
                        .iter()
                        .filter_map(|row| row.final_living_occupied_cell_count),
                ),
                mean_births_since_start_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.births_since_start),
                ),
                mean_deaths_since_start_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.deaths_since_start),
                ),
                mean_living_condition_permille_completed_only: mean_u64(
                    completed
                        .iter()
                        .filter_map(|row| row.mean_living_condition_permille.map(u64::from)),
                ),
                mean_resource_scarcity_deaths_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.resource_scarcity_deaths),
                ),
                mean_resource_unmet_need_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.resource_unmet_need),
                ),
                mean_migration_moves_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.migration_moves_completed),
                ),
                mean_migration_total_distance_cells_completed_only: mean_u64(
                    completed
                        .iter()
                        .filter_map(|row| row.migration_total_distance_cells),
                ),
                pooled_mean_migration_distance_cells_per_move_completed_only: if migration_moves
                    == 0
                {
                    None
                } else {
                    Some(migration_distance as f64 / migration_moves as f64)
                },
                source_completed_run_ids: completed
                    .iter()
                    .map(|row| format!("{}/{}", row.point_id, row.run_id))
                    .collect(),
            }
        })
        .collect()
}

fn mean_u64''',
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"fn write_runs_csv\(path: &Path, rows: &\[DerivedRunRow\]\) -> Result<\(\), io::Error> \{.*?\n\}\n\nfn write_points_csv",
    '''fn write_runs_csv(path: &Path, rows: &[DerivedRunRow]) -> Result<(), io::Error> {
    let mut csv = String::from(
        "sweep_id,point_id,experiment_id,run_id,seed,state,attempt,status_relative_path,manifest_relative_path,world_width,world_height,initial_population,household_size,max_person_records,resource_productivity_scale_permille,resource_seasonality_scale_permille,annual_food_need,disable_migration,migration_radius,stop_reason,state_digest64,final_living_population,births_since_start,deaths_since_start,household_count,mean_living_condition_permille,authoritative_event_count,final_living_occupied_cell_count,resource_scarcity_deaths,resource_unmet_need,migration_moves_completed,migration_total_distance_cells\\n",
    );
    for row in rows {
        csv.push_str(&csv_line(&[
            row.sweep_id.clone(),
            row.point_id.clone(),
            row.experiment_id.clone().unwrap_or_default(),
            row.run_id.clone(),
            row.seed.to_string(),
            row.state.clone(),
            row.attempt.to_string(),
            row.status_relative_path.clone(),
            row.manifest_relative_path.clone().unwrap_or_default(),
            row.world_width.to_string(),
            row.world_height.to_string(),
            row.initial_population.to_string(),
            row.household_size.to_string(),
            row.max_person_records.to_string(),
            row.resource_productivity_scale_permille.to_string(),
            row.resource_seasonality_scale_permille.to_string(),
            row.annual_food_need.to_string(),
            row.disable_migration.to_string(),
            row.migration_radius.to_string(),
            row.stop_reason.clone().unwrap_or_default(),
            optional_to_string(row.state_digest64),
            optional_to_string(row.final_living_population),
            optional_to_string(row.births_since_start),
            optional_to_string(row.deaths_since_start),
            optional_to_string(row.household_count),
            optional_to_string(row.mean_living_condition_permille),
            optional_to_string(row.authoritative_event_count),
            optional_to_string(row.final_living_occupied_cell_count),
            optional_to_string(row.resource_scarcity_deaths),
            optional_to_string(row.resource_unmet_need),
            optional_to_string(row.migration_moves_completed),
            optional_to_string(row.migration_total_distance_cells),
        ]));
    }
    fs::write(path, csv)
}

fn write_points_csv''',
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"fn write_points_csv\(path: &Path, rows: &\[DerivedPointRow\]\) -> Result<\(\), io::Error> \{.*?\n\}\n\nfn csv_line",
    '''fn write_points_csv(path: &Path, rows: &[DerivedPointRow]) -> Result<(), io::Error> {
    let mut csv = String::from(
        "sweep_id,point_id,experiment_id,initial_population,resource_productivity_scale_permille,resource_seasonality_scale_permille,disable_migration,migration_radius,planned_runs,completed_runs,failed_runs,incomplete_runs,other_non_completed_runs,duration_reached_runs,population_extinct_runs,person_record_limit_reached_runs,mean_final_living_population_completed_only,mean_final_living_occupied_cell_count_completed_only,mean_births_since_start_completed_only,mean_deaths_since_start_completed_only,mean_living_condition_permille_completed_only,mean_resource_scarcity_deaths_completed_only,mean_resource_unmet_need_completed_only,mean_migration_moves_completed_only,mean_migration_total_distance_cells_completed_only,pooled_mean_migration_distance_cells_per_move_completed_only,source_completed_run_ids\\n",
    );
    for row in rows {
        csv.push_str(&csv_line(&[
            row.sweep_id.clone(),
            row.point_id.clone(),
            row.experiment_id.clone().unwrap_or_default(),
            row.initial_population.to_string(),
            row.resource_productivity_scale_permille.to_string(),
            row.resource_seasonality_scale_permille.to_string(),
            row.disable_migration.to_string(),
            row.migration_radius.to_string(),
            row.planned_runs.to_string(),
            row.completed_runs.to_string(),
            row.failed_runs.to_string(),
            row.incomplete_runs.to_string(),
            row.other_non_completed_runs.to_string(),
            row.duration_reached_runs.to_string(),
            row.population_extinct_runs.to_string(),
            row.person_record_limit_reached_runs.to_string(),
            row.mean_final_living_population_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_final_living_occupied_cell_count_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_births_since_start_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_deaths_since_start_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_living_condition_permille_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_resource_scarcity_deaths_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_resource_unmet_need_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_migration_moves_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_migration_total_distance_cells_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.pooled_mean_migration_distance_cells_per_move_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.source_completed_run_ids.join("|"),
        ]));
    }
    fs::write(path, csv)
}

fn csv_line''',
)

# Test constructors and all SweepDimensions literals need the newly-required field.
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "            resource_productivity_scale_permille: 1_000,\n            annual_food_need: 100,",
    "            resource_productivity_scale_permille: 1_000,\n            resource_seasonality_scale_permille: 1_000,\n            annual_food_need: 100,",
)
sub(
    "crates/anthrosim-cli/src/sweep.rs",
    r"(            resource_productivity_scale_permille: vec!\[[^\n]*\],\n)(            annual_food_need:)",
    r"\1            resource_seasonality_scale_permille: vec![],\n\2",
    expected=5,
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "            resource_productivity_scale_permille: point\n                .settings\n                .resource_productivity_scale_permille,\n            annual_food_need: point.settings.annual_food_need,",
    "            resource_productivity_scale_permille: point\n                .settings\n                .resource_productivity_scale_permille,\n            resource_seasonality_scale_permille: point\n                .settings\n                .resource_seasonality_scale_permille,\n            annual_food_need: point.settings.annual_food_need,",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "            authoritative_event_count: None,\n        }",
    "            authoritative_event_count: None,\n            final_living_occupied_cell_count: None,\n            resource_scarcity_deaths: None,\n            resource_unmet_need: None,\n            migration_moves_completed: None,\n            migration_total_distance_cells: None,\n        }",
)
replace(
    "crates/anthrosim-cli/src/sweep.rs",
    "    #[test]\n    fn duplicate_dimension_values_are_rejected() {",
    "    #[test]\n    fn seasonality_dimension_is_part_of_deterministic_sweep_identity() {\n        let mut seasonal = dimensions();\n        seasonal.resource_productivity_scale_permille = vec![1_000];\n        seasonal.annual_food_need.clear();\n        seasonal.resource_seasonality_scale_permille = vec![0, 500, 1_000];\n        let points = expand_sweep_points(&small_settings(), &seasonal).expect(\"points\");\n        assert_eq!(points.len(), 3);\n        assert_eq!(points[0].settings.resource_seasonality_scale_permille, 0);\n        assert_eq!(points[1].settings.resource_seasonality_scale_permille, 500);\n        assert_eq!(points[2].settings.resource_seasonality_scale_permille, 1_000);\n        let first = build_sweep_manifest(small_settings(), vec![3, 7], seasonal.clone())\n            .expect(\"sweep\");\n        seasonal.resource_seasonality_scale_permille = vec![0, 1_000];\n        let changed = build_sweep_manifest(small_settings(), vec![3, 7], seasonal)\n            .expect(\"sweep\");\n        assert_ne!(first.sweep_id, changed.sweep_id);\n    }\n\n    #[test]\n    fn duplicate_dimension_values_are_rejected() {",
)

print("M7.6 source transformations applied")
