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
use serde_json::Value;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-audit-v6-area-k-cross-root-binding-transplant-{}-{id}",
        std::process::id()
    ))
}

fn tiny_definition() -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(742_001, 1)
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
        seeds: vec![742_001],
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

fn protocol(study_id: &str) -> StudyProtocol {
    StudyProtocol {
        schema_version: StudyProtocol::CURRENT_SCHEMA_VERSION,
        protocol_revision: 1,
        study_id: study_id.to_owned(),
        status: StudyScientificStatus::Exploratory,
        research_question: "Does study-root verification bind the result to its frozen protocol?"
            .to_owned(),
        applicability_domain: "Synthetic cross-root provenance control.".to_owned(),
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
            rationale: "Use the complete one-year synthetic run.".to_owned(),
        }],
        observables: vec![StudyObservable {
            id: "binding_identity".to_owned(),
            role: StudyObservableRole::Primary,
            source: "study-result-binding.json".to_owned(),
            analysis_window_id: "full".to_owned(),
            interpretation: "The finalized result remains bound to its own frozen study protocol."
                .to_owned(),
        }],
        comparisons: vec![StudyComparison {
            id: "root-binding".to_owned(),
            hypothesis_ids: vec!["four".to_owned(), "twelve".to_owned()],
            observable_ids: vec!["binding_identity".to_owned()],
            prediction: "A result binding from another protocol root is rejected.".to_owned(),
            decision_criterion: "Root-aware verification rejects cross-study binding transplant."
                .to_owned(),
        }],
        evidence_roles: vec![],
        uncertainty: StudyUncertaintyPlan {
            parameter_uncertainty: vec!["Resource periods/year is explicitly varied.".to_owned()],
            structural_uncertainty: vec![],
        },
        ensemble_policy: StudyEnsemblePolicy {
            seed_policy: "Use one exact frozen seed.".to_owned(),
            pairing_policy: "Use the same seed across treatments and both study roots.".to_owned(),
            replication_policy: "No adaptive replication.".to_owned(),
        },
        run_handling: StudyRunHandling {
            stopping_rules: vec!["Use configured one-year duration.".to_owned()],
            exclusion_rules: vec!["No post-hoc exclusions.".to_owned()],
            censoring_rules: vec!["Operational failures remain explicit.".to_owned()],
        },
        sensitivity_plan: vec!["No additional sensitivity analysis is required here.".to_owned()],
        equifinality_plan: vec!["No substantive equifinality claim is made here.".to_owned()],
        manipulation_checks: vec![StudyManipulationCheck {
            id: "root-identity".to_owned(),
            mechanism: "Study protocol/result binding".to_owned(),
            criterion: "Each finalized result verifies only in its own frozen study root."
                .to_owned(),
            failure_handling: "Reject transplanted result binding.".to_owned(),
        }],
        analysis_method: "Compare two byte-identical research executions under distinct protocols."
            .to_owned(),
        multiplicity_policy: "One provenance-integrity comparison.".to_owned(),
        held_out_corroboration: vec![],
        permitted_interpretations: vec!["Provenance/orchestration integrity only.".to_owned()],
        prohibited_interpretations: vec!["Empirical validation.".to_owned()],
        amendment: None,
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write fixture");
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

fn verifier_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("scripts/audit-v6-area-k-verify-study-root.py")
}

fn prepare_execute_finalize(
    root: &Path,
    study_id: &str,
    study_binary: &str,
    research_binary: &str,
) -> PathBuf {
    fs::create_dir_all(root).expect("create study root");
    let protocol_path = root.join("protocol.json");
    let definition_path = root.join("definition.json");
    let study_dir = root.join("study");
    write_json(&protocol_path, &protocol(study_id));
    write_json(&definition_path, &tiny_definition());

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

    let finalized = run(
        Command::new(study_binary)
            .arg("finalize")
            .arg("--study-dir")
            .arg(&study_dir),
        "anthrosim-study finalize",
    );
    require_success(&finalized, "anthrosim-study finalize");
    study_dir
}

#[test]
fn root_verifier_rejects_binding_transplant_between_distinct_protocols_with_identical_research() {
    let root = temp_root();
    let study_binary = env!("CARGO_BIN_EXE_anthrosim-study");
    let research_binary = env!("CARGO_BIN_EXE_anthrosim-research");
    let verifier = verifier_path();

    let study_a = prepare_execute_finalize(
        &root.join("a"),
        "audit-v6-area-k-cross-root-a",
        study_binary,
        research_binary,
    );
    let study_b = prepare_execute_finalize(
        &root.join("b"),
        "audit-v6-area-k-cross-root-b",
        study_binary,
        research_binary,
    );

    for relative in [
        "research/analysis/points.json",
        "research/analysis/runs.json",
    ] {
        assert_eq!(
            fs::read(study_a.join(relative)).expect("read A research artifact"),
            fs::read(study_b.join(relative)).expect("read B research artifact"),
            "research artifact {relative} should be byte-identical across protocol-only change"
        );
    }

    let verify_a = run(
        Command::new("python3").arg(&verifier).arg(&study_a),
        "verify untouched study A",
    );
    require_success(&verify_a, "verify untouched study A");
    let verify_b = run(
        Command::new("python3").arg(&verifier).arg(&study_b),
        "verify untouched study B",
    );
    require_success(&verify_b, "verify untouched study B");

    let binding_a = fs::read(study_a.join("study-result-binding.json")).expect("read binding A");
    let binding_b = fs::read(study_b.join("study-result-binding.json")).expect("read binding B");
    assert_ne!(
        binding_a, binding_b,
        "distinct frozen protocols must produce distinct study bindings"
    );
    fs::write(study_a.join("study-result-binding.json"), &binding_b)
        .expect("transplant binding B into A");

    let transplanted = run(
        Command::new("python3").arg(&verifier).arg(&study_a),
        "verify transplanted binding",
    );

    println!("research_artifacts_byte_identical=true");
    println!("untouched_a_verified=true");
    println!("untouched_b_verified=true");
    println!(
        "transplanted_binding_rejected={}",
        !transplanted.status.success()
    );
    print!("{}", String::from_utf8_lossy(&transplanted.stdout));
    if !transplanted.stderr.is_empty() {
        println!(
            "stderr={}",
            String::from_utf8_lossy(&transplanted.stderr).trim()
        );
    }

    assert!(
        !transplanted.status.success(),
        "a result binding from a distinct frozen study protocol must not authenticate another root even when the underlying research execution artifacts are byte-identical"
    );

    fs::remove_dir_all(root).expect("cleanup fixture root");
}
