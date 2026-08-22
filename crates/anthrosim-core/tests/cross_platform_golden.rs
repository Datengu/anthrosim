use std::{env, fs, path::PathBuf};

use anthrosim_core::{
    ExperimentConfig, PopulationConfig, RunManifest, Simulation, SimulationCheckpoint, WorldConfig,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CrossPlatformGoldenRun {
    manifest: RunManifest,
    checkpoint: SimulationCheckpoint,
}

fn golden_fixture() -> CrossPlatformGoldenRun {
    let config = ExperimentConfig::new(0xA17_2026, 12)
        .with_world(WorldConfig::new(24, 16))
        .with_population(PopulationConfig::new(600).with_max_person_records(20_000));

    let mut recorded = Simulation::new(config).unwrap().run_recorded().unwrap();

    // Exact source revision is provenance, not scientific run state. The CI
    // matrix executes the same commit, but clearing it keeps this fixture
    // explicitly focused on the platform-independent determinism boundary.
    recorded.manifest.git_commit = None;
    recorded.checkpoint.git_commit = None;

    assert!(!recorded.events().is_empty());
    assert!(recorded.metrics().len() > 1);
    assert!(recorded.manifest.statistics.migration_decision_boundaries > 0);

    CrossPlatformGoldenRun {
        manifest: recorded.manifest,
        checkpoint: recorded.checkpoint,
    }
}

#[test]
fn deterministic_fixture_is_byte_stable_and_exportable() {
    let first = serde_json::to_vec_pretty(&golden_fixture()).unwrap();
    let second = serde_json::to_vec_pretty(&golden_fixture()).unwrap();
    assert_eq!(first, second);

    let Some(path) = env::var_os("ANTHROSIM_CROSS_PLATFORM_GOLDEN") else {
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, first).unwrap();
}
