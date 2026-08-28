use std::{io, path::PathBuf, process::Command};

#[test]
fn research_sweep_exposure_script_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let test_scripts = [
        repo_root.join("scripts/test-research-sweep-exposure.py"),
        repo_root.join("scripts/test-issue-226-m8-fixed-horizon.py"),
    ];

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
                for test_script in &test_scripts {
                    let output = Command::new(&python)
                        .arg(test_script)
                        .output()
                        .expect("Python interpreter already probed successfully");
                    assert!(
                        output.status.success(),
                        "sweep exposure regression suite {} failed with {}\nstdout:\n{}\nstderr:\n{}",
                        test_script.display(),
                        python.display(),
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                return;
            }
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!("failed to probe Python interpreter {}: {error}", python.display()),
        }
    }

    eprintln!("skipping sweep exposure regression suite because no Python interpreter was found");
}
