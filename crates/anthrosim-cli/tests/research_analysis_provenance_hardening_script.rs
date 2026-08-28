use std::{io, path::PathBuf, process::Command};

#[test]
fn research_analysis_provenance_hardening_script_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let test_script = repo_root.join("scripts/test-research-analysis-provenance-hardening.py");

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
                    "research analysis-provenance hardening suite failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch research analysis-provenance hardening suite with {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!(
        "skipping research analysis-provenance hardening suite because no Python interpreter was found"
    );
}
