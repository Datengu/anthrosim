use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[test]
fn spatial_ensemble_preserves_declared_founder_definition() {
    let root = repository_root();
    let founder = root.join("examples/founder-population-declared-v1.json");
    let landscape = root.join("examples/m8-first-evidence-grounded-benchmark/landscape.json");
    let mechanisms =
        root.join("examples/m8-first-evidence-grounded-benchmark/spatial-mechanisms-flat.json");
    let evidence = root.join("examples/m8-first-evidence-grounded-benchmark/evidence.json");
    let run_dir = unique_temp_dir();

    let output = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "ensemble",
            "--seeds",
            "21301",
            "--years",
            "0",
            "--world-width",
            "16",
            "--world-height",
            "16",
            "--founder-population",
            founder.to_str().expect("founder path must be UTF-8"),
            "--landscape",
            landscape.to_str().expect("landscape path must be UTF-8"),
            "--mechanisms",
            mechanisms.to_str().expect("mechanisms path must be UTF-8"),
            "--evidence",
            evidence.to_str().expect("evidence path must be UTF-8"),
            "--disable-migration",
            "--run-dir",
            run_dir.to_str().expect("run directory must be UTF-8"),
        ])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(
        output.status.success(),
        "spatial declared-founder ensemble failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest_bytes =
        fs::read(run_dir.join("experiment-manifest.json")).expect("experiment manifest");
    let manifest: serde_json::Value =
        serde_json::from_slice(&manifest_bytes).expect("valid experiment manifest JSON");
    let run = &manifest["runs"][0];

    assert_eq!(run["experiment"]["population"]["initialPopulation"], 2);
    assert_eq!(
        run["experiment"]["population"]["initialization"],
        "declared_founder_state_v1"
    );
    assert_eq!(
        run["experiment"]["founderPopulation"]["initializationId"],
        "declared-founder-cli-example-v1"
    );
    assert_eq!(
        run["spatial"]["founderPopulation"]["initializationId"],
        "declared-founder-cli-example-v1"
    );

    fs::remove_dir_all(&run_dir).expect("temp run directory should clean up");
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-spatial-declared-founder-{}-{nonce}",
        std::process::id()
    ))
}
