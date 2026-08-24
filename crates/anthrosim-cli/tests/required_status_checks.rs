use std::{io, path::PathBuf, process::Command};

#[test]
fn required_status_checks_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let script = repo_root.join("scripts/validate-required-status-checks.py");

    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        match Command::new(&python).arg(&script).output() {
            Ok(output) => {
                assert!(
                    output.status.success(),
                    "required status-check contract failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch required status-check validator with {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!("skipping required status-check contract because no Python interpreter was found");
}
