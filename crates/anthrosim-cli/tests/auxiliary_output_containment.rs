use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

#[test]
fn run_rejects_direct_manifest_output_inside_new_run_dir_before_writing() {
    let root = unique_temp_root("direct-output");
    let run_dir = root.join("run");
    let output_path = run_dir.join("checkpoint.json");

    let output = base_run_command()
        .args(["--run-dir", utf8(&run_dir), "--output", utf8(&output_path)])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("inside controlled run directory"));
    assert!(!run_dir.exists());
    cleanup(&root);
}

#[test]
fn run_rejects_parent_alias_before_existing_bundle_is_modified() {
    let root = unique_temp_root("parent-alias");
    let run_dir = root.join("run");
    fs::create_dir_all(run_dir.join("nested")).unwrap();
    let checkpoint = run_dir.join("checkpoint.json");
    fs::write(&checkpoint, "sentinel\n").unwrap();
    let alias = run_dir.join("nested").join("..").join("checkpoint.json");

    let output = base_run_command()
        .args(["--run-dir", utf8(&run_dir), "--world-output", utf8(&alias)])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&checkpoint).unwrap(), "sentinel\n");
    cleanup(&root);
}

#[test]
fn run_rejects_auxiliary_outputs_that_alias_each_other() {
    let root = unique_temp_root("aux-collision");
    fs::create_dir_all(&root).unwrap();
    let output_path = root.join("state.json");

    let output = base_run_command()
        .args([
            "--world-output",
            utf8(&output_path),
            "--population-output",
            utf8(&output_path),
        ])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("auxiliary outputs must resolve to distinct files")
    );
    assert!(!output_path.exists());
    cleanup(&root);
}

#[cfg(unix)]
#[test]
fn run_rejects_symlink_alias_into_existing_bundle() {
    use std::os::unix::fs::symlink;

    let root = unique_temp_root("symlink-alias");
    let run_dir = root.join("run");
    fs::create_dir_all(&run_dir).unwrap();
    let checkpoint = run_dir.join("checkpoint.json");
    fs::write(&checkpoint, "sentinel\n").unwrap();
    let alias_dir = root.join("run-alias");
    symlink(&run_dir, &alias_dir).unwrap();
    let alias = alias_dir.join("checkpoint.json");

    let output = base_run_command()
        .args([
            "--run-dir",
            utf8(&run_dir),
            "--population-output",
            utf8(&alias),
        ])
        .output()
        .expect("anthrosim CLI should execute");

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&checkpoint).unwrap(), "sentinel\n");
    cleanup(&root);
}

#[test]
fn resume_rejects_manifest_output_inside_in_place_bundle_before_mutation() {
    let root = unique_temp_root("resume-output");
    let run_dir = root.join("run");
    let create = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "run",
            "--years",
            "2",
            "--checkpoint-year",
            "1",
            "--population",
            "10",
            "--world-width",
            "2",
            "--world-height",
            "2",
            "--disable-migration",
            "--run-dir",
            utf8(&run_dir),
        ])
        .output()
        .expect("checkpoint run should execute");
    assert!(
        create.status.success(),
        "checkpoint run failed: {}",
        String::from_utf8_lossy(&create.stderr)
    );

    let checkpoint = run_dir.join("checkpoint.json");
    let before = fs::read(&checkpoint).unwrap();
    let output_path = run_dir.join("manifest.json");
    let resume = Command::new(env!("CARGO_BIN_EXE_anthrosim"))
        .args([
            "resume",
            "--checkpoint",
            utf8(&checkpoint),
            "--run-dir",
            utf8(&run_dir),
            "--output",
            utf8(&output_path),
        ])
        .output()
        .expect("resume CLI should execute");

    assert!(!resume.status.success());
    assert_eq!(fs::read(&checkpoint).unwrap(), before);
    assert!(!output_path.exists());
    cleanup(&root);
}

fn base_run_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_anthrosim"));
    command.args([
        "run",
        "--years",
        "0",
        "--population",
        "2",
        "--world-width",
        "1",
        "--world-height",
        "1",
        "--disable-migration",
    ]);
    command
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("test path must be UTF-8")
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anthrosim-aux-output-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}
