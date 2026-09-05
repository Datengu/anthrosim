use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResearchDimension,
    ResearchDimensionKind, ResearchExperimentDefinition, ResearchRunConfig, ResourceConfig,
    StudyAnalysisWindow, StudyAnalysisWindowSelectionRule, StudyComparison, StudyEnsemblePolicy,
    StudyHypothesis, StudyHypothesisKind, StudyManipulationCheck, StudyObservable,
    StudyObservableRole, StudyProtocol, StudyRunHandling, StudyScientificStatus,
    StudyUncertaintyPlan, WorldConfig,
};
use serde_json::{Value, json};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-audit-v4-area-k-finalize-analysis-tamper-{}-{id}",
        std::process::id()
    ))
}

fn tiny_definition() -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(541_001, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(
            PopulationConfig::new(8)
                .with_target_household_size(4)
                .with_max_person_records(64),
        )
        .with_demography(DemographyConfig::synthetic_validation_v1())
        .with_resources(ResourceConfig::synthetic_validation_v1())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    experiment
        .resources
        .max_scarcity_mortality_probability_per_million = 0;
    ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![541_001],
        base: ResearchRunConfig {
            experiment,
            spatial: None,
        },
        dimensions: vec![ResearchDimension {
            id: "m3_periods_per_year".to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: "/experiment/resources/periodsPerYear".to_owned(),
            values: vec![Value::from(4), Value::from(12)],
        }],
    }
}

fn exploratory_protocol() -> StudyProtocol {
    StudyProtocol {
        schema_version: StudyProtocol::CURRENT_SCHEMA_VERSION,
        protocol_revision: 1,
        study_id: "audit-v4-area-k-finalize-analysis-tamper".to_owned(),
        status: StudyScientificStatus::Exploratory,
        research_question: "Are the two declared resource-period treatments preserved?".to_owned(),
        applicability_domain: "Synthetic orchestration-integrity adversary.".to_owned(),
        hypotheses: vec![
            StudyHypothesis {
                id: "four".to_owned(),
                kind: StudyHypothesisKind::NullModel,
                statement: "Four periods is the reference arm.".to_owned(),
            },
            StudyHypothesis {
                id: "twelve".to_owned(),
                kind: StudyHypothesisKind::Alternative,
                statement: "Twelve periods is the alternative arm.".to_owned(),
            },
        ],
        analysis_windows: vec![StudyAnalysisWindow {
            id: "full".to_owned(),
            analysis_start_day: 0,
            analysis_end_day_inclusive: None,
            selection_rule: StudyAnalysisWindowSelectionRule::InitialStateInScope,
            rationale: "Use the complete one-year synthetic execution.".to_owned(),
        }],
        observables: vec![StudyObservable {
            id: "treatment_rows".to_owned(),
            role: StudyObservableRole::Primary,
            source: "research.analysis.runs coordinates/resultingConfiguration".to_owned(),
            analysis_window_id: "full".to_owned(),
            interpretation: "The standard analysis rows must preserve the executed treatments."
                .to_owned(),
        }],
        comparisons: vec![StudyComparison {
            id: "period-treatment".to_owned(),
            hypothesis_ids: vec!["four".to_owned(), "twelve".to_owned()],
            observable_ids: vec!["treatment_rows".to_owned()],
            prediction: "The finalized study preserves period values 4 and 12.".to_owned(),
            decision_criterion: "The canonical research rows match the immutable research plan."
                .to_owned(),
        }],
        evidence_roles: vec![],
        uncertainty: StudyUncertaintyPlan {
            parameter_uncertainty: vec!["Resource periods/year is explicitly varied.".to_owned()],
            structural_uncertainty: vec![],
        },
        ensemble_policy: StudyEnsemblePolicy {
            seed_policy: "Use the exact frozen research seed.".to_owned(),
            pairing_policy: "Use the same seed across the two treatment points.".to_owned(),
            replication_policy: "No adaptive replication in this synthetic adversary.".to_owned(),
        },
        run_handling: StudyRunHandling {
            stopping_rules: vec!["Use the configured one-year duration.".to_owned()],
            exclusion_rules: vec!["No post-hoc exclusions.".to_owned()],
            censoring_rules: vec!["Operational failures remain explicit.".to_owned()],
        },
        sensitivity_plan: vec!["No additional sensitivity analysis is required here.".to_owned()],
        equifinality_plan: vec!["No substantive equifinality claim is made here.".to_owned()],
        manipulation_checks: vec![StudyManipulationCheck {
            id: "periods-realized".to_owned(),
            mechanism: "M3 resource-period schedule".to_owned(),
            criterion: "Executed/canonical treatment rows retain values 4 and 12.".to_owned(),
            failure_handling: "Do not finalize a study with altered treatment rows.".to_owned(),
        }],
        analysis_method:
            "Inspect the immutable research-plan treatment coordinates and canonical analysis rows."
                .to_owned(),
        multiplicity_policy: "One synthetic integrity comparison.".to_owned(),
        held_out_corroboration: vec![],
        permitted_interpretations: vec!["Orchestration integrity only.".to_owned()],
        prohibited_interpretations: vec!["Empirical validation.".to_owned()],
        amendment: None,
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write fixture");
}

fn read_value(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn run(command: &mut Command, role: &str) -> Output {
    command
        .output()
        .unwrap_or_else(|error| panic!("launch {role}: {error}"))
}

fn require_success(output: &Output, role: &str) {
    assert!(
        output.status.success(),
        "{role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn study_finalize_rejects_canonical_analysis_rows_that_disagree_with_immutable_research_plan() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("create fixture root");
    let protocol_path = root.join("protocol.json");
    let definition_path = root.join("definition.json");
    let study_dir = root.join("study");
    write_json(&protocol_path, &exploratory_protocol());
    write_json(&definition_path, &tiny_definition());

    let study_binary = env!("CARGO_BIN_EXE_anthrosim-study");
    let research_binary = env!("CARGO_BIN_EXE_anthrosim-research");

    let prepared = run(
        Command::new(study_binary)
            .arg("prepare")
            .arg("--protocol")
            .arg(&protocol_path)
            .arg("--definition")
            .arg(&definition_path)
            .arg("--study-dir")
            .arg(&study_dir),
        "anthrosim-study prepare",
    );
    require_success(&prepared, "anthrosim-study prepare");

    let executed = run(
        Command::new(research_binary)
            .arg("--definition")
            .arg(study_dir.join("research-definition.json"))
            .arg("--run-dir")
            .arg(study_dir.join("research")),
        "anthrosim-research",
    );
    require_success(&executed, "anthrosim-research");

    let research_plan = read_value(&study_dir.join("research/research-plan.json"));
    let plan_values = research_plan["points"]
        .as_array()
        .expect("planned points")
        .iter()
        .map(|point| {
            point["point"]["coordinates"][0]["value"]
                .as_u64()
                .expect("planned value")
        })
        .collect::<Vec<_>>();
    assert_eq!(plan_values, vec![4, 12]);

    let points_path = study_dir.join("research/analysis/points.json");
    let runs_path = study_dir.join("research/analysis/runs.json");
    let mut points = read_value(&points_path);
    let mut runs = read_value(&runs_path);
    let original_point_values = points["points"]
        .as_array()
        .expect("analysis points")
        .iter()
        .map(|point| {
            point["coordinates"][0]["value"]
                .as_u64()
                .expect("point value")
        })
        .collect::<Vec<_>>();
    let original_run_values = runs["runs"]
        .as_array()
        .expect("analysis runs")
        .iter()
        .map(|run| run["coordinates"][0]["value"].as_u64().expect("run value"))
        .collect::<Vec<_>>();
    assert_eq!(original_point_values, plan_values);
    assert_eq!(original_run_values, plan_values);

    points["points"][0]["coordinates"][0]["value"] = json!(999_u64);
    points["points"][0]["resultingConfiguration"]["experiment"]["resources"]["periodsPerYear"] =
        json!(999_u64);
    runs["runs"][0]["coordinates"][0]["value"] = json!(999_u64);
    runs["runs"][0]["resultingConfiguration"]["experiment"]["resources"]["periodsPerYear"] =
        json!(999_u64);
    write_json(&points_path, &points);
    write_json(&runs_path, &runs);

    let tampered_point_values = points["points"]
        .as_array()
        .expect("analysis points")
        .iter()
        .map(|point| {
            point["coordinates"][0]["value"]
                .as_u64()
                .expect("point value")
        })
        .collect::<Vec<_>>();
    let tampered_run_values = runs["runs"]
        .as_array()
        .expect("analysis runs")
        .iter()
        .map(|run| run["coordinates"][0]["value"].as_u64().expect("run value"))
        .collect::<Vec<_>>();
    assert_eq!(tampered_point_values, vec![999, 12]);
    assert_eq!(tampered_run_values, vec![999, 12]);

    let finalized = run(
        Command::new(study_binary)
            .arg("finalize")
            .arg("--study-dir")
            .arg(&study_dir),
        "anthrosim-study finalize",
    );

    println!("immutable_plan_values={plan_values:?}");
    println!("tampered_points_values={tampered_point_values:?}");
    println!("tampered_runs_values={tampered_run_values:?}");
    println!("finalize_status={}", finalized.status);

    if finalized.status.success() {
        let binding = read_value(&study_dir.join("study-result-binding.json"));
        println!("finalized_result_identity={}", binding["resultIdentity"]);
        println!("finalized_research_id={}", binding["researchId"]);
        println!("finalized_result_artifacts={}", binding["resultArtifacts"]);
        panic!(
            "anthrosim-study finalize accepted canonical analysis rows whose treatment coordinates/configuration disagree with the immutable research plan"
        );
    }

    fs::remove_dir_all(root).expect("cleanup fixture root");
}
