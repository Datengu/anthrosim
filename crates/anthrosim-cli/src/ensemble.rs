use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use anthrosim_core::{
    ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, RunManifest, Simulation,
    SimulationCheckpoint, WorldConfig,
};
use serde::{Deserialize, Serialize};

use crate::{read_json, write_completed_bundle, write_json};

const ENSEMBLE_PLAN_SCHEMA_VERSION: u32 = 1;
const ENSEMBLE_COMPLETION_SCHEMA_VERSION: u32 = 1;
const EXPERIMENT_MANIFEST_SCHEMA_VERSION: u32 = 1;
const RUN_STATUS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnsembleRunSettings {
    pub(crate) years: u64,
    pub(crate) world_width: u32,
    pub(crate) world_height: u32,
    pub(crate) population: u32,
    pub(crate) household_size: u16,
    pub(crate) max_person_records: u64,
    pub(crate) resource_productivity_scale_permille: u16,
    pub(crate) annual_food_need: u32,
    pub(crate) disable_migration: bool,
    pub(crate) migration_radius: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsemblePlan {
    schema_version: u32,
    definition: EnsembleDefinition,
    runs: Vec<PlannedRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsembleDefinition {
    seeds: Vec<u64>,
    settings: EnsembleRunSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlannedRun {
    seed: u64,
    relative_run_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExperimentManifest {
    schema_version: u32,
    experiment_id: String,
    model_version: String,
    git_commit: Option<String>,
    runs: Vec<ExperimentRunSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExperimentRunSpec {
    run_id: String,
    relative_run_dir: String,
    experiment: ExperimentConfig,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExperimentIdentity<'a> {
    schema_version: u32,
    model_version: &'a str,
    git_commit: &'a Option<String>,
    runs: &'a [ExperimentRunSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RunLifecycle {
    Planned,
    Running,
    Completed,
    Failed,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunStatus {
    schema_version: u32,
    experiment_id: String,
    run_id: String,
    seed: u64,
    state: RunLifecycle,
    attempt: u32,
    message: Option<String>,
    result: Option<RunResultRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunResultRef {
    manifest_relative_path: String,
    state_digest64: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnsembleRunCompletion {
    schema_version: u32,
    seed: u64,
    status: String,
    manifest: String,
}

enum BundleInspection {
    AbsentOrIncomplete,
    Valid(RunResultRef),
}

pub(crate) fn experiment_config(
    seed: u64,
    settings: &EnsembleRunSettings,
) -> ExperimentConfig {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(settings.resource_productivity_scale_permille)
        .with_annual_need_units_per_person(settings.annual_food_need);
    let migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(!settings.disable_migration)
        .with_candidate_radius_cells(settings.migration_radius);
    ExperimentConfig::new(seed, settings.years)
        .with_world(WorldConfig::new(
            settings.world_width,
            settings.world_height,
        ))
        .with_population(
            PopulationConfig::new(settings.population)
                .with_target_household_size(settings.household_size)
                .with_max_person_records(settings.max_person_records),
        )
        .with_resources(resources)
        .with_migration(migration)
}

pub(crate) fn resolve_ensemble_seeds(
    explicit_seeds: Vec<u64>,
    seed_start: Option<u64>,
    seed_count: Option<u32>,
) -> Result<Vec<u64>, io::Error> {
    if !explicit_seeds.is_empty() {
        validate_unique_seeds(&explicit_seeds)?;
        return Ok(explicit_seeds);
    }

    let (start, count) = match (seed_start, seed_count) {
        (Some(start), Some(count)) => (start, count),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ensemble requires either --seeds or both --seed-start and --seed-count",
            ));
        }
    };

    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--seed-count must be greater than zero",
        ));
    }

    let final_offset = u64::from(count - 1);
    start.checked_add(final_offset).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "seed range exceeds the maximum u64 seed",
        )
    })?;

    Ok((0..count).map(|offset| start + u64::from(offset)).collect())
}

fn validate_unique_seeds(seeds: &[u64]) -> Result<(), io::Error> {
    let mut seen = HashSet::with_capacity(seeds.len());
    for &seed in seeds {
        if !seen.insert(seed) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("duplicate ensemble seed {seed} would target the same run directory"),
            ));
        }
    }
    Ok(())
}

fn plan_ensemble(
    settings: EnsembleRunSettings,
    seeds: Vec<u64>,
) -> Result<EnsemblePlan, io::Error> {
    if seeds.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ensemble must contain at least one seed",
        ));
    }
    validate_unique_seeds(&seeds)?;

    let runs = seeds
        .iter()
        .map(|&seed| PlannedRun {
            seed,
            relative_run_dir: run_relative_dir(seed),
        })
        .collect();

    Ok(EnsemblePlan {
        schema_version: ENSEMBLE_PLAN_SCHEMA_VERSION,
        definition: EnsembleDefinition { seeds, settings },
        runs,
    })
}

fn build_experiment_manifest(
    settings: &EnsembleRunSettings,
    seeds: &[u64],
) -> Result<ExperimentManifest, Box<dyn std::error::Error>> {
    if seeds.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ensemble must contain at least one seed",
        )
        .into());
    }
    validate_unique_seeds(seeds)?;

    let runs = seeds
        .iter()
        .map(|&seed| ExperimentRunSpec {
            run_id: run_id(seed),
            relative_run_dir: run_relative_dir(seed),
            experiment: experiment_config(seed, settings),
        })
        .collect::<Vec<_>>();
    let model_version = env!("CARGO_PKG_VERSION").to_owned();
    let git_commit = option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned);
    let identity = ExperimentIdentity {
        schema_version: EXPERIMENT_MANIFEST_SCHEMA_VERSION,
        model_version: &model_version,
        git_commit: &git_commit,
        runs: &runs,
    };
    let identity_bytes = serde_json::to_vec(&identity)?;
    let experiment_id = format!(
        "anthrosim-exp-v{EXPERIMENT_MANIFEST_SCHEMA_VERSION}-{:016x}",
        fnv1a64(&identity_bytes)
    );

    Ok(ExperimentManifest {
        schema_version: EXPERIMENT_MANIFEST_SCHEMA_VERSION,
        experiment_id,
        model_version,
        git_commit,
        runs,
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn run_id(seed: u64) -> String {
    format!("seed-{seed:020}")
}

fn run_relative_dir(seed: u64) -> String {
    format!("runs/{}", run_id(seed))
}

fn status_relative_path(run_id: &str) -> PathBuf {
    Path::new("status").join(format!("{run_id}.json"))
}

fn initial_status(manifest: &ExperimentManifest, spec: &ExperimentRunSpec) -> RunStatus {
    RunStatus {
        schema_version: RUN_STATUS_SCHEMA_VERSION,
        experiment_id: manifest.experiment_id.clone(),
        run_id: spec.run_id.clone(),
        seed: spec.experiment.seed,
        state: RunLifecycle::Planned,
        attempt: 0,
        message: None,
        result: None,
    }
}

pub(crate) fn execute_ensemble(
    directory: &Path,
    settings: EnsembleRunSettings,
    seeds: Vec<u64>,
    retry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan_ensemble(settings.clone(), seeds.clone())?;
    let expected_manifest = build_experiment_manifest(&settings, &seeds)?;

    let manifest = if retry {
        load_matching_manifest(directory, &expected_manifest)?
    } else {
        initialize_experiment(directory, &plan, &expected_manifest)?;
        expected_manifest
    };

    let mut unsuccessful = 0_u32;
    for spec in &manifest.runs {
        let mut status = reconcile_status(directory, &manifest, spec)?;
        if status.state == RunLifecycle::Completed {
            println!("kept completed ensemble run {}", spec.run_id);
            continue;
        }

        let succeeded = execute_run_attempt(directory, &manifest, spec, &mut status)?;
        if !succeeded {
            unsuccessful = unsuccessful.saturating_add(1);
        }
    }

    if unsuccessful > 0 {
        return Err(io::Error::other(format!(
            "ensemble finished with {unsuccessful} unsuccessful run(s); inspect status/*.json and retry the exact experiment with --retry"
        ))
        .into());
    }

    println!(
        "completed experiment {} with {} runs in {}",
        manifest.experiment_id,
        manifest.runs.len(),
        directory.display()
    );
    Ok(())
}

fn initialize_experiment(
    directory: &Path,
    plan: &EnsemblePlan,
    manifest: &ExperimentManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    require_empty_ensemble_directory(directory)?;
    fs::create_dir_all(directory)?;
    write_json(&directory.join("experiment-manifest.json"), manifest)?;
    write_json(&directory.join("ensemble-plan.json"), plan)?;
    for spec in &manifest.runs {
        write_status(directory, &initial_status(manifest, spec))?;
    }
    Ok(())
}

fn load_matching_manifest(
    directory: &Path,
    expected: &ExperimentManifest,
) -> Result<ExperimentManifest, Box<dyn std::error::Error>> {
    let path = directory.join("experiment-manifest.json");
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "cannot retry {}: experiment-manifest.json is missing",
                directory.display()
            ),
        )
        .into());
    }

    let actual: ExperimentManifest = read_json(&path)?;
    if actual.schema_version != EXPERIMENT_MANIFEST_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported experiment manifest schema {}; expected {}",
                actual.schema_version, EXPERIMENT_MANIFEST_SCHEMA_VERSION
            ),
        )
        .into());
    }
    if actual != *expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "retry definition does not match immutable experiment {}",
                actual.experiment_id
            ),
        )
        .into());
    }
    Ok(actual)
}

fn reconcile_status(
    directory: &Path,
    manifest: &ExperimentManifest,
    spec: &ExperimentRunSpec,
) -> Result<RunStatus, Box<dyn std::error::Error>> {
    let status_path = directory.join(status_relative_path(&spec.run_id));
    let mut status = if status_path.is_file() {
        let loaded: RunStatus = read_json(&status_path)?;
        validate_status_identity(manifest, spec, &loaded)?;
        loaded
    } else {
        let mut missing = initial_status(manifest, spec);
        missing.state = RunLifecycle::Incomplete;
        missing.message = Some("status record was missing during reconciliation".to_owned());
        missing
    };

    let bundle = inspect_completed_bundle(directory, manifest, spec)?;
    match bundle {
        BundleInspection::Valid(result) => {
            status.state = RunLifecycle::Completed;
            status.message = None;
            status.result = Some(result);
            if status.attempt == 0 {
                status.attempt = 1;
            }
            write_status(directory, &status)?;
            Ok(status)
        }
        BundleInspection::AbsentOrIncomplete => {
            if status.state == RunLifecycle::Completed || status.state == RunLifecycle::Running {
                status.state = RunLifecycle::Incomplete;
                status.message = Some(
                    "previous execution did not leave a complete provenance-valid run bundle"
                        .to_owned(),
                );
                status.result = None;
                write_status(directory, &status)?;
            } else if !status_path.is_file() {
                write_status(directory, &status)?;
            }
            Ok(status)
        }
    }
}

fn validate_status_identity(
    manifest: &ExperimentManifest,
    spec: &ExperimentRunSpec,
    status: &RunStatus,
) -> Result<(), io::Error> {
    if status.schema_version != RUN_STATUS_SCHEMA_VERSION
        || status.experiment_id != manifest.experiment_id
        || status.run_id != spec.run_id
        || status.seed != spec.experiment.seed
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("status identity mismatch for {}", spec.run_id),
        ));
    }
    Ok(())
}

fn inspect_completed_bundle(
    directory: &Path,
    experiment_manifest: &ExperimentManifest,
    spec: &ExperimentRunSpec,
) -> Result<BundleInspection, Box<dyn std::error::Error>> {
    let run_directory = directory.join(&spec.relative_run_dir);
    if !run_directory.is_dir() {
        return Ok(BundleInspection::AbsentOrIncomplete);
    }

    let required = [
        "manifest.json",
        "world.json",
        "initial-population.json",
        "events.json",
        "metrics.json",
        "checkpoint.json",
        "completion.json",
    ];
    if required
        .iter()
        .any(|artifact| !run_directory.join(artifact).is_file())
    {
        return Ok(BundleInspection::AbsentOrIncomplete);
    }

    let completion: EnsembleRunCompletion = read_json(&run_directory.join("completion.json"))?;
    if completion.schema_version != ENSEMBLE_COMPLETION_SCHEMA_VERSION
        || completion.seed != spec.experiment.seed
        || completion.status != "completed"
        || completion.manifest != "manifest.json"
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid completion marker for {}", spec.run_id),
        )
        .into());
    }

    let run_manifest: RunManifest = read_json(&run_directory.join("manifest.json"))?;
    if run_manifest.experiment != spec.experiment
        || run_manifest.model_version != experiment_manifest.model_version
        || run_manifest.git_commit != experiment_manifest.git_commit
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "completed bundle provenance does not match immutable experiment for {}",
                spec.run_id
            ),
        )
        .into());
    }

    let checkpoint: SimulationCheckpoint = read_json(&run_directory.join("checkpoint.json"))?;
    if checkpoint.experiment != spec.experiment
        || checkpoint.model_version != experiment_manifest.model_version
        || checkpoint.state_digest64 != run_manifest.state_digest64
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "completed checkpoint does not reconcile with immutable experiment for {}",
                spec.run_id
            ),
        )
        .into());
    }

    Ok(BundleInspection::Valid(RunResultRef {
        manifest_relative_path: format!("{}/manifest.json", spec.relative_run_dir),
        state_digest64: run_manifest.state_digest64,
    }))
}

fn execute_run_attempt(
    directory: &Path,
    manifest: &ExperimentManifest,
    spec: &ExperimentRunSpec,
    status: &mut RunStatus,
) -> Result<bool, Box<dyn std::error::Error>> {
    let run_directory = directory.join(&spec.relative_run_dir);
    if run_directory.exists() {
        fs::remove_dir_all(&run_directory)?;
    }

    status.state = RunLifecycle::Running;
    status.attempt = status.attempt.saturating_add(1);
    status.message = None;
    status.result = None;
    write_status(directory, status)?;

    let attempt = (|| -> Result<RunResultRef, Box<dyn std::error::Error>> {
        let simulation = Simulation::new(spec.experiment.clone())?;
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded()?;
        write_completed_bundle(
            &run_directory,
            &world,
            &initial_population,
            &recorded,
        )?;
        write_json(
            &run_directory.join("completion.json"),
            &EnsembleRunCompletion {
                schema_version: ENSEMBLE_COMPLETION_SCHEMA_VERSION,
                seed: spec.experiment.seed,
                status: "completed".to_owned(),
                manifest: "manifest.json".to_owned(),
            },
        )?;
        Ok(RunResultRef {
            manifest_relative_path: format!("{}/manifest.json", spec.relative_run_dir),
            state_digest64: recorded.manifest.state_digest64,
        })
    })();

    match attempt {
        Ok(result) => {
            status.state = RunLifecycle::Completed;
            status.message = None;
            status.result = Some(result);
            write_status(directory, status)?;
            println!(
                "completed experiment {} run {} attempt {}",
                manifest.experiment_id, spec.run_id, status.attempt
            );
            Ok(true)
        }
        Err(error) => {
            status.state = RunLifecycle::Failed;
            status.message = Some(error.to_string());
            status.result = None;
            write_status(directory, status)?;
            eprintln!(
                "failed experiment {} run {} attempt {}: {error}",
                manifest.experiment_id, spec.run_id, status.attempt
            );
            Ok(false)
        }
    }
}

fn write_status(directory: &Path, status: &RunStatus) -> Result<(), Box<dyn std::error::Error>> {
    write_json(
        &directory.join(status_relative_path(&status.run_id)),
        status,
    )
}

fn require_empty_ensemble_directory(directory: &Path) -> Result<(), io::Error> {
    if !directory.exists() {
        return Ok(());
    }
    if !directory.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "ensemble output path {} exists and is not a directory",
                directory.display()
            ),
        ));
    }
    if fs::read_dir(directory)?.next().transpose()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "ensemble output directory {} is not empty; use --retry only for the exact immutable experiment already stored there",
                directory.display()
            ),
        ));
    }
    Ok(())
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

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("anthrosim-{label}-{}-{nanos}", std::process::id()))
    }

    fn read_status(root: &Path, seed: u64) -> RunStatus {
        read_json(&root.join(status_relative_path(&run_id(seed)))).expect("status")
    }

    #[test]
    fn explicit_seed_plan_is_deterministic_and_separated() {
        let settings = small_settings();
        let first = plan_ensemble(settings.clone(), vec![9, 2, 42]).expect("valid plan");
        let second = plan_ensemble(settings, vec![9, 2, 42]).expect("valid plan");

        assert_eq!(first, second);
        assert_eq!(first.definition.seeds, vec![9, 2, 42]);
        assert_eq!(
            first.runs[0].relative_run_dir,
            "runs/seed-00000000000000000009"
        );
        assert_eq!(
            first.runs[1].relative_run_dir,
            "runs/seed-00000000000000000002"
        );
        assert_ne!(
            first.runs[0].relative_run_dir,
            first.runs[1].relative_run_dir
        );
    }

    #[test]
    fn consecutive_seed_range_is_stable_and_overflow_checked() {
        assert_eq!(
            resolve_ensemble_seeds(Vec::new(), Some(100), Some(4)).expect("valid range"),
            vec![100, 101, 102, 103]
        );
        assert!(resolve_ensemble_seeds(Vec::new(), Some(7), Some(0)).is_err());
        assert!(resolve_ensemble_seeds(Vec::new(), Some(u64::MAX), Some(2)).is_err());
    }

    #[test]
    fn duplicate_explicit_seeds_are_rejected() {
        assert!(resolve_ensemble_seeds(vec![5, 8, 5], None, None).is_err());
    }

    #[test]
    fn immutable_manifest_is_stable_and_contains_exact_configs() {
        let settings = small_settings();
        let first = build_experiment_manifest(&settings, &[5, 9]).expect("manifest");
        let second = build_experiment_manifest(&settings, &[5, 9]).expect("manifest");
        assert_eq!(first, second);
        assert_eq!(first.runs[0].experiment, experiment_config(5, &settings));
        assert_eq!(first.runs[1].experiment, experiment_config(9, &settings));

        let changed = build_experiment_manifest(&settings, &[5, 10]).expect("manifest");
        assert_ne!(first.experiment_id, changed.experiment_id);
    }

    #[test]
    fn completed_retry_is_idempotent_and_does_not_rewrite_manifest() {
        let root = temp_path("retry-completed");
        execute_ensemble(&root, small_settings(), vec![11, 12], false).expect("fresh experiment");
        let before = fs::read(root.join("experiment-manifest.json")).expect("manifest bytes");
        let before_status = read_status(&root, 11);

        execute_ensemble(&root, small_settings(), vec![11, 12], true).expect("retry complete");

        let after = fs::read(root.join("experiment-manifest.json")).expect("manifest bytes");
        let after_status = read_status(&root, 11);
        assert_eq!(before, after);
        assert_eq!(before_status, after_status);
        assert_eq!(after_status.attempt, 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn retry_rejects_changed_definition_without_mutating_manifest() {
        let root = temp_path("retry-mismatch");
        execute_ensemble(&root, small_settings(), vec![21], false).expect("fresh experiment");
        let before = fs::read(root.join("experiment-manifest.json")).expect("manifest bytes");

        assert!(execute_ensemble(&root, small_settings(), vec![22], true).is_err());

        let after = fs::read(root.join("experiment-manifest.json")).expect("manifest bytes");
        assert_eq!(before, after);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn interrupted_running_status_is_reconciled_and_retried() {
        let root = temp_path("retry-interrupted");
        execute_ensemble(&root, small_settings(), vec![31, 32], false).expect("fresh experiment");

        let mut interrupted = read_status(&root, 31);
        interrupted.state = RunLifecycle::Running;
        interrupted.result = None;
        write_status(&root, &interrupted).expect("write interrupted status");
        fs::remove_file(
            root.join(run_relative_dir(31))
                .join("completion.json"),
        )
        .expect("remove completion marker");

        execute_ensemble(&root, small_settings(), vec![31, 32], true).expect("retry experiment");

        let retried = read_status(&root, 31);
        let untouched = read_status(&root, 32);
        assert_eq!(retried.state, RunLifecycle::Completed);
        assert_eq!(retried.attempt, 2);
        assert_eq!(untouched.attempt, 1);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn missing_completed_artifact_is_not_silently_kept() {
        let root = temp_path("retry-missing-artifact");
        execute_ensemble(&root, small_settings(), vec![41], false).expect("fresh experiment");
        fs::remove_file(root.join(run_relative_dir(41)).join("metrics.json"))
            .expect("remove metrics");

        execute_ensemble(&root, small_settings(), vec![41], true).expect("retry experiment");

        let status = read_status(&root, 41);
        assert_eq!(status.state, RunLifecycle::Completed);
        assert_eq!(status.attempt, 2);
        assert!(root.join(run_relative_dir(41)).join("metrics.json").is_file());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn failed_runs_are_recorded_and_retry_deterministically() {
        let root = temp_path("retry-failed");
        let mut invalid = small_settings();
        invalid.world_width = 0;

        assert!(execute_ensemble(&root, invalid.clone(), vec![51], false).is_err());
        let failed = read_status(&root, 51);
        assert_eq!(failed.state, RunLifecycle::Failed);
        assert_eq!(failed.attempt, 1);
        assert!(failed.message.is_some());

        assert!(execute_ensemble(&root, invalid, vec![51], true).is_err());
        let retried = read_status(&root, 51);
        assert_eq!(retried.state, RunLifecycle::Failed);
        assert_eq!(retried.attempt, 2);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn fresh_execution_refuses_nonempty_root() {
        let root = temp_path("nonempty");
        fs::create_dir_all(&root).expect("root");
        fs::write(root.join("unrelated.txt"), "do not overwrite").expect("file");
        assert!(execute_ensemble(&root, small_settings(), vec![61], false).is_err());
        assert_eq!(
            fs::read_to_string(root.join("unrelated.txt")).expect("unrelated file"),
            "do not overwrite"
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}
