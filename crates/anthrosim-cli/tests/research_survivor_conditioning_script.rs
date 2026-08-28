use std::{path::PathBuf, process::Command};

fn python() -> PathBuf {
    for candidate in ["python3", "python"] {
        if Command::new(candidate)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return PathBuf::from(candidate);
        }
    }
    panic!("python interpreter required for survivor-conditioning regression test");
}

#[test]
fn survivor_conditioning_research_gate_regressions_pass() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf();
    let status = Command::new(python())
        .current_dir(&root)
        .arg("scripts/test-research-survivor-conditioning.py")
        .status()
        .expect("run survivor-conditioning regression script");
    assert!(
        status.success(),
        "survivor-conditioning regression script failed"
    );
}
