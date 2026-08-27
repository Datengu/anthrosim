use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use anthrosim_core::{
    Population, ResearchCoordinate, ResearchExperimentDefinition, ResearchPoint, ResearchRunConfig,
    Simulation, SimulationCheckpoint, SourceRevisionIdentity, SpatialLandscapeCheckpoint,
    SpatialLandscapeRunManifest, SpatialLandscapeSimulation, World, research_run_identity,
    validate_resolved_research_run,
};
use clap::Parser;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[path = "../bundle.rs"]
mod bundle;
#[allow(dead_code)]
#[path = "../run_directory.rs"]
mod run_directory;

use run_directory::{RunDirectoryTransaction, recover_interrupted_replacement};

const RESEARCH_MANIFEST: &str = "research-manifest.json";
const RESEARCH_STATE: &str = "research-state.json";

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-research",
    about = "Execute a versioned exact-configuration AnthroSim research experiment"
)]
struct Cli {
    /// Versioned research experiment definition JSON.
    #[arg(long)]
    definition: PathBuf,

    /// Immutable experiment root containing plan, run bundles, recovery state and analysis tables.
    #[arg(long)]
    run_dir: PathBuf,

    /// Reconcile completed bundles and retry only missing/failed runs in this exact experiment.
    #[arg(long, default_value_t = false)]
    retry: bool,
}

fn main() -> std::process::ExitCode {
    match execute(&Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-research: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn execute(cli: &Cli) -> Result<(), Box<dyn Error>> {
    let definition: ResearchExperimentDefinition = read_json(&cli.definition)?;
    let manifest = build_manifest(definition)?;
    prepare_root(&cli.run_dir, &manifest, cli.retry)?;
    let state_path = cli.run_dir.join(RESEARCH_STATE);
    let mut state = load_or_initialize_state(&state_path, &manifest, cli.retry)?;

    let mut failures = Vec::new();
    for point in &manifest.points {
        for planned in &point.runs {
            let run_dir = cli.run_dir.join(&planned.relative_dir);
            recover_interrupted_replacement(&run_dir)?;

            if run_dir.exists() {
                let digest = validate_completed_run(&run_dir, planned)?;
                state.reconcile_completed(planned, digest);
                write_state(&state_path, &state)?;
                continue;
            }

            let attempt = state.begin_attempt(planned)?;
            write_state(&state_path, &state)?;
            match execute_planned_run(&run_dir, planned) {
                Ok(completion) => {
                    state.finish_completed(planned, attempt, completion)?;
                    write_state(&state_path, &state)?;
                }
                Err(error) => {
                    let message = error.to_string();
                    state.finish_failed(planned, attempt, message.clone())?;
                    write_state(&state_path, &state)?;
                    failures.push(format!("{}: {message}", planned.run_id));
                }
            }
        }
    }

    write_analysis(&cli.run_dir, &manifest, &state)?;
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} research run(s) failed; immutable state was preserved for --retry: {}",
            failures.len(),
            failures.join("; ")
        )
        .into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResearchExecutionManifest {
    schema_version: u32,
    research_id: String,
    definition_identity: String,
    source: SourceRevisionIdentity,
    definition: ResearchExperimentDefinition,
    points: Vec<PlannedPoint>,
}

impl ResearchExecutionManifest {
    const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlannedPoint {
    point: ResearchPoint,
    runs: Vec<PlannedRun>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlannedRun {
    seed: u64,
    run_id: String,
    relative_dir: PathBuf,
    run_config: ResearchRunConfig,
}

fn build_manifest(
    definition: ResearchExperimentDefinition,
) -> Result<ResearchExecutionManifest, Box<dyn Error>> {
    let definition_identity = definition.identity()?;
    let points = definition.expand()?;

    // Fail closed before publishing an experiment root. Seed changes do not alter configuration
    // validity, so one normal authoritative constructor per scientific point is sufficient here.
    for point in &points {
        validate_resolved_research_run(&point.run_config)?;
    }

    let source = SourceRevisionIdentity::current();
    let planned_points = points
        .into_iter()
        .map(|point| {
            let runs = definition
                .seeds
                .iter()
                .enumerate()
                .map(|(seed_index, seed)| {
                    let run_config = point.run_config.for_seed(*seed);
                    let run_id = research_run_identity(&point.point_id, &run_config, &source)?;
                    Ok(PlannedRun {
                        seed: *seed,
                        run_id,
                        relative_dir: PathBuf::from("points")
                            .join(format!("point-{:06}", point.index))
                            .join("runs")
                            .join(format!("seed-{seed_index:06}-{seed:020}")),
                        run_config,
                    })
                })
                .collect::<Result<Vec<_>, anthrosim_core::ResearchExperimentError>>()?;
            Ok(PlannedPoint { point, runs })
        })
        .collect::<Result<Vec<_>, anthrosim_core::ResearchExperimentError>>()?;

    let research_id = execution_identity(&definition_identity, &source);
    Ok(ResearchExecutionManifest {
        schema_version: ResearchExecutionManifest::CURRENT_SCHEMA_VERSION,
        research_id,
        definition_identity,
        source,
        definition,
        points: planned_points,
    })
}

fn execution_identity(definition_identity: &str, source: &SourceRevisionIdentity) -> String {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        definition_identity: &'a str,
        source: &'a SourceRevisionIdentity,
    }
    let bytes = serde_json::to_vec(&Identity {
        schema_version: 1,
        definition_identity,
        source,
    })
    .expect("research execution identity must serialize");
    format!("research-execution-v1-{:016x}", fnv1a64(&bytes))
}

fn prepare_root(
    root: &Path,
    expected: &ResearchExecutionManifest,
    retry: bool,
) -> Result<(), Box<dyn Error>> {
    reject_symlink(root, "research root")?;
    let manifest_path = root.join(RESEARCH_MANIFEST);
    if retry {
        if !root.is_dir() {
            return Err(format!(
                "research root does not exist for --retry: {}",
                root.display()
            )
            .into());
        }
        let recorded: ResearchExecutionManifest = read_json(&manifest_path)?;
        if &recorded != expected {
            return Err(
                "--retry definition/source does not exactly match immutable research-manifest.json"
                    .into(),
            );
        }
        return Ok(());
    }

    if root.exists() {
        if !root.is_dir() {
            return Err(format!(
                "research root exists and is not a directory: {}",
                root.display()
            )
            .into());
        }
        if fs::read_dir(root)?.next().transpose()?.is_some() {
            return Err(format!("research root is not empty: {}", root.display()).into());
        }
    } else {
        fs::create_dir_all(root)?;
    }
    write_json_atomic(&manifest_path, expected, "immutable research manifest")?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RunStateKind {
    Planned,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RunState {
    run_id: String,
    point_id: String,
    seed: u64,
    relative_dir: PathBuf,
    attempt: u32,
    state: RunStateKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    state_digest64: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResearchExecutionState {
    schema_version: u32,
    research_id: String,
    runs: BTreeMap<String, RunState>,
}

impl ResearchExecutionState {
    const CURRENT_SCHEMA_VERSION: u32 = 1;

    fn new(manifest: &ResearchExecutionManifest) -> Self {
        let runs = manifest
            .points
            .iter()
            .flat_map(|point| {
                point.runs.iter().map(move |run| {
                    (
                        run.run_id.clone(),
                        RunState {
                            run_id: run.run_id.clone(),
                            point_id: point.point.point_id.clone(),
                            seed: run.seed,
                            relative_dir: run.relative_dir.clone(),
                            attempt: 0,
                            state: RunStateKind::Planned,
                            state_digest64: None,
                            error: None,
                        },
                    )
                })
            })
            .collect();
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            research_id: manifest.research_id.clone(),
            runs,
        }
    }

    fn validate_plan(&self, manifest: &ResearchExecutionManifest) -> Result<(), Box<dyn Error>> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported research state schema {}; supported schema is {}",
                self.schema_version,
                Self::CURRENT_SCHEMA_VERSION
            )
            .into());
        }
        if self.research_id != manifest.research_id {
            return Err(
                "research-state.json belongs to a different immutable research identity".into(),
            );
        }
        let expected = Self::new(manifest);
        if self.runs.len() != expected.runs.len() {
            return Err("research-state.json run set does not match immutable manifest".into());
        }
        for (run_id, expected_run) in expected.runs {
            let actual = self
                .runs
                .get(&run_id)
                .ok_or("research-state.json is missing an immutable planned run")?;
            if actual.run_id != expected_run.run_id
                || actual.point_id != expected_run.point_id
                || actual.seed != expected_run.seed
                || actual.relative_dir != expected_run.relative_dir
            {
                return Err(
                    "research-state.json immutable run identity fields do not match manifest"
                        .into(),
                );
            }
        }
        Ok(())
    }

    fn run_mut(&mut self, planned: &PlannedRun) -> Result<&mut RunState, Box<dyn Error>> {
        self.runs
            .get_mut(&planned.run_id)
            .ok_or_else(|| "research state is missing planned run".into())
    }

    fn begin_attempt(&mut self, planned: &PlannedRun) -> Result<u32, Box<dyn Error>> {
        let run = self.run_mut(planned)?;
        run.attempt = run
            .attempt
            .checked_add(1)
            .ok_or("research run attempt counter overflow")?;
        run.state = RunStateKind::Running;
        run.error = None;
        run.state_digest64 = None;
        Ok(run.attempt)
    }

    fn finish_completed(
        &mut self,
        planned: &PlannedRun,
        attempt: u32,
        completion: RunCompletion,
    ) -> Result<(), Box<dyn Error>> {
        let run = self.run_mut(planned)?;
        if run.attempt != attempt {
            return Err("research run attempt changed while executing".into());
        }
        run.state = RunStateKind::Completed;
        run.state_digest64 = Some(completion.state_digest64);
        run.error = None;
        Ok(())
    }

    fn finish_failed(
        &mut self,
        planned: &PlannedRun,
        attempt: u32,
        error: String,
    ) -> Result<(), Box<dyn Error>> {
        let run = self.run_mut(planned)?;
        if run.attempt != attempt {
            return Err("research run attempt changed while executing".into());
        }
        run.state = RunStateKind::Failed;
        run.state_digest64 = None;
        run.error = Some(error);
        Ok(())
    }

    fn reconcile_completed(&mut self, planned: &PlannedRun, digest: u64) {
        if let Some(run) = self.runs.get_mut(&planned.run_id) {
            run.state = RunStateKind::Completed;
            if run.attempt == 0 {
                run.attempt = 1;
            }
            run.state_digest64 = Some(digest);
            run.error = None;
        }
    }
}

fn load_or_initialize_state(
    path: &Path,
    manifest: &ResearchExecutionManifest,
    retry: bool,
) -> Result<ResearchExecutionState, Box<dyn Error>> {
    if path.exists() {
        let state: ResearchExecutionState = read_json(path)?;
        state.validate_plan(manifest)?;
        return Ok(state);
    }
    if retry {
        // A crash after publishing the immutable manifest but before the first mutable state write
        // is recoverable because the complete run plan lives in the manifest.
        let state = ResearchExecutionState::new(manifest);
        write_state(path, &state)?;
        return Ok(state);
    }
    let state = ResearchExecutionState::new(manifest);
    write_state(path, &state)?;
    Ok(state)
}

fn write_state(path: &Path, state: &ResearchExecutionState) -> Result<(), Box<dyn Error>> {
    write_json_atomic(path, state, "research execution state")
}

#[derive(Debug, Clone, Copy)]
struct RunCompletion {
    state_digest64: u64,
}

fn execute_planned_run(
    run_dir: &Path,
    planned: &PlannedRun,
) -> Result<RunCompletion, Box<dyn Error>> {
    let transaction = RunDirectoryTransaction::fresh(run_dir)?;
    let staging = transaction.staging_dir();
    let completion = match &planned.run_config.spatial {
        Some(spatial) => execute_spatial_run(staging, planned, spatial)?,
        None => execute_core_run(staging, planned)?,
    };
    bundle::validated_bundle_files(staging)?;
    transaction.commit()?;
    validate_completed_run(run_dir, planned)?;
    Ok(completion)
}

fn execute_core_run(staging: &Path, planned: &PlannedRun) -> Result<RunCompletion, Box<dyn Error>> {
    let simulation = Simulation::new(planned.run_config.experiment.clone())?;
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded()?;
    write_core_bundle(
        staging,
        &world,
        &initial_population,
        &recorded.checkpoint,
        &recorded.manifest,
    )?;
    Ok(RunCompletion {
        state_digest64: recorded.checkpoint.state_digest64,
    })
}

fn execute_spatial_run(
    staging: &Path,
    planned: &PlannedRun,
    spatial: &anthrosim_core::ResearchSpatialConfig,
) -> Result<RunCompletion, Box<dyn Error>> {
    let simulation = SpatialLandscapeSimulation::new(
        planned.run_config.experiment.clone(),
        spatial.landscape.clone(),
        spatial.mechanisms.clone(),
    )?;
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded()?;
    write_core_bundle(
        staging,
        &world,
        &initial_population,
        &recorded.checkpoint.core_checkpoint,
        &recorded.manifest.core_manifest,
    )?;
    write_json(staging.join("landscape.json"), &spatial.landscape)?;
    write_json(staging.join("spatial-mechanisms.json"), &spatial.mechanisms)?;
    write_json(staging.join("landscape-manifest.json"), &recorded.manifest)?;
    write_json(
        staging.join("landscape-checkpoint.json"),
        &recorded.checkpoint,
    )?;
    Ok(RunCompletion {
        state_digest64: recorded.checkpoint.core_checkpoint.state_digest64,
    })
}

fn write_core_bundle(
    staging: &Path,
    world: &World,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
    manifest: &anthrosim_core::RunManifest,
) -> Result<(), Box<dyn Error>> {
    write_json(staging.join("manifest.json"), manifest)?;
    write_json(staging.join("checkpoint.json"), checkpoint)?;
    write_json(staging.join("world.json"), world)?;
    write_json(staging.join("initial-population.json"), initial_population)?;
    write_json(staging.join("events.json"), &checkpoint.events)?;
    write_json(staging.join("metrics.json"), &checkpoint.metrics)?;
    if let Some(evidence) = &checkpoint.experiment.evidence {
        write_json(staging.join("evidence.json"), evidence)?;
    }
    write_json(
        staging.join("completion.json"),
        &CompletionMarker {
            schema_version: 1,
            seed: checkpoint.experiment.seed,
            status: "completed",
            manifest: "manifest.json",
        },
    )?;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CompletionMarker<'a> {
    schema_version: u32,
    seed: u64,
    status: &'a str,
    manifest: &'a str,
}

fn validate_completed_run(run_dir: &Path, planned: &PlannedRun) -> Result<u64, Box<dyn Error>> {
    bundle::validated_bundle_files(run_dir)?;
    let checkpoint: SimulationCheckpoint = read_json(&run_dir.join("checkpoint.json"))?;
    if checkpoint.experiment != planned.run_config.experiment {
        return Err(format!(
            "completed bundle {} has an ExperimentConfig different from immutable research plan",
            run_dir.display()
        )
        .into());
    }
    match &planned.run_config.spatial {
        Some(expected) => {
            let landscape: anthrosim_core::LandscapeBundle =
                read_json(&run_dir.join("landscape.json"))?;
            let mechanisms: anthrosim_core::SpatialMechanismConfig =
                read_json(&run_dir.join("spatial-mechanisms.json"))?;
            let wrapper_manifest: SpatialLandscapeRunManifest =
                read_json(&run_dir.join("landscape-manifest.json"))?;
            let wrapper_checkpoint: SpatialLandscapeCheckpoint =
                read_json(&run_dir.join("landscape-checkpoint.json"))?;
            if landscape != expected.landscape
                || mechanisms != expected.mechanisms
                || wrapper_manifest.core_manifest.experiment != planned.run_config.experiment
                || wrapper_checkpoint.core_checkpoint.experiment != planned.run_config.experiment
            {
                return Err("completed spatial bundle differs from immutable research plan".into());
            }
        }
        None => {
            if run_dir.join("landscape.json").exists() {
                return Err(
                    "non-spatial planned run unexpectedly contains spatial artifacts".into(),
                );
            }
        }
    }
    Ok(checkpoint.state_digest64)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisPoints<'a> {
    schema_version: u32,
    research_id: &'a str,
    points: Vec<AnalysisPoint<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisPoint<'a> {
    point_id: &'a str,
    index: u64,
    coordinates: &'a [ResearchCoordinate],
    resulting_configuration: &'a ResearchRunConfig,
    run_ids: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisRuns<'a> {
    schema_version: u32,
    research_id: &'a str,
    runs: Vec<AnalysisRun<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisRun<'a> {
    point_id: &'a str,
    run_id: &'a str,
    seed: u64,
    coordinates: &'a [ResearchCoordinate],
    resulting_configuration: &'a ResearchRunConfig,
    relative_dir: &'a Path,
    attempt: u32,
    state: &'a RunStateKind,
    state_digest64: Option<u64>,
    error: Option<&'a str>,
}

fn write_analysis(
    root: &Path,
    manifest: &ResearchExecutionManifest,
    state: &ResearchExecutionState,
) -> Result<(), Box<dyn Error>> {
    let points = manifest
        .points
        .iter()
        .map(|planned| AnalysisPoint {
            point_id: &planned.point.point_id,
            index: planned.point.index,
            coordinates: &planned.point.coordinates,
            resulting_configuration: &planned.point.run_config,
            run_ids: planned.runs.iter().map(|run| run.run_id.as_str()).collect(),
        })
        .collect();
    let mut runs = Vec::new();
    for planned_point in &manifest.points {
        for planned in &planned_point.runs {
            let run_state = state
                .runs
                .get(&planned.run_id)
                .ok_or("research state is missing a run while deriving analysis")?;
            runs.push(AnalysisRun {
                point_id: &planned_point.point.point_id,
                run_id: &planned.run_id,
                seed: planned.seed,
                coordinates: &planned_point.point.coordinates,
                resulting_configuration: &planned.run_config,
                relative_dir: &planned.relative_dir,
                attempt: run_state.attempt,
                state: &run_state.state,
                state_digest64: run_state.state_digest64,
                error: run_state.error.as_deref(),
            });
        }
    }
    write_json_atomic(
        &root.join("analysis/points.json"),
        &AnalysisPoints {
            schema_version: 1,
            research_id: &manifest.research_id,
            points,
        },
        "research point analysis",
    )?;
    write_json_atomic(
        &root.join("analysis/runs.json"),
        &AnalysisRuns {
            schema_version: 1,
            research_id: &manifest.research_id,
            runs,
        },
        "research run analysis",
    )?;
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    let content = bundle::artifact_fs::read_to_string(path, "AnthroSim research JSON input")?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: Serialize + ?Sized>(path: PathBuf, value: &T) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn write_json_atomic<T: Serialize + ?Sized>(
    path: &Path,
    value: &T,
    role: &str,
) -> Result<(), Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    bundle::artifact_fs::atomic_write(path, &bytes, role)?;
    Ok(())
}

fn reject_symlink(path: &Path, role: &str) -> Result<(), Box<dyn Error>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("{role} may not be a symbolic link: {}", path.display()).into())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use anthrosim_core::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
        WorldConfig,
    };
    use serde_json::Value;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

    fn tiny_definition() -> ResearchExperimentDefinition {
        let mut experiment = ExperimentConfig::new(101, 1)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(20)
                    .with_target_household_size(5)
                    .with_max_person_records(100),
            )
            .with_demography(DemographyConfig::synthetic_validation_v1())
            .with_resources(ResourceConfig::synthetic_validation_v1())
            .with_migration(MigrationConfig::synthetic_validation_v1());
        experiment
            .resources
            .max_scarcity_mortality_probability_per_million = 0;
        ResearchExperimentDefinition {
            schema_version: 1,
            seeds: vec![101],
            base: ResearchRunConfig {
                experiment,
                spatial: None,
            },
            dimensions: vec![anthrosim_core::ResearchDimension {
                id: "m3_periods_per_year".to_owned(),
                kind: anthrosim_core::ResearchDimensionKind::Numeric,
                path: "/experiment/resources/periodsPerYear".to_owned(),
                values: vec![Value::from(4), Value::from(12)],
            }],
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-research-{name}-{}-{id}",
            std::process::id()
        ))
    }

    #[test]
    fn exact_plan_executes_and_retry_reconciles_without_reexecution() {
        let root = temp_root("retry");
        let definition_path = root.with_extension("json");
        fs::write(
            &definition_path,
            serde_json::to_vec_pretty(&tiny_definition()).unwrap(),
        )
        .unwrap();
        execute(&Cli {
            definition: definition_path.clone(),
            run_dir: root.clone(),
            retry: false,
        })
        .unwrap();
        let before: ResearchExecutionState = read_json(&root.join(RESEARCH_STATE)).unwrap();
        assert!(
            before
                .runs
                .values()
                .all(|run| run.state == RunStateKind::Completed)
        );
        assert!(before.runs.values().all(|run| run.attempt == 1));

        execute(&Cli {
            definition: definition_path.clone(),
            run_dir: root.clone(),
            retry: true,
        })
        .unwrap();
        let after: ResearchExecutionState = read_json(&root.join(RESEARCH_STATE)).unwrap();
        assert!(
            after
                .runs
                .values()
                .all(|run| run.state == RunStateKind::Completed)
        );
        assert!(after.runs.values().all(|run| run.attempt == 1));

        let runs: serde_json::Value = read_json(&root.join("analysis/runs.json")).unwrap();
        let rows = runs["runs"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["coordinates"][0]["id"], "m3_periods_per_year");
        assert!(rows[0]["resultingConfiguration"]["experiment"]["demography"].is_object());
        assert!(rows[0]["resultingConfiguration"]["experiment"]["migration"].is_object());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn invalid_definition_fails_before_research_root_is_created() {
        let root = temp_root("invalid");
        let definition_path = root.with_extension("json");
        let mut definition = tiny_definition();
        definition.dimensions[0].path = "/experiment/resources/notAField".to_owned();
        fs::write(
            &definition_path,
            serde_json::to_vec_pretty(&definition).unwrap(),
        )
        .unwrap();
        assert!(
            execute(&Cli {
                definition: definition_path.clone(),
                run_dir: root.clone(),
                retry: false,
            })
            .is_err()
        );
        assert!(!root.exists());
        fs::remove_file(definition_path).unwrap();
    }
}
