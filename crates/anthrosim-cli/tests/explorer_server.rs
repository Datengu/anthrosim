use std::{io, path::PathBuf, process::Command};

#[test]
fn explorer_server_contract() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("anthrosim-cli must live under crates/<name>");
    let test_script = repo_root.join("scripts/test-serve-explorer.py");

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
                    "Explorer server regression suite failed with {}\nstdout:\n{}\nstderr:\n{}",
                    python.display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "failed to launch Explorer server regression suite with {}: {error}",
                python.display()
            ),
        }
    }

    eprintln!("skipping Explorer server regression suite because no Python interpreter was found");
}
