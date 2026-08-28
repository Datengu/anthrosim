use std::{
    fs, io,
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

const ANALYSIS_PROGRAM: &str = r#"import argparse, json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--input", required=True)
parser.add_argument("--output", required=True)
args = parser.parse_args()

source = json.loads(Path(args.input).read_text(encoding="utf-8"))
rows = source["runs"]
result = {
    "schemaVersion": 1,
    "researchId": source["researchId"],
    "runCount": len(rows),
    "completedRunCount": sum(1 for row in rows if row["state"] == "completed"),
}
Path(args.output).write_text(
    json.dumps(result, sort_keys=True, separators=(",", ":")) + "\n",
    encoding="utf-8",
)
"#;

fn temp_root(name: &str) -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-analysis-provenance-e2e-{name}-{}-{id}",
        std::process::id()
    ))
}

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
        seeds: vec![101],
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

fn confirmatory_protocol() -> StudyProtocol {
    StudyProtocol {
        schema_version: StudyProtocol::CURRENT_SCHEMA_VERSION,
        protocol_revision: 1,
        study_id: "analysis-provenance-e2e".to_owned(),
        status: StudyScientificStatus::Confirmatory,
        research_question:
            "Does the declared M3 period-count treatment produce two preserved research arms?"
                .to_owned(),
        applicability_domain: "Synthetic executable provenance verification only.".to_owned(),
        hypotheses: vec![
            StudyHypothesis {
                id: "four_periods".to_owned(),
                kind: StudyHypothesisKind::NullModel,
                statement: "The four-period arm is the reference model.".to_owned(),
            },
            StudyHypothesis {
                id: "twelve_periods".to_owned(),
                kind: StudyHypothesisKind::Alternative,
                statement: "The twelve-period arm is the declared alternative.".to_owned(),
            },
        ],
        analysis_windows: vec![StudyAnalysisWindow {
            id: "primary".to_owned(),
            analysis_start_day: 0,
            analysis_end_day_inclusive: None,
            selection_rule: StudyAnalysisWindowSelectionRule::InitialStateInScope,
            rationale:
                "The synthetic provenance test intentionally includes the complete one-year run."
                    .to_owned(),
        }],
        observables: vec![StudyObservable {
            id: "completed_runs".to_owned(),
            role: StudyObservableRole::Primary,
            source: "research.analysis.runs completed-state count".to_owned(),
            analysis_window_id: "primary".to_owned(),
            interpretation:
                "Both declared research arms must be present as completed immutable runs."
                    .to_owned(),
        }],
        comparisons: vec![StudyComparison {
            id: "declared_arms".to_owned(),
            hypothesis_ids: vec!["four_periods".to_owned(), "twelve_periods".to_owned()],
            observable_ids: vec!["completed_runs".to_owned()],
            prediction:
                "The immutable research design contains one completed run per declared arm."
                    .to_owned(),
            decision_criterion:
                "The canonical downstream result must report exactly two completed runs.".to_owned(),
        }],
        evidence_roles: vec![],
        uncertainty: StudyUncertaintyPlan {
            parameter_uncertainty: vec!["M3 periods per year varies between 4 and 12.".to_owned()],
            structural_uncertainty: vec![],
        },
        ensemble_policy: StudyEnsemblePolicy {
            seed_policy: "Use exactly seed 101 for both paired arms.".to_owned(),
            pairing_policy: "Pair the same seed across both declared period-count values."
                .to_owned(),
            replication_policy: "No adaptive replicate addition.".to_owned(),
        },
        run_handling: StudyRunHandling {
            stopping_rules: vec!["Run each arm for the configured one model year.".to_owned()],
            exclusion_rules: vec!["No post-hoc exclusions are permitted.".to_owned()],
            censoring_rules: vec![
                "Any non-completed run invalidates this synthetic check.".to_owned(),
            ],
        },
        sensitivity_plan: vec![
            "No additional sensitivity analysis is required for this infrastructure test."
                .to_owned(),
        ],
        equifinality_plan: vec![
            "No substantive equifinality claim is made by this infrastructure test.".to_owned(),
        ],
        manipulation_checks: vec![StudyManipulationCheck {
            id: "period_count_realized".to_owned(),
            mechanism: "M3 resource-period schedule".to_owned(),
            criterion: "The two research rows preserve periodsPerYear values 4 and 12.".to_owned(),
            failure_handling:
                "Fail the infrastructure regression rather than interpret the result.".to_owned(),
        }],
        analysis_method: "Count the immutable completed run rows produced by anthrosim-research."
            .to_owned(),
        multiplicity_policy: "One synthetic primary result only.".to_owned(),
        held_out_corroboration: vec![],
        permitted_interpretations: vec![
            "Executable provenance plumbing is connected end to end.".to_owned(),
        ],
        prohibited_interpretations: vec!["Any empirical or archaeological inference.".to_owned()],
        amendment: None,
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize fixture");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write fixture");
}

fn run_checked(command: &mut Command, role: &str) -> Output {
    let output = command.output().unwrap_or_else(|error| {
        panic!("failed to launch {role}: {error}");
    });
    assert!(
        output.status.success(),
        "{role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn find_python() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        match Command::new(&python).arg("--version").output() {
            Ok(output) if output.status.success() => return Some(python),
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        }
    }
    None
}

#[test]
fn immutable_sweep_to_downstream_result_to_integrity_archive_is_replayable() {
    let Some(python) = find_python() else {
        eprintln!("skipping real analysis-provenance end-to-end test because Python was not found");
        return;
    };

    let root = temp_root("chain");
    fs::create_dir_all(&root).expect("create fixture root");
    let protocol_path = root.join("protocol.json");
    let definition_path = root.join("research-definition-source.json");
    let study_dir = root.join("study");
    write_json(&protocol_path, &confirmatory_protocol());
    write_json(&definition_path, &tiny_definition());

    let study_binary = env!("CARGO_BIN_EXE_anthrosim-study");
    let research_binary = env!("CARGO_BIN_EXE_anthrosim-research");
    run_checked(
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
    run_checked(
        Command::new(research_binary)
            .arg("--definition")
            .arg(study_dir.join("research-definition.json"))
            .arg("--run-dir")
            .arg(study_dir.join("research")),
        "anthrosim-research immutable sweep",
    );
    run_checked(
        Command::new(study_binary)
            .arg("finalize")
            .arg("--study-dir")
            .arg(&study_dir),
        "anthrosim-study finalize",
    );

    let research_plan: Value = serde_json::from_slice(
        &fs::read(study_dir.join("research/research-plan.json")).expect("read research plan"),
    )
    .expect("parse research plan");
    let research_manifest: Value = serde_json::from_slice(
        &fs::read(study_dir.join("research/research-manifest.json"))
            .expect("read research manifest"),
    )
    .expect("parse research manifest");
    assert_eq!(research_plan, research_manifest);

    let runs_path = study_dir.join("research/analysis/runs.json");
    let runs: Value = serde_json::from_slice(&fs::read(&runs_path).expect("read run analysis"))
        .expect("parse run analysis");
    let rows = runs["runs"].as_array().expect("run rows");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row["state"] == "completed"));
    let period_values: Vec<u64> = rows
        .iter()
        .map(|row| {
            row["coordinates"][0]["value"]
                .as_u64()
                .expect("numeric period value")
        })
        .collect();
    assert_eq!(period_values, vec![4, 12]);

    let analysis_dir = study_dir.join("analysis");
    fs::create_dir_all(&analysis_dir).expect("create analysis dir");
    fs::write(analysis_dir.join("analyze.py"), ANALYSIS_PROGRAM).expect("write analysis script");
    fs::write(
        analysis_dir.join("environment.lock"),
        format!("python={}\nstdlib-only=true\n", python.display()),
    )
    .expect("write environment lock");
    let analysis_definition_path = analysis_dir.join("definition.json");
    write_json(
        &analysis_definition_path,
        &json!({
            "schemaVersion": 1,
            "definitionType": "anthrosim-analysis-definition",
            "analysisId": "real-sweep-run-count-v1",
            "analysisStatus": "confirmatory",
            "executionMode": "scripted",
            "workingDirectory": ".",
            "command": [
                python.to_string_lossy(),
                "analysis/analyze.py",
                "--input",
                "research/analysis/runs.json",
                "--output",
                "analysis/result.json"
            ],
            "arguments": {
                "estimand": "completed immutable research-run count",
                "expectedArms": [4, 12]
            },
            "analysisRngSeeds": [],
            "runtimeDescription": "Python standard library; exact interpreter command is preserved in command and environment.lock.",
            "reproductionCriterion": "exact_output_bytes",
            "inputs": [
                {"path": "research/analysis/runs.json", "role": "immutable-derived-run-table"}
            ],
            "implementation": [
                {"path": "analysis/analyze.py", "role": "canonical-analysis-script"}
            ],
            "environment": [
                {"path": "analysis/environment.lock", "role": "analysis-environment-lock"}
            ],
            "outputs": [
                {"path": "analysis/result.json", "role": "canonical-machine-readable-result"}
            ],
            "manualSteps": [],
            "observationModelIdentity": null
        }),
    );

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let provenance_script = repo_root.join("scripts/research-analysis-provenance.py");
    let integrity_script = repo_root.join("scripts/research-integrity.py");

    let run_output = run_checked(
        Command::new(&python)
            .arg(&provenance_script)
            .arg("run")
            .arg(&study_dir)
            .arg(&analysis_definition_path),
        "downstream analysis provenance run",
    );
    let provenance_identity = String::from_utf8_lossy(&run_output.stdout)
        .trim()
        .to_owned();
    assert!(provenance_identity.starts_with("analysis-provenance-v1-sha256-"));

    let result: Value = serde_json::from_slice(
        &fs::read(analysis_dir.join("result.json")).expect("read analysis result"),
    )
    .expect("parse analysis result");
    assert_eq!(result["runCount"], 2);
    assert_eq!(result["completedRunCount"], 2);
    assert_eq!(result["researchId"], runs["researchId"]);

    let provenance: Value = serde_json::from_slice(
        &fs::read(analysis_dir.join("analysis-provenance.json")).expect("read analysis provenance"),
    )
    .expect("parse analysis provenance");
    assert_eq!(provenance["provenanceIdentity"], provenance_identity);
    assert_eq!(provenance["study"]["researchId"], runs["researchId"]);
    assert_eq!(provenance["study"]["boundBeforeExecution"], true);
    assert_eq!(provenance["executionStatus"], "executed_by_wrapper");

    run_checked(
        Command::new(&python)
            .arg(&provenance_script)
            .arg("verify")
            .arg(&study_dir),
        "analysis provenance verify",
    );
    run_checked(
        Command::new(&python)
            .arg(&provenance_script)
            .arg("replay")
            .arg(&study_dir),
        "analysis provenance replay",
    );
    run_checked(
        Command::new(&python)
            .arg(&integrity_script)
            .arg("create")
            .arg(&study_dir),
        "research integrity create",
    );
    run_checked(
        Command::new(&python)
            .arg(&integrity_script)
            .arg("verify")
            .arg(&study_dir),
        "research integrity verify",
    );

    fs::remove_dir_all(&root).expect("remove fixture root");
}
