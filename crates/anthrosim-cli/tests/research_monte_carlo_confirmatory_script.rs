use std::{io, path::PathBuf, process::Command};

#[test]
fn research_monte_carlo_confirmatory_seed_binding_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let test_scripts = [
        repo_root.join("scripts/test-research-monte-carlo-confirmatory.py"),
        repo_root.join("scripts/test-research-monte-carlo-sample-binding.py"),
    ];

    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        let mut interpreter_found = false;
        for test_script in &test_scripts {
            match Command::new(&python).arg(test_script).output() {
                Ok(output) => {
                    interpreter_found = true;
                    assert!(
                        output.status.success(),
                        "confirmatory Monte Carlo binding suite {} failed with {}\nstdout:\n{}\nstderr:\n{}",
                        test_script.display(),
                        python.display(),
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => break,
                Err(error) => panic!(
                    "failed to launch confirmatory Monte Carlo binding suite {} with {}: {error}",
                    test_script.display(),
                    python.display()
                ),
            }
        }
        if interpreter_found {
            return;
        }
    }

    eprintln!(
        "skipping confirmatory Monte Carlo binding suites because no Python interpreter was found"
    );
}
