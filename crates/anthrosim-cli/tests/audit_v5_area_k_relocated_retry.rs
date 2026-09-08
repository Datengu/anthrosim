use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig,
    ResearchExperimentDefinition, ResearchRunConfig, ResourceConfig, WorldConfig,
};
use serde_json::Value;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_root(name: &str) -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-audit-v5-area-k-{name}-{}-{id}",
        std::process::id()
    ))
}

fn tiny_definition() -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(919, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(
            PopulationConfig::new(12)
                .with_target_household_size(3)
                .with_max_person_records(64),
        )
        .with_demography(DemographyConfig::synthetic_validation_v1())
        .with_resources(ResourceConfig::synthetic_validation_v1())
        .with_migration(MigrationConfig::synthetic_validation_v1());
    experiment
        .resources
        .max_scarcity_mortality_probability_per_million = 0;

    ResearchExperimentDefinition {
        schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
        seeds: vec![919],
        base: ResearchRunConfig {
            experiment,
            spatial: None,
        },
        dimensions: vec![],
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

fn read_value(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read JSON artifact")).expect("parse JSON artifact")
}

#[test]
fn relocated_root_and_missing_mutable_state_preserve_scientific_identity_on_retry() {
    let root = temp_root("relocated-retry");
    fs::create_dir_all(&root).expect("create fixture root");

    let first_definition = root.join("definition-original.json");
    let second_definition_dir = root.join("different-definition-location");
    fs::create_dir_all(&second_definition_dir).expect("create alternate definition directory");
    let second_definition = second_definition_dir.join("definition-copy.json");
    let definition = tiny_definition();
    write_json(&first_definition, &definition);
    write_json(&second_definition, &definition);

    let original_research_root = root.join("research-original-location");
    let relocated_research_root = root.join("operationally-relocated").join("research");
    let research_binary = env!("CARGO_BIN_EXE_anthrosim-research");

    run_checked(
        Command::new(research_binary)
            .arg("--definition")
            .arg(&first_definition)
            .arg("--run-dir")
            .arg(&original_research_root),
        "initial anthrosim-research execution",
    );

    let plan_before = read_value(&original_research_root.join("research-plan.json"));
    let runs_before = read_value(&original_research_root.join("analysis/runs.json"));
    let research_id_before = plan_before["researchId"].clone();
    let definition_identity_before = plan_before["definitionIdentity"].clone();
    let run_id_before = runs_before["runs"][0]["runId"].clone();
    let state_digest_before = runs_before["runs"][0]["stateDigest64"].clone();
    let relative_dir = runs_before["runs"][0]["relativeDir"]
        .as_str()
        .expect("analysis row relativeDir");

    let child_before = original_research_root.join(relative_dir);
    let sentinel_name = "audit-v5-retention-sentinel.txt";
    fs::write(child_before.join(sentinel_name), "must survive retained-bundle retry\n")
        .expect("write retention sentinel");

    fs::create_dir_all(relocated_research_root.parent().expect("relocated parent"))
        .expect("create relocation parent");
    fs::rename(&original_research_root, &relocated_research_root)
        .expect("relocate complete research root");
    fs::remove_file(relocated_research_root.join("research-state.json"))
        .expect("remove mutable execution state to simulate recoverable interruption");

    run_checked(
        Command::new(research_binary)
            .arg("--definition")
            .arg(&second_definition)
            .arg("--run-dir")
            .arg(&relocated_research_root)
            .arg("--retry"),
        "relocated anthrosim-research retry",
    );

    let plan_after = read_value(&relocated_research_root.join("research-plan.json"));
    let manifest_after = read_value(&relocated_research_root.join("research-manifest.json"));
    let runs_after = read_value(&relocated_research_root.join("analysis/runs.json"));

    assert_eq!(plan_after, manifest_after, "immutable root copies diverged");
    assert_eq!(plan_after["researchId"], research_id_before);
    assert_eq!(plan_after["definitionIdentity"], definition_identity_before);
    assert_eq!(runs_after["runs"][0]["runId"], run_id_before);
    assert_eq!(runs_after["runs"][0]["stateDigest64"], state_digest_before);
    assert_eq!(runs_after["runs"][0]["state"], "completed");
    assert!(
        relocated_research_root
            .join(relative_dir)
            .join(sentinel_name)
            .is_file(),
        "validated completed child bundle was re-executed/replaced instead of retained"
    );
    assert!(
        relocated_research_root.join("research-state.json").is_file(),
        "missing mutable state was not reconstructed"
    );

    eprintln!("research_id={}", plan_after["researchId"]);
    eprintln!("run_id={}", runs_after["runs"][0]["runId"]);
    eprintln!("state_digest64={}", runs_after["runs"][0]["stateDigest64"]);
    eprintln!("retained_child_bundle=true");
    eprintln!("reconstructed_mutable_state=true");

    fs::remove_dir_all(&root).expect("remove fixture root");
}
