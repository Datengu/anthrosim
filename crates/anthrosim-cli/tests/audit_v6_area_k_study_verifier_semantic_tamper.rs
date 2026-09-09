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
        "anthrosim-audit-v6-area-k-study-verifier-semantic-tamper-{}-{id}",
        std::process::id()
    ))
}

fn tiny_definition() -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(741_001, 1)
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
        seeds: vec![741_001],
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
        study_id: "audit-v6-area-k-study-verifier-semantic-tamper".to_owned(),
        status: StudyScientificStatus::Exploratory,
        research_question:
            "Does root-aware verification preserve the treatments that were actually executed?"
                .to_owned(),
        applicability_domain: "Synthetic orchestration/provenance adversary.".to_owned(),
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
            interpretation:
                "The finalized and verified canonical analysis rows must preserve executed treatments."
                    .to_owned(),
        }],
        comparisons: vec![StudyComparison {
            id: "period-treatment".to_owned(),
            hypothesis_ids: vec!["four".to_owned(), "twelve".to_owned()],
            observable_ids: vec!["treatment_rows".to_owned()],
            prediction: "The finalized study preserves period values 4 and 12.".to_owned(),
            decision_criterion:
                "Producer and root-aware verifier agree that canonical rows match immutable execution."
                    .to_owned(),
        }],
        evidence_roles: vec![],
        uncertainty: StudyUncertaintyPlan {
            parameter_uncertainty: vec!["Resource periods/year is explicitly varied.".to_owned()],
            structural_uncertainty: vec![],
        },
        ensemble_policy: StudyEnsemblePolicy {
            seed_policy: "Use the exact frozen research seed.".to_owned(),
            pairing_policy: "Use the same seed across treatment points.".to_owned(),
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
            criterion: "Canonical treatment rows retain the actually executed values 4 and 12."
                .to_owned(),
            failure_handling: "Reject a finalized/verified root whose rows contradict execution."
                .to_owned(),
        }],
        analysis_method:
            "Compare immutable research treatment coordinates with canonical finalized rows."
                .to_owned(),
        multiplicity_policy: "One synthetic integrity comparison.".to_owned(),
        held_out_corroboration: vec![],
        permitted_interpretations: vec!["Orchestration/provenance integrity only.".to_owned()],
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

fn helper_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("scripts/audit-v6-area-k-study-verifier-semantic-tamper.py")
}

#[test]
fn root_aware_verifier_rejects_self_consistent_binding_over_semantically_forged_analysis_rows() {
    if std::env::var_os("ANTHROSIM_AUDIT_V6_AREA_K_STUDY_VERIFIER_SEMANTIC_TAMPER").is_none() {
        return;
    }

    let root = temp_root();
    fs::create_dir_all(&root).expect("create fixture root");
    let protocol_path = root.join("protocol.json");
    let definition_path = root.join("definition.json");
    let study_dir = root.join("study");
    write_json(&protocol_path, &exploratory_protocol());
    write_json(&definition_path, &tiny_definition());

    let study_binary = env!("CARGO_BIN_EXE_anthrosim-study");
    let research_binary = env!("CARGO_BIN_EXE_anthrosim-research");
    let helper = helper_path();

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
        "initial anthrosim-study finalize",
    );
    require_success(&finalized, "initial anthrosim-study finalize");

    let positive_verify = run(
        Command::new("python3")
            .arg(&helper)
            .arg("verify")
            .arg(&study_dir),
        "positive root-aware verifier control",
    );
    require_success(&positive_verify, "positive root-aware verifier control");

    let research_plan = read_value(&study_dir.join("research/research-plan.json"));
    let immutable_values = research_plan["points"]
        .as_array()
        .expect("planned points")
        .iter()
        .map(|point| {
            point["point"]["coordinates"][0]["value"]
                .as_u64()
                .expect("planned value")
        })
        .collect::<Vec<_>>();
    assert_eq!(immutable_values, vec![4, 12]);

    let tampered = run(
        Command::new("python3")
            .arg(&helper)
            .arg("tamper-and-rebind")
            .arg(&study_dir),
        "coordinated semantic tamper and rebind",
    );
    require_success(&tampered, "coordinated semantic tamper and rebind");

    let producer_recheck = run(
        Command::new(study_binary)
            .arg("finalize")
            .arg("--study-dir")
            .arg(&study_dir),
        "producer semantic recheck",
    );
    assert!(
        !producer_recheck.status.success(),
        "producer unexpectedly accepted forged canonical analysis rows\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&producer_recheck.stdout),
        String::from_utf8_lossy(&producer_recheck.stderr)
    );

    let verifier = run(
        Command::new("python3")
            .arg(&helper)
            .arg("verify")
            .arg(&study_dir),
        "root-aware study-result verifier",
    );

    println!("immutable_plan_treatments={immutable_values:?}");
    print!("{}", String::from_utf8_lossy(&tampered.stdout));
    println!("producer_finalize_rejects=true");
    println!(
        "producer_error={}",
        String::from_utf8_lossy(&producer_recheck.stderr).trim()
    );
    print!("{}", String::from_utf8_lossy(&verifier.stdout));
    if !verifier.stderr.is_empty() {
        println!(
            "verifier_stderr={}",
            String::from_utf8_lossy(&verifier.stderr).trim()
        );
    }

    assert!(
        !verifier.status.success(),
        "predeclared Area-K oracle failed: the root-aware finalized-study verifier accepted a freshly self-consistent study-result binding over canonical treatment rows that the authoritative producer independently rejects as contradicting the immutable research plan/state"
    );

    fs::remove_dir_all(root).expect("cleanup fixture root");
}
