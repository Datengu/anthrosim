use std::{fs, io, path::PathBuf, process::Command};

use anthrosim_core::StudyProtocol;

#[test]
fn research_evidence_role_audit_script_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");

    let example_path = repo_root.join("examples/study-evidence-role-firewall-v1.json");
    let example: StudyProtocol = serde_json::from_slice(
        &fs::read(&example_path).expect("read evidence-role firewall example"),
    )
    .expect("deserialize evidence-role firewall example as StudyProtocol");
    example
        .validate()
        .expect("evidence-role firewall example must satisfy StudyProtocol v1");

    let test_script = repo_root.join("scripts/test-research-evidence-role-audit.py");
    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        match Command::new(&python).arg(&test_script).output() {
            Ok(output) => {
                assert!(
                    output.status.success(),
                    "research evidence-role audit regression suite failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch research evidence-role audit regression suite with {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!(
        "skipping research evidence-role audit regression suite because no Python interpreter was found"
    );
}
