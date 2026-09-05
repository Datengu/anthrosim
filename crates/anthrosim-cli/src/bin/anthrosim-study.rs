use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use anthrosim_core::{
    ResearchCoordinate, ResearchExperimentDefinition, ResearchPoint, ResearchRunConfig,
    RunManifest, SimulationCheckpoint, SourceRevisionIdentity, SpatialLandscapeCheckpoint,
    SpatialLandscapeRunManifest, StudyProtocol, StudyScientificStatus, research_run_identity,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

#[allow(dead_code)]
#[path = "../bundle.rs"]
mod bundle;

const STUDY_PLAN: &str = "study-plan.json";
const STUDY_MANIFEST: &str = "study-manifest.json";
const STUDY_PROTOCOL: &str = "study-protocol.json";
const RESEARCH_DEFINITION: &str = "research-definition.json";
const RESEARCH_DIR: &str = "research";
const RESULT_BINDING: &str = "study-result-binding.json";
const OBSERVABLE_SUPPORT_BINDING_PREFIX: &str = "observable-support-plan-v1:";
const OBSERVABLE_SUPPORT_REQUIREMENT_KIND: &str = "observable_support_sensitivity";

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-study",
    about = "Freeze a study protocol before execution and bind it to AnthroSim research results"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Freeze a protocol and research definition into a new immutable study root.
    Prepare {
        #[arg(long)]
        protocol: PathBuf,
        #[arg(long)]
        definition: PathBuf,
        #[arg(long)]
        study_dir: PathBuf,
    },
    /// Verify the frozen study root and bind the completed research result to its protocol.
    Finalize {
        #[arg(long)]
        study_dir: PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command {
        Command::Prepare {
            protocol,
            definition,
            study_dir,
        } => prepare(&study_dir, &protocol, &definition).map(|plan| {
            println!("{}", plan.study_execution_id);
            eprintln!(
                "run the frozen definition with: anthrosim-research --definition {} --run-dir {}",
                study_dir.join(RESEARCH_DEFINITION).display(),
                study_dir.join(RESEARCH_DIR).display()
            );
        }),
        Command::Finalize { study_dir } => finalize(&study_dir).map(|binding| {
            println!("{}", binding.result_identity);
        }),
    };

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-study: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudyExecutionPlan {
    schema_version: u32,
    study_execution_id: String,
    protocol_identity: String,
    bound_before_execution: bool,
    confirmatory_pre_result_claim_eligible: bool,
    protocol: StudyProtocol,
    definition_identity: String,
    source: SourceRevisionIdentity,
    definition: ResearchExperimentDefinition,
    research_relative_dir: PathBuf,
}

impl StudyExecutionPlan {
    const CURRENT_SCHEMA_VERSION: u32 = 1;

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported study execution plan schema {}; supported schema is {}",
                self.schema_version,
                Self::CURRENT_SCHEMA_VERSION
            )
            .into());
        }
        self.protocol.validate()?;
        self.definition.validate()?;
        if self.protocol_identity != self.protocol.identity()? {
            return Err("study plan protocol identity does not match embedded protocol".into());
        }
        if self.definition_identity != self.definition.identity()? {
            return Err("study plan definition identity does not match embedded definition".into());
        }
        if self.research_relative_dir != Path::new(RESEARCH_DIR) {
            return Err(
                "study plan researchRelativeDir is not the schema-v1 fixed research root".into(),
            );
        }
        if !self.bound_before_execution {
            return Err("study plan must record protocol binding before execution".into());
        }
        if self.confirmatory_pre_result_claim_eligible
            != self.protocol.confirmatory_pre_result_claim_eligible()
        {
            return Err("study plan confirmatory eligibility differs from protocol amendment/status semantics".into());
        }
        let expected_execution_id = study_execution_identity(
            &self.protocol_identity,
            &self.definition_identity,
            &self.source,
        )?;
        if self.study_execution_id != expected_execution_id {
            return Err(
                "study execution identity does not match frozen protocol/definition/source".into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudyResultBinding {
    schema_version: u32,
    result_identity: String,
    study_execution_id: String,
    protocol_identity: String,
    protocol_revision: u32,
    study_id: String,
    scientific_status: StudyScientificStatus,
    bound_before_execution: bool,
    confirmatory_pre_result_claim_eligible: bool,
    definition_identity: String,
    research_id: String,
    source: SourceRevisionIdentity,
    research_relative_dir: PathBuf,
    run_counts: StudyRunCounts,
    result_artifacts: Vec<StudyResultArtifact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    analysis_requirements: Vec<StudyAnalysisRequirement>,
}

impl StudyResultBinding {
    const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudyRunCounts {
    completed: u64,
    failed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudyResultArtifact {
    path: PathBuf,
    digest64: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StudyAnalysisRequirement {
    kind: String,
    identity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResearchManifestView {
    schema_version: u32,
    research_id: String,
    definition_identity: String,
    source: SourceRevisionIdentity,
    definition: ResearchExperimentDefinition,
    points: Vec<PlannedPointView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlannedPointView {
    point: ResearchPoint,
    runs: Vec<PlannedRunView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlannedRunView {
    seed: u64,
    run_id: String,
    relative_dir: PathBuf,
    run_config: ResearchRunConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResearchStateView {
    schema_version: u32,
    research_id: String,
    runs: BTreeMap<String, ResearchRunStateView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ResearchRunStateView {
    run_id: String,
    point_id: String,
    seed: u64,
    relative_dir: PathBuf,
    attempt: u32,
    state: String,
    #[serde(default)]
    state_digest64: Option<u64>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisPointsView<'a> {
    schema_version: u32,
    research_id: &'a str,
    points: Vec<AnalysisPointView<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisPointView<'a> {
    point_id: &'a str,
    index: u64,
    coordinates: &'a [ResearchCoordinate],
    resulting_configuration: &'a ResearchRunConfig,
    run_ids: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisRunsView<'a> {
    schema_version: u32,
    research_id: &'a str,
    runs: Vec<AnalysisRunView<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisRunView<'a> {
    point_id: &'a str,
    run_id: &'a str,
    seed: u64,
    coordinates: &'a [ResearchCoordinate],
    resulting_configuration: &'a ResearchRunConfig,
    relative_dir: &'a Path,
    attempt: u32,
    state: &'a str,
    state_digest64: Option<u64>,
    error: Option<&'a str>,
}

fn prepare(
    study_dir: &Path,
    protocol_path: &Path,
    definition_path: &Path,
) -> Result<StudyExecutionPlan, Box<dyn Error>> {
    reject_symlink(study_dir, "study root")?;
    if study_dir.exists() {
        if !study_dir.is_dir() {
            return Err(format!("study root is not a directory: {}", study_dir.display()).into());
        }
        if fs::read_dir(study_dir)?.next().transpose()?.is_some() {
            return Err(format!(
                "study root is not empty; protocol revisions require a new immutable study root: {}",
                study_dir.display()
            )
            .into());
        }
    } else {
        fs::create_dir_all(study_dir)?;
    }

    let protocol: StudyProtocol = read_json_regular(protocol_path, "study protocol")?;
    protocol.validate()?;
    let definition: ResearchExperimentDefinition =
        read_json_regular(definition_path, "research experiment definition")?;
    definition.validate()?;

    let protocol_identity = protocol.identity()?;
    let definition_identity = definition.identity()?;
    let source = SourceRevisionIdentity::current();
    let study_execution_id =
        study_execution_identity(&protocol_identity, &definition_identity, &source)?;
    let plan = StudyExecutionPlan {
        schema_version: StudyExecutionPlan::CURRENT_SCHEMA_VERSION,
        study_execution_id,
        protocol_identity,
        bound_before_execution: true,
        confirmatory_pre_result_claim_eligible: protocol.confirmatory_pre_result_claim_eligible(),
        protocol,
        definition_identity,
        source,
        definition,
        research_relative_dir: PathBuf::from(RESEARCH_DIR),
    };
    plan.validate()?;

    write_json_atomic(
        &study_dir.join(RESEARCH_DEFINITION),
        &plan.definition,
        "frozen research definition",
    )?;
    write_json_atomic(
        &study_dir.join(STUDY_PROTOCOL),
        &plan.protocol,
        "frozen study protocol",
    )?;
    write_json_atomic(&study_dir.join(STUDY_PLAN), &plan, "immutable study plan")?;
    write_json_atomic(
        &study_dir.join(STUDY_MANIFEST),
        &plan,
        "immutable study manifest",
    )?;
    Ok(plan)
}

fn finalize(study_dir: &Path) -> Result<StudyResultBinding, Box<dyn Error>> {
    let plan = load_frozen_plan(study_dir)?;
    let research_root = study_dir.join(&plan.research_relative_dir);
    reject_symlink(&research_root, "research root")?;
    if !research_root.is_dir() {
        return Err(format!(
            "bound research root is missing; execute anthrosim-research first: {}",
            research_root.display()
        )
        .into());
    }

    let research_manifest_value: Value = read_json_regular(
        &research_root.join("research-manifest.json"),
        "research manifest",
    )?;
    let research_plan_value: Value =
        read_json_regular(&research_root.join("research-plan.json"), "research plan")?;
    if research_manifest_value != research_plan_value {
        return Err("research-manifest.json and research-plan.json do not contain the same immutable research plan".into());
    }
    let research: ResearchManifestView = serde_json::from_value(research_manifest_value)?;
    let expected_research = expected_research_manifest(&plan)?;
    if research != expected_research {
        return Err(
            "research manifest/plan do not match the deterministic expansion of the frozen study definition/source"
                .into(),
        );
    }

    let state: ResearchStateView =
        read_json_regular(&research_root.join("research-state.json"), "research state")?;
    if state.schema_version != 1 || state.research_id != research.research_id {
        return Err("research-state.json does not match the completed research execution".into());
    }
    let run_counts = validate_research_state(&research_root, &research, &state)?;

    let expected_points = expected_analysis_points(&research)?;
    let expected_runs = expected_analysis_runs(&research, &state)?;
    let result_paths = [
        (
            PathBuf::from(RESEARCH_DIR).join("analysis/points.json"),
            expected_points,
        ),
        (
            PathBuf::from(RESEARCH_DIR).join("analysis/runs.json"),
            expected_runs,
        ),
    ];
    let mut result_artifacts = Vec::with_capacity(result_paths.len());
    for (relative, expected) in result_paths {
        let path = study_dir.join(&relative);
        if !regular_file_exists(&path)? {
            return Err(format!(
                "required research result artifact is missing: {}",
                path.display()
            )
            .into());
        }
        let actual: Value = read_json_regular(&path, "research analysis artifact")?;
        if actual != expected {
            return Err(format!(
                "research analysis artifact differs from immutable research plan/state: {}",
                path.display()
            )
            .into());
        }
        result_artifacts.push(StudyResultArtifact {
            path: relative,
            digest64: fnv1a64(&fs::read(path)?),
        });
    }

    let analysis_requirements = study_analysis_requirements(&plan.protocol)?;
    let mut binding = StudyResultBinding {
        schema_version: StudyResultBinding::CURRENT_SCHEMA_VERSION,
        result_identity: String::new(),
        study_execution_id: plan.study_execution_id.clone(),
        protocol_identity: plan.protocol_identity.clone(),
        protocol_revision: plan.protocol.protocol_revision,
        study_id: plan.protocol.study_id.clone(),
        scientific_status: plan.protocol.status,
        bound_before_execution: plan.bound_before_execution,
        confirmatory_pre_result_claim_eligible: plan.confirmatory_pre_result_claim_eligible,
        definition_identity: plan.definition_identity.clone(),
        research_id: research.research_id,
        source: plan.source.clone(),
        research_relative_dir: plan.research_relative_dir.clone(),
        run_counts,
        result_artifacts,
        analysis_requirements,
    };
    binding.result_identity = result_binding_identity(&binding)?;

    let path = study_dir.join(RESULT_BINDING);
    if regular_file_exists(&path)? {
        let existing: StudyResultBinding = read_json_regular(&path, "study result binding")?;
        if existing != binding {
            return Err("existing study-result-binding.json differs from the frozen protocol/research result; create a new study revision rather than rewriting result provenance".into());
        }
    } else {
        write_json_atomic(&path, &binding, "study result binding")?;
    }
    Ok(binding)
}

fn expected_research_manifest(
    plan: &StudyExecutionPlan,
) -> Result<ResearchManifestView, Box<dyn Error>> {
    let points = plan
        .definition
        .expand()?
        .into_iter()
        .map(|point| {
            let runs = plan
                .definition
                .seeds
                .iter()
                .enumerate()
                .map(|(seed_index, seed)| {
                    let run_config = point.run_config.for_seed(*seed);
                    let run_id = research_run_identity(&point.point_id, &run_config, &plan.source)?;
                    Ok(PlannedRunView {
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
            Ok(PlannedPointView { point, runs })
        })
        .collect::<Result<Vec<_>, anthrosim_core::ResearchExperimentError>>()?;
    Ok(ResearchManifestView {
        schema_version: 1,
        research_id: research_execution_identity(&plan.definition_identity, &plan.source)?,
        definition_identity: plan.definition_identity.clone(),
        source: plan.source.clone(),
        definition: plan.definition.clone(),
        points,
    })
}

fn validate_research_state(
    research_root: &Path,
    research: &ResearchManifestView,
    state: &ResearchStateView,
) -> Result<StudyRunCounts, Box<dyn Error>> {
    let expected_run_count = research
        .points
        .iter()
        .try_fold(0_usize, |count, point| count.checked_add(point.runs.len()))
        .ok_or("immutable research run count overflow")?;
    if state.runs.len() != expected_run_count {
        return Err("research-state.json run set does not match immutable research plan".into());
    }

    let mut completed = 0_u64;
    let mut failed = 0_u64;
    for planned_point in &research.points {
        for planned in &planned_point.runs {
            let run = state
                .runs
                .get(&planned.run_id)
                .ok_or("research-state.json is missing an immutable planned run")?;
            if run.run_id != planned.run_id
                || run.point_id != planned_point.point.point_id
                || run.seed != planned.seed
                || run.relative_dir != planned.relative_dir
            {
                return Err(
                    "research-state.json immutable run identity fields do not match research plan"
                        .into(),
                );
            }
            let run_dir = research_root.join(&planned.relative_dir);
            match run.state.as_str() {
                "completed" => {
                    if run.attempt == 0 || run.state_digest64.is_none() || run.error.is_some() {
                        return Err(
                            "completed research state row has invalid attempt/digest/error fields"
                                .into(),
                        );
                    }
                    let actual_digest =
                        validate_completed_run(&run_dir, planned, &research.source)?;
                    if run.state_digest64 != Some(actual_digest) {
                        return Err(
                            "completed research state digest differs from validated child bundle"
                                .into(),
                        );
                    }
                    completed = completed
                        .checked_add(1)
                        .ok_or("completed run count overflow")?;
                }
                "failed" => {
                    if run.attempt == 0 || run.state_digest64.is_some() || run.error.is_none() {
                        return Err(
                            "failed research state row has invalid attempt/digest/error fields"
                                .into(),
                        );
                    }
                    if path_entry_exists(&run_dir)? {
                        return Err(
                            "failed research state row unexpectedly has a published child run directory"
                                .into(),
                        );
                    }
                    failed = failed.checked_add(1).ok_or("failed run count overflow")?;
                }
                "planned" | "running" => {
                    return Err(format!(
                        "research execution is not finished; run {} remains {}",
                        planned.run_id, run.state
                    )
                    .into());
                }
                other => {
                    return Err(format!("research state contains unknown run state {other}").into());
                }
            }
        }
    }
    Ok(StudyRunCounts { completed, failed })
}

fn validate_completed_run(
    run_dir: &Path,
    planned: &PlannedRunView,
    source: &SourceRevisionIdentity,
) -> Result<u64, Box<dyn Error>> {
    bundle::validated_bundle_files(run_dir)?;
    let run_manifest: RunManifest =
        read_json_regular(&run_dir.join("manifest.json"), "completed run manifest")?;
    let checkpoint: SimulationCheckpoint =
        read_json_regular(&run_dir.join("checkpoint.json"), "completed run checkpoint")?;
    let source_mismatch = run_manifest.model_version != source.model_version
        || run_manifest.model_semantics_id != source.model_semantics_id
        || run_manifest.git_commit != source.git_commit
        || checkpoint.model_version != source.model_version
        || checkpoint.model_semantics_id != source.model_semantics_id
        || checkpoint.git_commit != source.git_commit;
    if source_mismatch
        || run_manifest.experiment != planned.run_config.experiment
        || checkpoint.experiment != planned.run_config.experiment
        || checkpoint.state_digest64 != run_manifest.state_digest64
    {
        return Err(format!(
            "completed bundle {} differs from immutable research run configuration/source",
            run_dir.display()
        )
        .into());
    }
    match &planned.run_config.spatial {
        Some(expected) => {
            let landscape: anthrosim_core::LandscapeBundle =
                read_json_regular(&run_dir.join("landscape.json"), "bound landscape")?;
            let mechanisms: anthrosim_core::SpatialMechanismConfig = read_json_regular(
                &run_dir.join("spatial-mechanisms.json"),
                "bound spatial mechanisms",
            )?;
            let wrapper_manifest: SpatialLandscapeRunManifest = read_json_regular(
                &run_dir.join("landscape-manifest.json"),
                "spatial landscape manifest",
            )?;
            let wrapper_checkpoint: SpatialLandscapeCheckpoint = read_json_regular(
                &run_dir.join("landscape-checkpoint.json"),
                "spatial landscape checkpoint",
            )?;
            if landscape != expected.landscape
                || mechanisms != expected.mechanisms
                || wrapper_manifest.core_manifest != run_manifest
                || wrapper_checkpoint.core_checkpoint != checkpoint
                || wrapper_checkpoint.spatial.spatial_model_semantics_id
                    != expected.spatial_model_semantics_id
            {
                return Err("completed spatial bundle differs from immutable research plan".into());
            }
        }
        None => {
            for name in [
                "landscape.json",
                "landscape-manifest.json",
                "landscape-checkpoint.json",
                "spatial-mechanisms.json",
            ] {
                if path_entry_exists(&run_dir.join(name))? {
                    return Err(
                        "non-spatial planned run unexpectedly contains spatial artifacts".into(),
                    );
                }
            }
        }
    }
    Ok(checkpoint.state_digest64)
}

fn expected_analysis_points(research: &ResearchManifestView) -> Result<Value, Box<dyn Error>> {
    let points = research
        .points
        .iter()
        .map(|planned| AnalysisPointView {
            point_id: &planned.point.point_id,
            index: planned.point.index,
            coordinates: &planned.point.coordinates,
            resulting_configuration: &planned.point.run_config,
            run_ids: planned.runs.iter().map(|run| run.run_id.as_str()).collect(),
        })
        .collect();
    Ok(serde_json::to_value(AnalysisPointsView {
        schema_version: 1,
        research_id: &research.research_id,
        points,
    })?)
}

fn expected_analysis_runs(
    research: &ResearchManifestView,
    state: &ResearchStateView,
) -> Result<Value, Box<dyn Error>> {
    let mut runs = Vec::new();
    for planned_point in &research.points {
        for planned in &planned_point.runs {
            let run_state = state
                .runs
                .get(&planned.run_id)
                .ok_or("research state is missing a run while deriving canonical analysis")?;
            runs.push(AnalysisRunView {
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
    Ok(serde_json::to_value(AnalysisRunsView {
        schema_version: 1,
        research_id: &research.research_id,
        runs,
    })?)
}

fn load_frozen_plan(study_dir: &Path) -> Result<StudyExecutionPlan, Box<dyn Error>> {
    reject_symlink(study_dir, "study root")?;
    if !study_dir.is_dir() {
        return Err(format!("study root is missing: {}", study_dir.display()).into());
    }
    let plan: StudyExecutionPlan = read_json_regular(&study_dir.join(STUDY_PLAN), "study plan")?;
    let manifest: StudyExecutionPlan =
        read_json_regular(&study_dir.join(STUDY_MANIFEST), "study manifest")?;
    if plan != manifest {
        return Err(
            "study-plan.json and study-manifest.json do not contain the same immutable plan".into(),
        );
    }
    plan.validate()?;

    let protocol: StudyProtocol =
        read_json_regular(&study_dir.join(STUDY_PROTOCOL), "frozen study protocol")?;
    let definition: ResearchExperimentDefinition = read_json_regular(
        &study_dir.join(RESEARCH_DEFINITION),
        "frozen research definition",
    )?;
    if protocol != plan.protocol || definition != plan.definition {
        return Err(
            "frozen study protocol/definition copies differ from immutable study plan".into(),
        );
    }
    if protocol.identity()? != plan.protocol_identity
        || definition.identity()? != plan.definition_identity
    {
        return Err(
            "frozen study protocol/definition identity does not match immutable study plan".into(),
        );
    }
    Ok(plan)
}

fn study_analysis_requirements(
    protocol: &StudyProtocol,
) -> Result<Vec<StudyAnalysisRequirement>, Box<dyn Error>> {
    let mut requirements = Vec::new();
    for observable in &protocol.observables {
        let normalized = observable.interpretation.replace(';', " ");
        for token in normalized.split_whitespace() {
            let Some(identity) = token.strip_prefix(OBSERVABLE_SUPPORT_BINDING_PREFIX) else {
                continue;
            };
            if !identity.starts_with("observable-support-plan-v1-sha256-")
                || identity.len() <= "observable-support-plan-v1-sha256-".len()
            {
                return Err(format!(
                    "observable {} has malformed observable-support plan binding",
                    observable.id
                )
                .into());
            }
            requirements.push(StudyAnalysisRequirement {
                kind: OBSERVABLE_SUPPORT_REQUIREMENT_KIND.to_owned(),
                identity: identity.to_owned(),
            });
        }
    }
    requirements.sort();
    requirements.dedup();
    Ok(requirements)
}

fn study_execution_identity(
    protocol_identity: &str,
    definition_identity: &str,
    source: &SourceRevisionIdentity,
) -> Result<String, Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        protocol_identity: &'a str,
        definition_identity: &'a str,
        source: &'a SourceRevisionIdentity,
    }
    stable_identity(
        "study-execution-v1",
        &Identity {
            schema_version: 1,
            protocol_identity,
            definition_identity,
            source,
        },
    )
}

fn research_execution_identity(
    definition_identity: &str,
    source: &SourceRevisionIdentity,
) -> Result<String, Box<dyn Error>> {
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
    })?;
    Ok(format!("research-execution-v1-{:016x}", fnv1a64(&bytes)))
}

fn result_binding_identity(binding: &StudyResultBinding) -> Result<String, Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        study_execution_id: &'a str,
        protocol_identity: &'a str,
        protocol_revision: u32,
        study_id: &'a str,
        scientific_status: StudyScientificStatus,
        bound_before_execution: bool,
        confirmatory_pre_result_claim_eligible: bool,
        definition_identity: &'a str,
        research_id: &'a str,
        source: &'a SourceRevisionIdentity,
        research_relative_dir: &'a Path,
        run_counts: StudyRunCounts,
        result_artifacts: &'a [StudyResultArtifact],
    }

    let base = Identity {
        schema_version: binding.schema_version,
        study_execution_id: &binding.study_execution_id,
        protocol_identity: &binding.protocol_identity,
        protocol_revision: binding.protocol_revision,
        study_id: &binding.study_id,
        scientific_status: binding.scientific_status,
        bound_before_execution: binding.bound_before_execution,
        confirmatory_pre_result_claim_eligible: binding.confirmatory_pre_result_claim_eligible,
        definition_identity: &binding.definition_identity,
        research_id: &binding.research_id,
        source: &binding.source,
        research_relative_dir: &binding.research_relative_dir,
        run_counts: binding.run_counts,
        result_artifacts: &binding.result_artifacts,
    };
    if binding.analysis_requirements.is_empty() {
        return stable_identity("study-result-v1", &base);
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct IdentityWithRequirements<'a> {
        #[serde(flatten)]
        base: Identity<'a>,
        analysis_requirements: &'a [StudyAnalysisRequirement],
    }
    stable_identity(
        "study-result-v1",
        &IdentityWithRequirements {
            base,
            analysis_requirements: &binding.analysis_requirements,
        },
    )
}

fn stable_identity<T: Serialize>(prefix: &str, value: &T) -> Result<String, Box<dyn Error>> {
    let encoded = serde_json::to_value(value)?;
    let bytes = canonical_json_bytes(&encoded)?;
    Ok(format!("{prefix}-{:016x}", fnv1a64(&bytes)))
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
            Value::Object(values) => {
                let mut keys: Vec<_> = values.keys().collect();
                keys.sort_unstable();
                let mut output = serde_json::Map::new();
                for key in keys {
                    output.insert(key.clone(), canonicalize(&values[key]));
                }
                Value::Object(output)
            }
            _ => value.clone(),
        }
    }
    Ok(serde_json::to_vec(&canonicalize(value))?)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn read_json_regular<T: DeserializeOwned>(path: &Path, role: &str) -> Result<T, Box<dyn Error>> {
    if !regular_file_exists(path)? {
        return Err(format!(
            "{role} is missing or is not a regular file: {}",
            path.display()
        )
        .into());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn regular_file_exists(path: &Path) -> Result<bool, Box<dyn Error>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(format!("artifact may not be a symbolic link: {}", path.display()).into())
        }
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn path_entry_exists(path: &Path) -> Result<bool, Box<dyn Error>> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use anthrosim_core::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResearchRunConfig,
        ResourceConfig, Simulation, StudyAnalysisWindow, StudyAnalysisWindowSelectionRule,
        StudyComparison, StudyEnsemblePolicy, StudyHypothesis, StudyHypothesisKind,
        StudyManipulationCheck, StudyObservable, StudyObservableRole, StudyRunHandling,
        StudyUncertaintyPlan, WorldConfig,
    };
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
            schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
            seeds: vec![101, 102],
            base: ResearchRunConfig {
                experiment,
                spatial: None,
            },
            dimensions: vec![],
        }
    }

    fn tiny_protocol() -> StudyProtocol {
        StudyProtocol {
            schema_version: StudyProtocol::CURRENT_SCHEMA_VERSION,
            protocol_revision: 1,
            study_id: "tiny-confirmatory-study".to_owned(),
            status: StudyScientificStatus::Confirmatory,
            research_question: "Does the treatment alter the primary observable?".to_owned(),
            applicability_domain: "Synthetic test domain.".to_owned(),
            hypotheses: vec![
                StudyHypothesis {
                    id: "null".to_owned(),
                    kind: StudyHypothesisKind::NullModel,
                    statement: "No treatment effect.".to_owned(),
                },
                StudyHypothesis {
                    id: "effect".to_owned(),
                    kind: StudyHypothesisKind::Alternative,
                    statement: "Treatment changes the observable.".to_owned(),
                },
            ],
            analysis_windows: vec![StudyAnalysisWindow {
                id: "full".to_owned(),
                analysis_start_day: 0,
                analysis_end_day_inclusive: None,
                selection_rule: StudyAnalysisWindowSelectionRule::InitialStateInScope,
                rationale: "The initialized state is part of this synthetic question.".to_owned(),
            }],
            observables: vec![StudyObservable {
                id: "terminal_population".to_owned(),
                role: StudyObservableRole::Primary,
                source: "metrics.population.livingPeople".to_owned(),
                analysis_window_id: "full".to_owned(),
                interpretation: "Compare the terminal population across declared arms.".to_owned(),
            }],
            comparisons: vec![StudyComparison {
                id: "primary".to_owned(),
                hypothesis_ids: vec!["null".to_owned(), "effect".to_owned()],
                observable_ids: vec!["terminal_population".to_owned()],
                prediction: "The treatment changes terminal population.".to_owned(),
                decision_criterion: "Use the exact predeclared paired contrast.".to_owned(),
            }],
            evidence_roles: vec![],
            uncertainty: StudyUncertaintyPlan {
                parameter_uncertainty: vec![],
                structural_uncertainty: vec![],
            },
            ensemble_policy: StudyEnsemblePolicy {
                seed_policy: "Use exactly the seeds in the frozen research definition.".to_owned(),
                pairing_policy: "Pair equal seeds across contrasts.".to_owned(),
                replication_policy: "Do not add seeds adaptively after inspection.".to_owned(),
            },
            run_handling: StudyRunHandling {
                stopping_rules: vec!["Use declared simulator stop reasons.".to_owned()],
                exclusion_rules: vec!["No post-hoc exclusions.".to_owned()],
                censoring_rules: vec!["Report operational failures separately.".to_owned()],
            },
            sensitivity_plan: vec![],
            equifinality_plan: vec![],
            manipulation_checks: vec![StudyManipulationCheck {
                id: "realized".to_owned(),
                mechanism: "synthetic treatment".to_owned(),
                criterion: "Declared treatment differs between arms.".to_owned(),
                failure_handling: "Do not claim a realized causal contrast.".to_owned(),
            }],
            analysis_method: "Paired descriptive contrast.".to_owned(),
            multiplicity_policy: "One primary contrast.".to_owned(),
            held_out_corroboration: vec![],
            permitted_interpretations: vec!["Synthetic test only.".to_owned()],
            prohibited_interpretations: vec!["Empirical validation.".to_owned()],
            amendment: None,
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-study-{name}-{}-{id}",
            std::process::id()
        ))
    }

    fn write_json<T: Serialize>(path: &Path, value: &T) {
        fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    fn write_completed_core_bundle(run_dir: &Path, run_config: &ResearchRunConfig) -> u64 {
        assert!(run_config.spatial.is_none());
        fs::create_dir_all(run_dir).unwrap();
        let simulation = Simulation::new(run_config.experiment.clone()).unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();
        write_json(&run_dir.join("manifest.json"), &recorded.manifest);
        write_json(&run_dir.join("checkpoint.json"), &recorded.checkpoint);
        write_json(&run_dir.join("world.json"), &world);
        write_json(
            &run_dir.join("initial-population.json"),
            &initial_population,
        );
        write_json(&run_dir.join("events.json"), &recorded.checkpoint.events);
        write_json(&run_dir.join("metrics.json"), &recorded.checkpoint.metrics);
        bundle::validated_bundle_files(run_dir).unwrap();
        recorded.checkpoint.state_digest64
    }

    fn fake_research_root(root: &Path, plan: &StudyExecutionPlan, run_state: &str) {
        let research_root = root.join(RESEARCH_DIR);
        fs::create_dir_all(research_root.join("analysis")).unwrap();
        let research = expected_research_manifest(plan).unwrap();
        write_json(&research_root.join("research-manifest.json"), &research);
        write_json(&research_root.join("research-plan.json"), &research);

        let mut state_runs = BTreeMap::new();
        for planned_point in &research.points {
            for planned in &planned_point.runs {
                let run_dir = research_root.join(&planned.relative_dir);
                let (attempt, state_digest64, error) = match run_state {
                    "completed" => (
                        1,
                        Some(write_completed_core_bundle(&run_dir, &planned.run_config)),
                        None,
                    ),
                    "failed" => (1, None, Some("synthetic failure".to_owned())),
                    _ => (1, None, None),
                };
                state_runs.insert(
                    planned.run_id.clone(),
                    ResearchRunStateView {
                        run_id: planned.run_id.clone(),
                        point_id: planned_point.point.point_id.clone(),
                        seed: planned.seed,
                        relative_dir: planned.relative_dir.clone(),
                        attempt,
                        state: run_state.to_owned(),
                        state_digest64,
                        error,
                    },
                );
            }
        }
        let state = ResearchStateView {
            schema_version: 1,
            research_id: research.research_id.clone(),
            runs: state_runs,
        };
        write_json(&research_root.join("research-state.json"), &state);
        write_json(
            &research_root.join("analysis/points.json"),
            &expected_analysis_points(&research).unwrap(),
        );
        write_json(
            &research_root.join("analysis/runs.json"),
            &expected_analysis_runs(&research, &state).unwrap(),
        );
    }

    #[test]
    fn research_execution_identity_matches_runner_field_order_contract() {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RunnerIdentity<'a> {
            schema_version: u32,
            definition_identity: &'a str,
            source: &'a SourceRevisionIdentity,
        }

        let source = SourceRevisionIdentity::current();
        let definition_identity = "research-definition-v1-test";
        let bytes = serde_json::to_vec(&RunnerIdentity {
            schema_version: 1,
            definition_identity,
            source: &source,
        })
        .unwrap();
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .starts_with("{\"schemaVersion\":1,\"definitionIdentity\":")
        );
        assert_eq!(
            research_execution_identity(definition_identity, &source).unwrap(),
            format!("research-execution-v1-{:016x}", fnv1a64(&bytes))
        );
    }

    #[test]
    fn prepare_freezes_exact_protocol_definition_and_identity() {
        let root = temp_root("prepare");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        write_json(&protocol_path, &tiny_protocol());
        write_json(&definition_path, &tiny_definition());

        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();
        assert!(plan.study_execution_id.starts_with("study-execution-v1-"));
        assert!(plan.protocol_identity.starts_with("study-protocol-v1-"));
        assert!(plan.confirmatory_pre_result_claim_eligible);
        assert_eq!(
            read_json_regular::<StudyProtocol>(&root.join(STUDY_PROTOCOL), "protocol").unwrap(),
            plan.protocol
        );
        assert_eq!(
            read_json_regular::<ResearchExperimentDefinition>(
                &root.join(RESEARCH_DEFINITION),
                "definition"
            )
            .unwrap(),
            plan.definition
        );
        assert!(prepare(&root, &protocol_path, &definition_path).is_err());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn protocol_change_changes_study_identity_not_simulation_definition_identity() {
        let root_a = temp_root("identity-a");
        let root_b = temp_root("identity-b");
        let protocol_a = root_a.with_extension("protocol.json");
        let protocol_b = root_b.with_extension("protocol.json");
        let definition_a = root_a.with_extension("definition.json");
        let definition_b = root_b.with_extension("definition.json");
        let first = tiny_protocol();
        let mut second = first.clone();
        second.comparisons[0]
            .decision_criterion
            .push_str(" Require robustness.");
        let definition = tiny_definition();
        write_json(&protocol_a, &first);
        write_json(&protocol_b, &second);
        write_json(&definition_a, &definition);
        write_json(&definition_b, &definition);

        let a = prepare(&root_a, &protocol_a, &definition_a).unwrap();
        let b = prepare(&root_b, &protocol_b, &definition_b).unwrap();
        assert_ne!(a.protocol_identity, b.protocol_identity);
        assert_ne!(a.study_execution_id, b.study_execution_id);
        assert_eq!(a.definition_identity, b.definition_identity);
        assert_eq!(a.definition, b.definition);

        for path in [&root_a, &root_b] {
            fs::remove_dir_all(path).unwrap();
        }
        for path in [&protocol_a, &protocol_b, &definition_a, &definition_b] {
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn finalize_binds_exact_research_identity_artifacts_and_is_idempotent() {
        let root = temp_root("finalize");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        write_json(&protocol_path, &tiny_protocol());
        write_json(&definition_path, &tiny_definition());
        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();
        fake_research_root(&root, &plan, "completed");

        let expected_research_id =
            research_execution_identity(&plan.definition_identity, &plan.source).unwrap();
        let first = finalize(&root).unwrap();
        let second = finalize(&root).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.protocol_identity, plan.protocol_identity);
        assert_eq!(first.definition_identity, plan.definition_identity);
        assert_eq!(first.research_id, expected_research_id);
        assert_eq!(first.run_counts.completed, 2);
        assert_eq!(first.run_counts.failed, 0);
        assert_eq!(first.result_artifacts.len(), 2);
        assert!(first.analysis_requirements.is_empty());
        assert!(
            first
                .result_artifacts
                .iter()
                .all(|artifact| artifact.digest64 != 0)
        );
        assert!(first.result_identity.starts_with("study-result-v1-"));
        assert!(first.confirmatory_pre_result_claim_eligible);

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn finalize_records_observable_support_analysis_requirement() {
        let root = temp_root("support-requirement");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        let mut protocol = tiny_protocol();
        let support_identity = "observable-support-plan-v1-sha256-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        protocol.observables[0].interpretation = format!(
            "Support-bound comparison; {OBSERVABLE_SUPPORT_BINDING_PREFIX}{support_identity}"
        );
        write_json(&protocol_path, &protocol);
        write_json(&definition_path, &tiny_definition());
        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();
        fake_research_root(&root, &plan, "completed");

        let binding = finalize(&root).unwrap();
        assert_eq!(
            binding.analysis_requirements,
            vec![StudyAnalysisRequirement {
                kind: OBSERVABLE_SUPPORT_REQUIREMENT_KIND.to_owned(),
                identity: support_identity.to_owned(),
            }]
        );
        let serialized: Value = read_json_regular(&root.join(RESULT_BINDING), "binding").unwrap();
        assert_eq!(
            serialized["analysisRequirements"][0]["identity"],
            Value::String(support_identity.to_owned())
        );

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn finalize_rejects_modified_protocol_unfinished_research_and_result_tampering() {
        let root = temp_root("reject");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        write_json(&protocol_path, &tiny_protocol());
        write_json(&definition_path, &tiny_definition());
        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();

        let mut modified = plan.protocol.clone();
        modified.analysis_method.push_str(" changed");
        write_json(&root.join(STUDY_PROTOCOL), &modified);
        assert!(load_frozen_plan(&root).is_err());
        write_json(&root.join(STUDY_PROTOCOL), &plan.protocol);

        fake_research_root(&root, &plan, "running");
        assert!(finalize(&root).is_err());

        fake_research_root(&root, &plan, "completed");
        let binding = finalize(&root).unwrap();
        let runs_path = root.join(RESEARCH_DIR).join("analysis/runs.json");
        let mut runs: Value = read_json_regular(&runs_path, "runs").unwrap();
        runs["tampered"] = Value::Bool(true);
        write_json(&runs_path, &runs);
        assert!(finalize(&root).is_err());
        let preserved: StudyResultBinding =
            read_json_regular(&root.join(RESULT_BINDING), "binding").unwrap();
        assert_eq!(preserved, binding);

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn finalize_rejects_self_consistent_rewrite_of_research_manifest_and_plan() {
        let root = temp_root("manifest-rewrite");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        write_json(&protocol_path, &tiny_protocol());
        write_json(&definition_path, &tiny_definition());
        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();
        fake_research_root(&root, &plan, "completed");

        let manifest_path = root.join(RESEARCH_DIR).join("research-manifest.json");
        let research_plan_path = root.join(RESEARCH_DIR).join("research-plan.json");
        let mut manifest: Value = read_json_regular(&manifest_path, "manifest").unwrap();
        manifest["points"][0]["point"]["index"] = Value::from(999_u64);
        write_json(&manifest_path, &manifest);
        write_json(&research_plan_path, &manifest);
        assert!(finalize(&root).is_err());
        assert!(!root.join(RESULT_BINDING).exists());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }

    #[test]
    fn finalize_rejects_state_digest_that_disagrees_with_validated_child_bundle() {
        let root = temp_root("state-digest");
        let protocol_path = root.with_extension("protocol.json");
        let definition_path = root.with_extension("definition.json");
        write_json(&protocol_path, &tiny_protocol());
        write_json(&definition_path, &tiny_definition());
        let plan = prepare(&root, &protocol_path, &definition_path).unwrap();
        fake_research_root(&root, &plan, "completed");

        let research_root = root.join(RESEARCH_DIR);
        let state_path = research_root.join("research-state.json");
        let mut state: ResearchStateView = read_json_regular(&state_path, "state").unwrap();
        let first_run_id = state.runs.keys().next().unwrap().clone();
        let row = state.runs.get_mut(&first_run_id).unwrap();
        row.state_digest64 = Some(row.state_digest64.unwrap().wrapping_add(1));
        write_json(&state_path, &state);
        write_json(
            &research_root.join("analysis/runs.json"),
            &expected_analysis_runs(&expected_research_manifest(&plan).unwrap(), &state).unwrap(),
        );
        assert!(finalize(&root).is_err());
        assert!(!root.join(RESULT_BINDING).exists());

        fs::remove_dir_all(&root).unwrap();
        fs::remove_file(protocol_path).unwrap();
        fs::remove_file(definition_path).unwrap();
    }
}
