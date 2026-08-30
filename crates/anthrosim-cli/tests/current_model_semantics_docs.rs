use std::{io, path::PathBuf, process::Command};

#[test]
fn current_model_semantics_documentation_matches_source_identity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let script = repo_root.join("scripts/test-current-model-semantics-docs.py");

    let mut candidates = Vec::new();
    if let Some(python) = std::env::var_os("PYTHON") {
        candidates.push(PathBuf::from(python));
    }
    candidates.push(PathBuf::from("python3"));
    candidates.push(PathBuf::from("python"));

    for python in candidates {
        let probe = Command::new(&python).arg("--version").output();
        match probe {
            Ok(output) if output.status.success() => {
                let output = Command::new(&python)
                    .arg(&script)
                    .output()
                    .expect("Python interpreter already probed successfully");
                assert!(
                    output.status.success(),
                    "current model-semantics documentation check failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to probe Python interpreter {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!(
        "skipping current model-semantics documentation check because no Python interpreter was found"
    );
}
