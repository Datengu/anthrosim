use std::{io, path::PathBuf, process::Command};

fn run_python_script(script: &std::path::Path) {
    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        match Command::new(&python).arg(script).output() {
            Ok(output) => {
                assert!(
                    output.status.success(),
                    "Python scientific-analysis test {} failed with {}\nstdout:\n{}\nstderr:\n{}",
                    script.display(),
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch Python scientific-analysis test {} with {}: {error}",
                script.display(),
                python.display()
            ),
        }
    }

    eprintln!(
        "skipping Python scientific-analysis test {} because no Python interpreter was found",
        script.display()
    );
}

#[test]
fn research_monte_carlo_sufficiency_script_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    run_python_script(&repo_root.join("scripts/test-research-monte-carlo-sufficiency.py"));
}

#[test]
fn audit_v3_independent_difference_pairing_adversary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    run_python_script(&repo_root.join("scripts/audit-v3-test-monte-carlo-independent-pairing.py"));
}
