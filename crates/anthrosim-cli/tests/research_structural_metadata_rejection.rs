use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResearchDimension,
    ResearchDimensionKind, ResearchExperimentDefinition, ResearchRunConfig, ResourceConfig,
    WorldConfig,
};
use serde_json::Value;

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-research-metadata-structure-{}-{id}",
        std::process::id()
    ))
}

fn metadata_only_definition() -> ResearchExperimentDefinition {
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
            id: "demography_structure".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: "/experiment/demography/scheduleId".to_owned(),
            values: vec![Value::from("alternative_a"), Value::from("alternative_b")],
        }],
    }
}

#[test]
fn metadata_only_structure_fails_before_manifest_or_analysis_publication() {
    let root = temp_root();
    fs::create_dir_all(&root).expect("create fixture root");
    let definition_path = root.join("definition.json");
    let run_dir = root.join("research");
    let mut bytes =
        serde_json::to_vec_pretty(&metadata_only_definition()).expect("serialize fixture");
    bytes.push(b'\n');
    fs::write(&definition_path, bytes).expect("write definition");

    let output = Command::new(env!("CARGO_BIN_EXE_anthrosim-research"))
        .arg("--definition")
        .arg(&definition_path)
        .arg("--run-dir")
        .arg(&run_dir)
        .output()
        .expect("launch anthrosim-research");

    assert!(
        !output.status.success(),
        "metadata-only structure was accepted"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not provide distinct executable alternatives"),
        "unexpected error: {stderr}"
    );
    assert!(
        !run_dir.exists(),
        "invalid structural sensitivity published a research root"
    );

    fs::remove_dir_all(root).expect("cleanup");
}
