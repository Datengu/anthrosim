use std::{fs, path::PathBuf, process::Command, time::SystemTime};

#[test]
fn demographic_observability_derives_and_checks_normal_run_bundle() {
    let run_dir = unique_temp_dir();
    let run = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "run",
            "--seed",
            "24680",
            "--years",
            "2",
            "--world-width",
            "4",
            "--world-height",
            "4",
            "--population",
            "80",
            "--max-person-records",
            "1000",
            "--disable-migration",
            "--run-dir",
            run_dir.to_str().expect("temp path must be UTF-8"),
        ])
        .output()
        .expect("anthrosim CLI should execute");
    assert!(
        run.status.success(),
        "run CLI failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );

    let derive = Command::new(env!("CARGO_BIN_EXE_anthrosim-demography-observability"))
        .args([
            "--run-dir",
            run_dir.to_str().expect("temp path must be UTF-8"),
        ])
        .output()
        .expect("demography observability CLI should execute");
    assert!(
        derive.status.success(),
        "observability CLI failed: {}",
        String::from_utf8_lossy(&derive.stderr)
    );

    let report_path = run_dir.join("demography-observability.json");
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&report_path).expect("report should be written"))
            .expect("report should be valid JSON");
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["requestedBirthSpacingDays"], 1_278);
    assert_eq!(report["effectiveBirthSpacingDays"], 1_460);
    assert_eq!(
        report["fertilityProbabilityIsConditionalOnM2Survival"],
        true
    );
    assert_eq!(report["parentageUsesPreSameDayM4Residence"], true);

    let check = Command::new(env!("CARGO_BIN_EXE_anthrosim-demography-observability"))
        .args([
            "--run-dir",
            run_dir.to_str().expect("temp path must be UTF-8"),
            "--check",
            report_path.to_str().expect("report path must be UTF-8"),
        ])
        .output()
        .expect("demography observability check should execute");
    assert!(
        check.status.success(),
        "observability check failed: {}",
        String::from_utf8_lossy(&check.stderr)
    );

    fs::remove_dir_all(&run_dir).expect("temp run directory should clean up");
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-demography-observability-{}-{nonce}",
        std::process::id()
    ))
}
