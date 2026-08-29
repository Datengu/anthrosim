use std::{fs, path::PathBuf};

use serde_json::Value;

#[test]
fn confirmatory_demography_reference_is_bound_to_current_model_semantics() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");

    let result: Value = serde_json::from_slice(
        &fs::read(
            repo_root.join("research/general-demography-baseline-v1/confirmatory-result.json"),
        )
        .expect("read canonical demographic confirmatory result"),
    )
    .expect("parse canonical demographic confirmatory result");
    assert_eq!(
        result["modelSemanticsId"].as_str(),
        Some(anthrosim_core::provenance::MODEL_SEMANTICS_ID),
        "the canonical issue #304 reference must identify the authoritative model semantics that produced it"
    );

    let workflow = fs::read_to_string(
        repo_root.join(".github/workflows/general-demography-baseline-analysis.yml"),
    )
    .expect("read issue #304 confirmatory workflow");
    assert!(
        workflow.contains("- \"crates/anthrosim-core/src/**\""),
        "all core-model source changes must trigger the issue #304 confirmatory reference check"
    );
    assert!(
        workflow.contains("'modelSemanticsId':model_semantics_id"),
        "the reproducible expected result must bind the authoritative MODEL_SEMANTICS_ID"
    );
}
