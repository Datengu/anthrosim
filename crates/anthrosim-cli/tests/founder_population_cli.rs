use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[test]
fn run_accepts_declared_founder_population_file() {
    let example = founder_example();
    let output = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "run",
            "--founder-population",
            example.to_str().expect("example path must be UTF-8"),
            "--world-width",
            "1",
            "--world-height",
            "1",
            "--years",
            "0",
            "--disable-migration",
        ])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI stdout should be a JSON manifest");
    assert_eq!(
        manifest["experiment"]["population"]["initialization"],
        "declared_founder_state_v1"
    );
    assert_eq!(manifest["population"]["initialPopulation"], 2);
    assert_eq!(
        manifest["experiment"]["founderPopulation"]["initializationId"],
        "declared-founder-cli-example-v1"
    );
}

#[test]
fn declared_founder_completed_run_directory_passes_bundle_validation() {
    let example = founder_example();
    let run_dir = unique_temp_dir();
    let output = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "run",
            "--founder-population",
            example.to_str().expect("example path must be UTF-8"),
            "--world-width",
            "1",
            "--world-height",
            "1",
            "--years",
            "0",
            "--disable-migration",
            "--run-dir",
            run_dir.to_str().expect("temp path must be UTF-8"),
        ])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(
        output.status.success(),
        "declared-founder run-directory CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(run_dir.join("checkpoint.json").is_file());
    assert!(run_dir.join("initial-population.json").is_file());
    fs::remove_dir_all(&run_dir).expect("temp run directory should clean up");
}

fn founder_example() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/founder-population-declared-v1.json")
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-declared-founder-bundle-{}-{nonce}",
        std::process::id()
    ))
}
