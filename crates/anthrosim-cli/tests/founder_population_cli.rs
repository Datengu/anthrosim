use std::{path::PathBuf, process::Command};

#[test]
fn run_accepts_declared_founder_population_file() {
    let example = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/founder-population-declared-v1.json");
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
