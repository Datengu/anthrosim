use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anthrosim_core::{RecordedRun, RunManifest, SimulationCheckpoint};

fn temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-{label}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn multi_seed_long_run_ensemble_produces_only_invariant_valid_completed_bundles() {
    let root = temp_path("m7-4-ensemble-soak");
    let status = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "ensemble",
            "--years",
            "150",
            "--world-width",
            "8",
            "--world-height",
            "8",
            "--population",
            "100",
            "--max-person-records",
            "20000",
            "--resource-productivity-scale-permille",
            "700",
            "--annual-food-need",
            "100",
            "--seeds",
            "7440,7441,7442",
            "--run-dir",
        ])
        .arg(&root)
        .status()
        .expect("run ensemble soak");
    assert!(status.success());

    for seed in [7_440_u64, 7_441, 7_442] {
        let run_dir = root.join(format!("runs/seed-{seed:020}"));
        let manifest: RunManifest = serde_json::from_str(
            &fs::read_to_string(run_dir.join("manifest.json")).expect("manifest file"),
        )
        .expect("manifest json");
        let checkpoint: SimulationCheckpoint = serde_json::from_str(
            &fs::read_to_string(run_dir.join("checkpoint.json")).expect("checkpoint file"),
        )
        .expect("checkpoint json");
        let run = RecordedRun {
            manifest,
            checkpoint,
        };
        run.validate_invariants().expect("valid completed run");

        let status_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(format!("status/seed-{seed:020}.json")))
                .expect("status file"),
        )
        .expect("status json");
        assert_eq!(
            status_json.get("state").and_then(serde_json::Value::as_str),
            Some("completed")
        );
    }

    fs::remove_dir_all(root).expect("cleanup");
}
