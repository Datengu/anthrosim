use std::{io, path::PathBuf, process::Command};

fn run_python_regression(script_name: &str) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let test_script = repo_root.join("scripts").join(script_name);

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
                    "research Monte Carlo regression {script_name} failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch research Monte Carlo regression {script_name} with {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!("skipping {script_name} because no Python interpreter was found");
}

#[test]
fn research_monte_carlo_sufficiency_script_contract() {
    run_python_regression("test-research-monte-carlo-sufficiency.py");
}

#[test]
fn research_monte_carlo_sequential_stopping_contract() {
    run_python_regression("test-research-monte-carlo-sequential-stopping.py");
}
