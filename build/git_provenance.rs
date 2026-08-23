use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn emit_git_provenance() {
    println!("cargo:rerun-if-env-changed=ANTHROSIM_GIT_COMMIT");

    if let Some(overridden) = env::var("ANTHROSIM_GIT_COMMIT")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
    {
        println!("cargo:rustc-env=ANTHROSIM_GIT_COMMIT={overridden}");
        return;
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo must provide CARGO_MANIFEST_DIR"),
    );
    let Some(repo_root) = git_output(&manifest_dir, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
    else {
        println!(
            "cargo:warning=AnthroSim Git source revision unavailable; run provenance will record gitCommit=null"
        );
        return;
    };

    watch_repository_state(&repo_root);

    let Some(commit) = git_output(&repo_root, &["rev-parse", "--verify", "HEAD"]) else {
        println!(
            "cargo:warning=AnthroSim Git HEAD could not be resolved; run provenance will record gitCommit=null"
        );
        return;
    };
    let Some(status) = git_output(
        &repo_root,
        &["status", "--porcelain", "--untracked-files=no"],
    ) else {
        println!(
            "cargo:warning=AnthroSim Git working-tree state could not be resolved; run provenance will record gitCommit=null"
        );
        return;
    };

    if status.trim().is_empty() {
        println!("cargo:rustc-env=ANTHROSIM_GIT_COMMIT={commit}");
    } else if let Some(dirty_identity) = dirty_tree_identity(&repo_root) {
        let revision = format!("{commit}-dirty-{dirty_identity}");
        println!("cargo:rustc-env=ANTHROSIM_GIT_COMMIT={revision}");
        println!(
            "cargo:warning=AnthroSim source tree has tracked modifications; provenance will record {revision}"
        );
    } else {
        println!(
            "cargo:warning=AnthroSim tracked source tree is dirty but its exact content identity could not be resolved; run provenance will record gitCommit=null"
        );
    }
}

fn dirty_tree_identity(repo_root: &Path) -> Option<String> {
    let diff = git_bytes(
        repo_root,
        &[
            "diff",
            "HEAD",
            "--binary",
            "--full-index",
            "--no-ext-diff",
            "--no-textconv",
            "--",
        ],
    )?;
    if diff.is_empty() {
        return None;
    }

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    {
        let stdin = child.stdin.as_mut()?;
        stdin.write_all(&diff).ok()?;
    }
    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn git_bytes(directory: &Path, args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(output.stdout)
}

fn git_output(directory: &Path, args: &[&str]) -> Option<String> {
    String::from_utf8(git_bytes(directory, args)?)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn watch_repository_state(repo_root: &Path) {
    for git_path in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_output(repo_root, &["rev-parse", "--git-path", git_path]) {
            emit_watch_path(repo_root, &path);
        }
    }

    if let Some(symbolic_ref) = git_output(repo_root, &["symbolic-ref", "-q", "HEAD"])
        && let Some(path) = git_output(
            repo_root,
            &["rev-parse", "--git-path", symbolic_ref.as_str()],
        )
    {
        emit_watch_path(repo_root, &path);
    }

    if let Some(files) = git_output(repo_root, &["ls-files"]) {
        for relative in files.lines().filter(|line| !line.is_empty()) {
            println!(
                "cargo:rerun-if-changed={}",
                repo_root.join(relative).display()
            );
        }
    }
}

fn emit_watch_path(repo_root: &Path, value: &str) {
    let path = Path::new(value);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };
    println!("cargo:rerun-if-changed={}", resolved.display());
}
