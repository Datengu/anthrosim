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
use serde_json::{Value, json};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temp_root() -> PathBuf {
    let id = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anthrosim-audit-v4-area-k-retry-tamper-{}-{id}",
        std::process::id()
    ))
}

fn tiny_definition(duration_years: u64) -> ResearchExperimentDefinition {
    let mut experiment = ExperimentConfig::new(540_001, duration_years)
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
        seeds: vec![540_001],
        base: ResearchRunConfig {
            experiment,
            spatial: None,
        },
        dimensions: vec![],
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}

fn read_value(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).expect("read json")).expect("parse json")
}

fn run_research(definition: &Path, run_dir: &Path, retry: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_anthrosim-research"));
    command
        .arg("--definition")
        .arg(definition)
        .arg("--run-dir")
        .arg(run_dir);
    if retry {
        command.arg("--retry");
    }
    command.output().expect("launch anthrosim-research")
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create copied root");
    for entry in fs::read_dir(source).expect("read source tree") {
        let entry = entry.expect("read source entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().expect("inspect source entry");
        assert!(!file_type.is_symlink(), "baseline must contain no symlinks");
        if file_type.is_dir() {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).expect("copy source file");
        }
    }
}

fn child_run_dir(root: &Path) -> PathBuf {
    let manifest = read_value(&root.join("research-manifest.json"));
    let relative = manifest["points"][0]["runs"][0]["relativeDir"]
        .as_str()
        .expect("planned relativeDir must be a string");
    root.join(relative)
}

fn assert_retry_fails(definition: &Path, run_dir: &Path, label: &str) {
    let output = run_research(definition, run_dir, true);
    assert!(
        !output.status.success(),
        "tamper case {label} unexpectedly survived retry; stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn retry_and_completed_bundle_tamper_matrix_is_fail_closed() {
    let fixture = temp_root();
    fs::create_dir_all(&fixture).expect("create fixture root");
    let definition_path = fixture.join("definition.json");
    let changed_definition_path = fixture.join("definition-changed.json");
    write_json(&definition_path, &tiny_definition(1));
    write_json(&changed_definition_path, &tiny_definition(2));

    let baseline = fixture.join("baseline");
    let initial = run_research(&definition_path, &baseline, false);
    assert!(
        initial.status.success(),
        "baseline research execution failed: {}",
        String::from_utf8_lossy(&initial.stderr)
    );

    let mut cases = 0_u32;

    // Positive control 1: exact retry accepts completed children without reexecution.
    let clean = fixture.join("clean-retry");
    copy_tree(&baseline, &clean);
    let clean_retry = run_research(&definition_path, &clean, true);
    assert!(clean_retry.status.success(), "exact retry must succeed");
    let clean_state = read_value(&clean.join("research-state.json"));
    let attempts = clean_state["runs"]
        .as_object()
        .expect("runs object")
        .values()
        .map(|run| run["attempt"].as_u64().expect("attempt"))
        .collect::<Vec<_>>();
    assert_eq!(attempts, vec![1], "clean retry must not reexecute");
    cases += 1;

    // Positive control 2: one immutable copy plus child bundles recovers missing
    // redundant metadata and malformed mutable state.
    let recoverable = fixture.join("recoverable");
    copy_tree(&baseline, &recoverable);
    fs::remove_file(recoverable.join("research-manifest.json")).expect("remove manifest");
    fs::write(recoverable.join("research-state.json"), b"{malformed").expect("damage state");
    let recovery = run_research(&definition_path, &recoverable, true);
    assert!(recovery.status.success(), "recoverable retry must succeed");
    assert_eq!(
        fs::read(recoverable.join("research-plan.json")).expect("read plan"),
        fs::read(recoverable.join("research-manifest.json")).expect("read recovered manifest"),
        "redundant immutable metadata must be restored exactly"
    );
    cases += 1;

    // Attack 1: a valid-but-contradictory immutable plan must not be repaired from
    // the still-correct manifest.
    let conflicting_plan = fixture.join("conflicting-plan");
    copy_tree(&baseline, &conflicting_plan);
    let mut plan = read_value(&conflicting_plan.join("research-plan.json"));
    plan["researchId"] = Value::String("research-execution-v1-tampered".to_owned());
    write_json(&conflicting_plan.join("research-plan.json"), &plan);
    assert_retry_fails(&definition_path, &conflicting_plan, "conflicting immutable plan");
    cases += 1;

    // Attack 2: if both immutable copies are damaged there is no authoritative
    // corroborating root and retry must fail.
    let both_damaged = fixture.join("both-immutable-damaged");
    copy_tree(&baseline, &both_damaged);
    fs::write(both_damaged.join("research-plan.json"), b"{broken-plan").expect("damage plan");
    fs::write(
        both_damaged.join("research-manifest.json"),
        b"{broken-manifest",
    )
    .expect("damage manifest");
    assert_retry_fails(&definition_path, &both_damaged, "both immutable copies damaged");
    cases += 1;

    // Attack 3: same root under a changed complete scientific definition fails.
    let changed_definition = fixture.join("changed-definition");
    copy_tree(&baseline, &changed_definition);
    assert_retry_fails(
        &changed_definition_path,
        &changed_definition,
        "changed research definition",
    );
    cases += 1;

    // Attack 4: completed child source provenance altered while configuration is
    // otherwise preserved.
    let child_source = fixture.join("child-source");
    copy_tree(&baseline, &child_source);
    let child = child_run_dir(&child_source);
    let mut manifest = read_value(&child.join("manifest.json"));
    manifest["gitCommit"] = Value::String("0000000000000000000000000000000000000000".to_owned());
    write_json(&child.join("manifest.json"), &manifest);
    assert_retry_fails(&definition_path, &child_source, "child source provenance");
    cases += 1;

    // Attack 5: a validly encoded but changed checkpoint digest fails semantic
    // bundle validation/reconciliation.
    let checkpoint_digest = fixture.join("checkpoint-digest");
    copy_tree(&baseline, &checkpoint_digest);
    let child = child_run_dir(&checkpoint_digest);
    let mut checkpoint = read_value(&child.join("checkpoint.json"));
    let digest = checkpoint["stateDigest64"].as_u64().expect("state digest");
    checkpoint["stateDigest64"] = Value::from(digest.wrapping_add(1));
    write_json(&child.join("checkpoint.json"), &checkpoint);
    assert_retry_fails(&definition_path, &checkpoint_digest, "checkpoint state digest");
    cases += 1;

    // Attack 6: standalone events remain valid JSON/EventLog shape but disagree
    // with the checkpoint.
    let events_tamper = fixture.join("events-tamper");
    copy_tree(&baseline, &events_tamper);
    let child = child_run_dir(&events_tamper);
    let mut events = read_value(&child.join("events.json"));
    let event_schema = events["schemaVersion"].as_u64().expect("event schema");
    events["schemaVersion"] = Value::from(event_schema + 1);
    write_json(&child.join("events.json"), &events);
    assert_retry_fails(&definition_path, &events_tamper, "events semantic mismatch");
    cases += 1;

    // Attack 7: standalone metrics likewise disagree with checkpoint metrics.
    let metrics_tamper = fixture.join("metrics-tamper");
    copy_tree(&baseline, &metrics_tamper);
    let child = child_run_dir(&metrics_tamper);
    let mut metrics = read_value(&child.join("metrics.json"));
    let metric_schema = metrics["schemaVersion"].as_u64().expect("metric schema");
    metrics["schemaVersion"] = Value::from(metric_schema + 1);
    write_json(&child.join("metrics.json"), &metrics);
    assert_retry_fails(&definition_path, &metrics_tamper, "metrics semantic mismatch");
    cases += 1;

    // Attack 8: world dimensions remain syntactically valid but no longer match
    // the checkpoint/world digest and cell geometry.
    let world_tamper = fixture.join("world-tamper");
    copy_tree(&baseline, &world_tamper);
    let child = child_run_dir(&world_tamper);
    let mut world = read_value(&child.join("world.json"));
    let width = world["width"].as_u64().expect("world width");
    world["width"] = Value::from(width + 1);
    write_json(&child.join("world.json"), &world);
    assert_retry_fails(&definition_path, &world_tamper, "world semantic mismatch");
    cases += 1;

    // Attack 9: completion marker remains structurally parseable but claims the
    // wrong seed.
    let completion_tamper = fixture.join("completion-tamper");
    copy_tree(&baseline, &completion_tamper);
    let child = child_run_dir(&completion_tamper);
    let mut completion = read_value(&child.join("completion.json"));
    completion["seed"] = json!(540_002_u64);
    write_json(&child.join("completion.json"), &completion);
    assert_retry_fails(&definition_path, &completion_tamper, "completion seed mismatch");
    cases += 1;

    assert_eq!(cases, 11);
    println!("audit_v4_area_k_retry_tamper_cases={cases}");
    println!("audit_v4_area_k_retry_tamper_status=pass");

    fs::remove_dir_all(fixture).expect("cleanup fixture root");
}
