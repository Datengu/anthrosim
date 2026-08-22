use std::{
    collections::HashSet,
    fmt::Display,
    fs,
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

use anthrosim_core::RunManifest;
use serde::{Deserialize, Serialize};

use crate::{
    ensemble::{EnsembleRunSettings, execute_ensemble},
    read_json, write_json,
};

const SWEEP_MANIFEST_SCHEMA_VERSION: u32 = 1;
const DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 1;
const MAX_SWEEP_POINTS: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SweepDimensions {
    pub(crate) population: Vec<u32>,
    pub(crate) household_size: Vec<u16>,
    pub(crate) resource_productivity_scale_permille: Vec<u16>,
    pub(crate) annual_food_need: Vec<u32>,
    pub(crate) disable_migration: Vec<bool>,
    pub(crate) migration_radius: Vec<u16>,
}

impl SweepDimensions {
    fn has_any_dimension(&self) -> bool {
        !self.population.is_empty()
            || !self.household_size.is_empty()
            || !self.resource_productivity_scale_permille.is_empty()
            || !self.annual_food_need.is_empty()
            || !self.disable_migration.is_empty()
            || !self.migration_radius.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SweepDefinition {
    seeds: Vec<u64>,
    base_settings: EnsembleRunSettings,
    dimensions: SweepDimensions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SweepPoint {
    point_id: String,
    relative_experiment_dir: String,
    settings: EnsembleRunSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SweepManifest {
    schema_version: u32,
    sweep_id: String,
    model_version: String,
    git_commit: Option<String>,
    definition: SweepDefinition,
    points: Vec<SweepPoint>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SweepIdentity<'a> {
    schema_version: u32,
    model_version: &'a str,
    git_commit: &'a Option<String>,
    definition: &'a SweepDefinition,
    points: &'a [SweepPoint],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedRunRow {
    schema_version: u32,
    provenance: &'static str,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DerivedPointRow {
    schema_version: u32,
    provenance: &'static str,
    sweep_id: String,
    point_id: String,
    experiment_id: Option<String>,
    planned_runs: u64,
    completed_runs: u64,
    failed_runs: u64,
    incomplete_runs: u64,
    other_non_completed_runs: u64,
    mean_final_living_population_completed_only: Option<f64>,
    mean_births_since_start_completed_only: Option<f64>,
    mean_deaths_since_start_completed_only: Option<f64>,
    source_completed_run_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisSummary {
    schema_version: u32,
    provenance: &'static str,
    sweep_id: String,
    run_rows: usize,
    point_rows: usize,
    completed_runs: usize,
    non_completed_runs: usize,
    note: &'static str,
}

pub(crate) fn execute_sweep(
    directory: &Path,
    base_settings: EnsembleRunSettings,
    seeds: Vec<u64>,
    dimensions: SweepDimensions,
    retry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = build_sweep_manifest(base_settings, seeds, dimensions)?;
    let manifest = if retry {
        load_matching_sweep_manifest(directory, &expected)?
    } else {
        initialize_sweep(directory, &expected)?;
        expected
    };

    let mut unsuccessful_points = 0_u64;
    for point in &manifest.points {
        let point_directory = directory.join(&point.relative_experiment_dir);
        let point_has_manifest = point_directory.join("experiment-manifest.json").is_file();
        let point_retry = retry && point_has_manifest;
        if let Err(error) = execute_ensemble(
            &point_directory,
            point.settings.clone(),
            manifest.definition.seeds.clone(),
            point_retry,
        ) {
            unsuccessful_points = unsuccessful_points.saturating_add(1);
            eprintln!(
                "sweep {} point {} did not fully complete: {error}",
                manifest.sweep_id, point.point_id
            );
        }
    }

    write_analysis_outputs(directory, &manifest)?;

    if unsuccessful_points > 0 {
        return Err(io::Error::other(format!(
            "sweep finished with {unsuccessful_points} point(s) containing unsuccessful runs; derived analysis explicitly records their non-completed states and the exact sweep can be retried with --retry"
        ))
        .into());
    }

    println!(
        "completed sweep {} with {} parameter point(s) in {}",
        manifest.sweep_id,
        manifest.points.len(),
        directory.display()
    );
    Ok(())
}

fn build_sweep_manifest(
    base_settings: EnsembleRunSettings,
    seeds: Vec<u64>,
    dimensions: SweepDimensions,
) -> Result<SweepManifest, Box<dyn std::error::Error>> {
    if seeds.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sweep requires at least one seed",
        )
        .into());
    }
    validate_unique("seed", &seeds)?;
    validate_dimensions(&dimensions)?;
    if !dimensions.has_any_dimension() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sweep requires at least one --sweep-* dimension; use ensemble for seed-only variation",
        )
        .into());
    }

    let points = expand_sweep_points(&base_settings, &dimensions)?;
    let definition = SweepDefinition {
        seeds,
        base_settings,
        dimensions,
    };
    let model_version = env!("CARGO_PKG_VERSION").to_owned();
    let git_commit = option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned);
    let identity = SweepIdentity {
        schema_version: SWEEP_MANIFEST_SCHEMA_VERSION,
        model_version: &model_version,
        git_commit: &git_commit,
        definition: &definition,
        points: &points,
    };
    let identity_bytes = serde_json::to_vec(&identity)?;
    let sweep_id = format!(
        "anthrosim-sweep-v{SWEEP_MANIFEST_SCHEMA_VERSION}-{:016x}",
        fnv1a64(&identity_bytes)
    );

    Ok(SweepManifest {
        schema_version: SWEEP_MANIFEST_SCHEMA_VERSION,
        sweep_id,
        model_version,
        git_commit,
        definition,
        points,
    })
}

fn validate_dimensions(dimensions: &SweepDimensions) -> Result<(), io::Error> {
    validate_unique("population", &dimensions.population)?;
    validate_unique("household-size", &dimensions.household_size)?;
    validate_unique(
        "resource-productivity-scale-permille",
        &dimensions.resource_productivity_scale_permille,
    )?;
    validate_unique("annual-food-need", &dimensions.annual_food_need)?;
    validate_unique("disable-migration", &dimensions.disable_migration)?;
    validate_unique("migration-radius", &dimensions.migration_radius)?;
    Ok(())
}

fn validate_unique<T>(name: &str, values: &[T]) -> Result<(), io::Error>
where
    T: Copy + Eq + Hash + Display,
{
    let mut seen = HashSet::with_capacity(values.len());
    for &value in values {
        if !seen.insert(value) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("duplicate value {value} in sweep dimension {name}"),
            ));
        }
    }
    Ok(())
}

fn expand_sweep_points(
    base: &EnsembleRunSettings,
    dimensions: &SweepDimensions,
) -> Result<Vec<SweepPoint>, io::Error> {
    let populations = values_or_base(&dimensions.population, base.population);
    let household_sizes = values_or_base(&dimensions.household_size, base.household_size);
    let productivities = values_or_base(
        &dimensions.resource_productivity_scale_permille,
        base.resource_productivity_scale_permille,
    );
    let annual_food_needs = values_or_base(&dimensions.annual_food_need, base.annual_food_need);
    let migration_switches = values_or_base(&dimensions.disable_migration, base.disable_migration);
    let migration_radii = values_or_base(&dimensions.migration_radius, base.migration_radius);

    let point_count = [
        populations.len(),
        household_sizes.len(),
        productivities.len(),
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
    Ok(points)
}

fn values_or_base<T: Copy>(values: &[T], base: T) -> Vec<T> {
    if values.is_empty() {
        vec![base]
    } else {
        values.to_vec()
    }
}

fn initialize_sweep(
    directory: &Path,
    manifest: &SweepManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    require_empty_directory(directory)?;
    fs::create_dir_all(directory)?;
    write_json(&directory.join("sweep-manifest.json"), manifest)?;
    Ok(())
}

fn load_matching_sweep_manifest(
    directory: &Path,
    expected: &SweepManifest,
) -> Result<SweepManifest, Box<dyn std::error::Error>> {
    let path = directory.join("sweep-manifest.json");
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "cannot retry {}: sweep-manifest.json is missing",
                directory.display()
            ),
        )
        .into());
    }
    let actual: SweepManifest = read_json(&path)?;
    if actual.schema_version != SWEEP_MANIFEST_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported sweep manifest schema {}; expected {}",
                actual.schema_version, SWEEP_MANIFEST_SCHEMA_VERSION
            ),
        )
        .into());
    }
    if actual != *expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "retry definition does not match immutable sweep {}",
                actual.sweep_id
            ),
        )
        .into());
    }
    Ok(actual)
}

fn write_analysis_outputs(
    directory: &Path,
    manifest: &SweepManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let run_rows = build_run_rows(directory, manifest)?;
    let point_rows = build_point_rows(manifest, &run_rows);
    let analysis_directory = directory.join("analysis");
    fs::create_dir_all(&analysis_directory)?;
    write_json(&analysis_directory.join("runs.json"), &run_rows)?;
    write_json(&analysis_directory.join("points.json"), &point_rows)?;
    write_runs_csv(&analysis_directory.join("runs.csv"), &run_rows)?;
    write_points_csv(&analysis_directory.join("points.csv"), &point_rows)?;
    let completed_runs = run_rows
        .iter()
        .filter(|row| row.state == "completed")
        .count();
    let summary = AnalysisSummary {
        schema_version: DERIVED_ANALYSIS_SCHEMA_VERSION,
        provenance: "derived",
        sweep_id: manifest.sweep_id.clone(),
        run_rows: run_rows.len(),
        point_rows: point_rows.len(),
        completed_runs,
        non_completed_runs: run_rows.len().saturating_sub(completed_runs),
        note: "Descriptive analysis only. Means are calculated from provenance-valid completed runs; every planned non-completed run remains explicit in runs.json/runs.csv and point status counts.",
    };
    write_json(&analysis_directory.join("summary.json"), &summary)?;
    Ok(())
}

fn build_run_rows(
    directory: &Path,
    sweep: &SweepManifest,
) -> Result<Vec<DerivedRunRow>, Box<dyn std::error::Error>> {
    let mut rows = Vec::with_capacity(sweep.points.len() * sweep.definition.seeds.len());
    for point in &sweep.points {
        let point_directory = directory.join(&point.relative_experiment_dir);
        let experiment_id = read_experiment_id(&point_directory)?;
        for &seed in &sweep.definition.seeds {
            let run_id = format!("seed-{seed:020}");
            let status_relative_path =
                format!("{}/status/{run_id}.json", point.relative_experiment_dir);
            let status_path = directory.join(&status_relative_path);
            let (state, attempt) = if status_path.is_file() {
                let value: serde_json::Value = read_json(&status_path)?;
                let state = value
                    .get("state")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("invalid_status")
                    .to_owned();
                let attempt = value
                    .get("attempt")
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(0);
                (state, attempt)
            } else {
                ("not_started".to_owned(), 0)
            };

            let manifest_relative_path = format!(
                "{}/runs/{run_id}/manifest.json",
                point.relative_experiment_dir
            );
            let run_manifest_path = directory.join(&manifest_relative_path);
            let mut row = DerivedRunRow {
                schema_version: DERIVED_ANALYSIS_SCHEMA_VERSION,
                provenance: "derived",
                sweep_id: sweep.sweep_id.clone(),
                point_id: point.point_id.clone(),
                experiment_id: experiment_id.clone(),
                run_id,
                seed,
                state: state.clone(),
                attempt,
                status_relative_path,
                manifest_relative_path: None,
                world_width: point.settings.world_width,
                world_height: point.settings.world_height,
                initial_population: point.settings.population,
                household_size: point.settings.household_size,
                max_person_records: point.settings.max_person_records,
                resource_productivity_scale_permille: point
                    .settings
                    .resource_productivity_scale_permille,
                annual_food_need: point.settings.annual_food_need,
                disable_migration: point.settings.disable_migration,
                migration_radius: point.settings.migration_radius,
                stop_reason: None,
                state_digest64: None,
                final_living_population: None,
                births_since_start: None,
                deaths_since_start: None,
                household_count: None,
                mean_living_condition_permille: None,
                authoritative_event_count: None,
            };

            if state == "completed" {
                if !run_manifest_path.is_file() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "completed sweep row {} / {} is missing its run manifest",
                            point.point_id, row.run_id
                        ),
                    )
                    .into());
                }
                let run_manifest: RunManifest = read_json(&run_manifest_path)?;
                if run_manifest.experiment.seed != seed
                    || run_manifest.experiment.world.width != point.settings.world_width
                    || run_manifest.experiment.world.height != point.settings.world_height
                    || run_manifest.experiment.population.initial_population
                        != point.settings.population
                    || run_manifest.experiment.population.target_household_size
                        != point.settings.household_size
                    || run_manifest.experiment.population.max_person_records
                        != point.settings.max_person_records
                    || run_manifest
                        .experiment
                        .resources
                        .productivity_scale_permille
                        != point.settings.resource_productivity_scale_permille
                    || run_manifest
                        .experiment
                        .resources
                        .annual_need_units_per_person
                        != point.settings.annual_food_need
                    || run_manifest.experiment.migration.enabled == point.settings.disable_migration
                    || run_manifest.experiment.migration.candidate_radius_cells
                        != point.settings.migration_radius
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "completed sweep row {} / {} does not match its declared parameter point",
                            point.point_id, row.run_id
                        ),
                    )
                    .into());
                }
                row.manifest_relative_path = Some(manifest_relative_path);
                row.stop_reason = serde_json::to_value(&run_manifest.stop_reason)?
                    .as_str()
                    .map(str::to_owned);
                row.state_digest64 = Some(run_manifest.state_digest64);
                row.final_living_population = Some(run_manifest.population.living_population);
                row.births_since_start = Some(run_manifest.population.births_since_start);
                row.deaths_since_start = Some(run_manifest.population.deaths_since_start);
                row.household_count = Some(run_manifest.population.household_count);
                row.mean_living_condition_permille =
                    Some(run_manifest.population.mean_living_condition_permille);
                row.authoritative_event_count =
                    Some(run_manifest.statistics.authoritative_event_count);
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

fn read_experiment_id(
    point_directory: &Path,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let path = point_directory.join("experiment-manifest.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value: serde_json::Value = read_json(&path)?;
    Ok(value
        .get("experimentId")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned))
}

fn build_point_rows(sweep: &SweepManifest, runs: &[DerivedRunRow]) -> Vec<DerivedPointRow> {
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
            let failed_runs = point_runs
                .iter()
                .filter(|row| row.state == "failed")
                .count() as u64;
            let incomplete_runs = point_runs
                .iter()
                .filter(|row| row.state == "incomplete")
                .count() as u64;
            let other_non_completed_runs =
                point_runs.len() as u64 - completed_runs - failed_runs - incomplete_runs;
            DerivedPointRow {
                schema_version: DERIVED_ANALYSIS_SCHEMA_VERSION,
                provenance: "derived",
                sweep_id: sweep.sweep_id.clone(),
                point_id: point.point_id.clone(),
                experiment_id: point_runs.first().and_then(|row| row.experiment_id.clone()),
                planned_runs: point_runs.len() as u64,
                completed_runs,
                failed_runs,
                incomplete_runs,
                other_non_completed_runs,
                mean_final_living_population_completed_only: mean_u64(
                    completed
                        .iter()
                        .filter_map(|row| row.final_living_population),
                ),
                mean_births_since_start_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.births_since_start),
                ),
                mean_deaths_since_start_completed_only: mean_u64(
                    completed.iter().filter_map(|row| row.deaths_since_start),
                ),
                source_completed_run_ids: completed
                    .iter()
                    .map(|row| format!("{}/{}", row.point_id, row.run_id))
                    .collect(),
            }
        })
        .collect()
}

fn mean_u64(values: impl Iterator<Item = u64>) -> Option<f64> {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    let total = values
        .iter()
        .fold(0_u128, |sum, value| sum + u128::from(*value));
    Some(total as f64 / values.len() as f64)
}

fn write_runs_csv(path: &Path, rows: &[DerivedRunRow]) -> Result<(), io::Error> {
    let mut csv = String::from(
        "sweep_id,point_id,experiment_id,run_id,seed,state,attempt,status_relative_path,manifest_relative_path,world_width,world_height,initial_population,household_size,max_person_records,resource_productivity_scale_permille,annual_food_need,disable_migration,migration_radius,stop_reason,state_digest64,final_living_population,births_since_start,deaths_since_start,household_count,mean_living_condition_permille,authoritative_event_count\n",
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
        ]));
    }
    fs::write(path, csv)
}

fn write_points_csv(path: &Path, rows: &[DerivedPointRow]) -> Result<(), io::Error> {
    let mut csv = String::from(
        "sweep_id,point_id,experiment_id,planned_runs,completed_runs,failed_runs,incomplete_runs,other_non_completed_runs,mean_final_living_population_completed_only,mean_births_since_start_completed_only,mean_deaths_since_start_completed_only,source_completed_run_ids\n",
    );
    for row in rows {
        csv.push_str(&csv_line(&[
            row.sweep_id.clone(),
            row.point_id.clone(),
            row.experiment_id.clone().unwrap_or_default(),
            row.planned_runs.to_string(),
            row.completed_runs.to_string(),
            row.failed_runs.to_string(),
            row.incomplete_runs.to_string(),
            row.other_non_completed_runs.to_string(),
            row.mean_final_living_population_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_births_since_start_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.mean_deaths_since_start_completed_only
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.source_completed_run_ids.join("|"),
        ]));
    }
    fs::write(path, csv)
}

fn csv_line(fields: &[String]) -> String {
    let mut line = fields
        .iter()
        .map(|field| csv_escape(field))
        .collect::<Vec<_>>()
        .join(",");
    line.push('\n');
    line
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn optional_to_string<T: Display>(value: Option<T>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn require_empty_directory(directory: &Path) -> Result<(), io::Error> {
    if !directory.exists() {
        return Ok(());
    }
    if !directory.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "sweep output path {} exists and is not a directory",
                directory.display()
            ),
        ));
    }
    if fs::read_dir(directory)?.next().transpose()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "sweep output directory {} is not empty; use --retry only for the exact immutable sweep stored there",
                directory.display()
            ),
        ));
    }
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn small_settings() -> EnsembleRunSettings {
        EnsembleRunSettings {
            years: 0,
            world_width: 4,
            world_height: 4,
            population: 12,
            household_size: 4,
            max_person_records: 100,
            resource_productivity_scale_permille: 1_000,
            annual_food_need: 100,
            disable_migration: false,
            migration_radius: 3,
        }
    }

    fn dimensions() -> SweepDimensions {
        SweepDimensions {
            population: vec![],
            household_size: vec![],
            resource_productivity_scale_permille: vec![800, 1_000],
            annual_food_need: vec![80, 100],
            disable_migration: vec![],
            migration_radius: vec![],
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("anthrosim-{label}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn sweep_expansion_is_deterministic_cartesian_product() {
        let first = expand_sweep_points(&small_settings(), &dimensions()).expect("points");
        let second = expand_sweep_points(&small_settings(), &dimensions()).expect("points");
        assert_eq!(first, second);
        assert_eq!(first.len(), 4);
        assert_eq!(first[0].settings.resource_productivity_scale_permille, 800);
        assert_eq!(first[0].settings.annual_food_need, 80);
        assert_eq!(first[1].settings.resource_productivity_scale_permille, 800);
        assert_eq!(first[1].settings.annual_food_need, 100);
        assert_eq!(
            first[2].settings.resource_productivity_scale_permille,
            1_000
        );
        assert_eq!(first[2].settings.annual_food_need, 80);
        assert_eq!(first[3].point_id, "point-000003");
    }

    #[test]
    fn duplicate_dimension_values_are_rejected() {
        let mut duplicate = dimensions();
        duplicate.annual_food_need = vec![100, 100];
        assert!(build_sweep_manifest(small_settings(), vec![1], duplicate).is_err());
    }

    #[test]
    fn immutable_sweep_manifest_is_stable_and_exact() {
        let first =
            build_sweep_manifest(small_settings(), vec![3, 7], dimensions()).expect("sweep");
        let second =
            build_sweep_manifest(small_settings(), vec![3, 7], dimensions()).expect("sweep");
        assert_eq!(first, second);
        assert_eq!(first.points.len(), 4);
        let mut changed = dimensions();
        changed.annual_food_need = vec![80, 120];
        let changed = build_sweep_manifest(small_settings(), vec![3, 7], changed).expect("sweep");
        assert_ne!(first.sweep_id, changed.sweep_id);
    }

    #[test]
    fn sweep_outputs_traceable_run_and_point_tables() {
        let root = temp_path("sweep-analysis");
        let dimensions = SweepDimensions {
            population: vec![],
            household_size: vec![],
            resource_productivity_scale_permille: vec![900, 1_000],
            annual_food_need: vec![],
            disable_migration: vec![],
            migration_radius: vec![],
        };
        execute_sweep(&root, small_settings(), vec![11, 12], dimensions, false)
            .expect("sweep completes");

        let runs: Vec<DerivedRunRow> = read_json(&root.join("analysis/runs.json")).expect("runs");
        let points: Vec<DerivedPointRow> =
            read_json(&root.join("analysis/points.json")).expect("points");
        assert_eq!(runs.len(), 4);
        assert!(runs.iter().all(|row| row.state == "completed"));
        assert!(runs.iter().all(|row| row.manifest_relative_path.is_some()));
        assert_eq!(points.len(), 2);
        assert!(points.iter().all(|row| row.completed_runs == 2));
        assert!(
            points
                .iter()
                .all(|row| row.source_completed_run_ids.len() == 2)
        );
        assert!(root.join("analysis/runs.csv").is_file());
        assert!(root.join("analysis/points.csv").is_file());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn retry_rejects_changed_sweep_without_mutating_manifest() {
        let root = temp_path("sweep-retry-mismatch");
        let dimensions = SweepDimensions {
            population: vec![12, 16],
            household_size: vec![],
            resource_productivity_scale_permille: vec![],
            annual_food_need: vec![],
            disable_migration: vec![],
            migration_radius: vec![],
        };
        execute_sweep(&root, small_settings(), vec![21], dimensions, false).expect("sweep");
        let before = fs::read(root.join("sweep-manifest.json")).expect("manifest");
        let changed = SweepDimensions {
            population: vec![12, 20],
            household_size: vec![],
            resource_productivity_scale_permille: vec![],
            annual_food_need: vec![],
            disable_migration: vec![],
            migration_radius: vec![],
        };
        assert!(execute_sweep(&root, small_settings(), vec![21], changed, true).is_err());
        let after = fs::read(root.join("sweep-manifest.json")).expect("manifest");
        assert_eq!(before, after);
        fs::remove_dir_all(root).expect("cleanup");
    }
}
