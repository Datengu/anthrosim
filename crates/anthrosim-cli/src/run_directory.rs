use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anthrosim_core::{Population, SimulationCheckpoint, World, rng::RngFactory};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);
const REPLACEMENT_TRANSACTION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitMode {
    Fresh,
    ReplaceVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplacementTransactionMarker {
    schema_version: u32,
    target_name: String,
    staging_name: String,
    backup_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryOutcome {
    NoRecoveryNeeded,
    AbandonedStageRemoved,
    PreviousBundleRestored,
    PromotedBundleKept,
    CompletedMarkerRemoved,
    LegacyStageRemoved,
    LegacyPreviousBundleRestored,
}

impl fmt::Display for RecoveryOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NoRecoveryNeeded => "no interrupted run-directory transaction was found",
            Self::AbandonedStageRemoved => {
                "removed an abandoned staged replacement; canonical bundle was unchanged"
            }
            Self::PreviousBundleRestored => {
                "restored the verified previous canonical bundle and discarded the interrupted stage"
            }
            Self::PromotedBundleKept => {
                "kept the already-promoted canonical bundle and removed its stale verified backup"
            }
            Self::CompletedMarkerRemoved => {
                "removed stale recovery metadata for an already-completed replacement"
            }
            Self::LegacyStageRemoved => {
                "removed one unmarked legacy staging directory while keeping the canonical bundle"
            }
            Self::LegacyPreviousBundleRestored => {
                "restored one unmarked legacy verified backup and discarded its abandoned stage"
            }
        })
    }
}

/// Builds a run bundle in a clean sibling directory and promotes it only after
/// the caller has finished writing and validating the complete artifact set.
#[derive(Debug)]
pub(crate) struct RunDirectoryTransaction {
    target: PathBuf,
    staging: PathBuf,
    backup: Option<PathBuf>,
    marker: Option<PathBuf>,
    mode: CommitMode,
    committed: bool,
}

impl RunDirectoryTransaction {
    /// Start a new run/checkpoint bundle. Existing non-empty targets are never
    /// overwritten implicitly.
    pub(crate) fn fresh(target: &Path) -> io::Result<Self> {
        require_no_unresolved_transaction(target)?;
        require_fresh_target(target)?;
        Self::start(target, CommitMode::Fresh)
    }

    /// Start a replacement after the caller has verified that the existing
    /// target is the source bundle for an explicit in-place resume.
    pub(crate) fn replace_verified(target: &Path) -> io::Result<Self> {
        require_no_unresolved_transaction(target)?;
        require_nonempty_directory(target)?;
        Self::start(target, CommitMode::ReplaceVerified)
    }

    pub(crate) fn staging_dir(&self) -> &Path {
        &self.staging
    }

    pub(crate) fn commit(mut self) -> io::Result<()> {
        materialize_original_founders_for_resumed_bundle(&self.staging)?;
        match self.mode {
            CommitMode::Fresh => self.commit_fresh()?,
            CommitMode::ReplaceVerified => self.commit_replacement()?,
        }
        self.committed = true;
        Ok(())
    }

    fn start(target: &Path, mode: CommitMode) -> io::Result<Self> {
        reject_symlink_target(target)?;
        let parent = usable_parent(target);
        fs::create_dir_all(parent)?;

        let staging = unique_sibling_path(target, "stage");
        fs::create_dir(&staging)?;

        let (backup, marker) = if mode == CommitMode::ReplaceVerified {
            let backup = unique_sibling_path(target, "backup");
            let marker = transaction_marker_path(target);
            let transaction = ReplacementTransactionMarker {
                schema_version: REPLACEMENT_TRANSACTION_SCHEMA_VERSION,
                target_name: target_base_name(target).to_owned(),
                staging_name: file_name_string(&staging)?,
                backup_name: file_name_string(&backup)?,
            };
            if let Err(error) = write_json(&marker, &transaction) {
                let _ = fs::remove_dir_all(&staging);
                return Err(error);
            }
            (Some(backup), Some(marker))
        } else {
            (None, None)
        };

        Ok(Self {
            target: target.to_owned(),
            staging,
            backup,
            marker,
            mode,
            committed: false,
        })
    }

    fn commit_fresh(&self) -> io::Result<()> {
        if self.target.exists() {
            reject_symlink_target(&self.target)?;
            if !self.target.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "run directory target exists and is not a directory: {}",
                        self.target.display()
                    ),
                ));
            }
            if directory_has_entries(&self.target)? {
                return Err(nonempty_target_error(&self.target));
            }
            fs::remove_dir(&self.target)?;
        }
        fs::rename(&self.staging, &self.target)
    }

    fn commit_replacement(&self) -> io::Result<()> {
        require_nonempty_directory_without_recovery_check(&self.target)?;
        let backup = self.backup.as_ref().ok_or_else(|| {
            io::Error::other("verified replacement transaction is missing its backup path")
        })?;
        let marker = self.marker.as_ref().ok_or_else(|| {
            io::Error::other("verified replacement transaction is missing its recovery marker")
        })?;

        fs::rename(&self.target, backup)?;
        match fs::rename(&self.staging, &self.target) {
            Ok(()) => {}
            Err(error) => {
                let restore = fs::rename(backup, &self.target);
                if restore.is_ok() {
                    let _ = fs::remove_file(marker);
                    return Err(io::Error::new(
                        error.kind(),
                        format!(
                            "could not promote staged resumed run into {}: {error}; previous verified bundle was restored",
                            self.target.display()
                        ),
                    ));
                }
                return Err(io::Error::other(format!(
                    "could not promote staged resumed run into {}: {error}; also failed to restore previous verified bundle from {}: {}; recovery metadata was retained for anthrosim-recover",
                    self.target.display(),
                    backup.display(),
                    restore.unwrap_err()
                )));
            }
        }

        if let Err(error) = fs::remove_dir_all(backup) {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "resumed run is committed at {}, but cleanup of verified previous bundle {} is pending: {error}; run anthrosim-recover --run-dir {}",
                    self.target.display(),
                    backup.display(),
                    self.target.display()
                ),
            ));
        }
        if let Err(error) = fs::remove_file(marker) {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "resumed run is committed at {} and its previous backup is removed, but recovery-marker cleanup is pending at {}: {error}; run anthrosim-recover --run-dir {}",
                    self.target.display(),
                    marker.display(),
                    self.target.display()
                ),
            ));
        }
        Ok(())
    }
}

impl Drop for RunDirectoryTransaction {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if self.staging.exists() {
            let _ = fs::remove_dir_all(&self.staging);
        }
        if let (Some(marker), Some(backup)) = (&self.marker, &self.backup)
            && !backup.exists()
            && self.target.exists()
        {
            let _ = fs::remove_file(marker);
        }
    }
}

/// Recover one interrupted replacement transaction for `target` without ever
/// promoting an abandoned staging directory. Marked transactions can be
/// resolved deterministically from the bound target/stage/backup paths. A
/// narrow legacy path restores the old verified backup when the pre-marker
/// implementation was interrupted after moving the canonical target aside.
#[allow(dead_code)]
pub(crate) fn recover_interrupted_replacement(target: &Path) -> io::Result<RecoveryOutcome> {
    let marker_path = transaction_marker_path(target);
    if marker_path.exists() {
        return recover_marked_transaction(target, &marker_path);
    }
    recover_legacy_remnants(target)
}

pub(crate) fn target_is_nonempty_directory(path: &Path) -> io::Result<bool> {
    require_no_unresolved_transaction(path)?;
    target_is_nonempty_directory_without_recovery_check(path)
}

pub(crate) fn same_existing_path(left: &Path, right: &Path) -> io::Result<bool> {
    Ok(fs::canonicalize(left)? == fs::canonicalize(right)?)
}

#[allow(dead_code)]
fn recover_marked_transaction(target: &Path, marker_path: &Path) -> io::Result<RecoveryOutcome> {
    reject_symlink_target(marker_path)?;
    let marker: ReplacementTransactionMarker = read_json(marker_path)?;
    validate_marker(target, &marker)?;
    let parent = usable_parent(target);
    let staging = parent.join(&marker.staging_name);
    let backup = parent.join(&marker.backup_name);

    let target_exists = checked_directory_exists(target, "canonical target")?;
    let staging_exists = checked_directory_exists(&staging, "transaction staging directory")?;
    let backup_exists = checked_directory_exists(&backup, "transaction backup directory")?;

    match (target_exists, staging_exists, backup_exists) {
        (true, true, false) => {
            fs::remove_dir_all(&staging)?;
            fs::remove_file(marker_path)?;
            Ok(RecoveryOutcome::AbandonedStageRemoved)
        }
        (false, true, true) | (false, false, true) => {
            fs::rename(&backup, target)?;
            if staging.exists() {
                fs::remove_dir_all(&staging)?;
            }
            fs::remove_file(marker_path)?;
            Ok(RecoveryOutcome::PreviousBundleRestored)
        }
        (true, false, true) => {
            fs::remove_dir_all(&backup).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "promoted canonical bundle is present at {}, but stale verified backup {} could not be removed: {error}",
                        target.display(),
                        backup.display()
                    ),
                )
            })?;
            fs::remove_file(marker_path)?;
            Ok(RecoveryOutcome::PromotedBundleKept)
        }
        (true, false, false) => {
            fs::remove_file(marker_path)?;
            Ok(RecoveryOutcome::CompletedMarkerRemoved)
        }
        state => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "ambiguous interrupted run-directory transaction for {}: target/stage/backup presence is {:?}; no files were changed",
                target.display(),
                state
            ),
        )),
    }
}

#[allow(dead_code)]
fn recover_legacy_remnants(target: &Path) -> io::Result<RecoveryOutcome> {
    let (stages, backups) = legacy_remnants(target)?;
    if stages.is_empty() && backups.is_empty() {
        return Ok(RecoveryOutcome::NoRecoveryNeeded);
    }

    let target_exists = checked_directory_exists(target, "canonical target")?;
    match (target_exists, stages.as_slice(), backups.as_slice()) {
        (true, [stage], []) => {
            fs::remove_dir_all(stage)?;
            Ok(RecoveryOutcome::LegacyStageRemoved)
        }
        (false, [], [backup]) => {
            fs::rename(backup, target)?;
            Ok(RecoveryOutcome::LegacyPreviousBundleRestored)
        }
        (false, [stage], [backup]) => {
            fs::rename(backup, target)?;
            fs::remove_dir_all(stage)?;
            Ok(RecoveryOutcome::LegacyPreviousBundleRestored)
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "ambiguous unmarked AnthroSim transaction remnants for {} ({} stage, {} backup); no files were changed",
                target.display(),
                stages.len(),
                backups.len()
            ),
        )),
    }
}

fn require_no_unresolved_transaction(target: &Path) -> io::Result<()> {
    let marker = transaction_marker_path(target);
    let (stages, backups) = legacy_remnants(target)?;
    if marker.exists() || !stages.is_empty() || !backups.is_empty() {
        return Err(io::Error::other(format!(
            "run directory {} has interrupted transaction state; run anthrosim-recover --run-dir {} before starting or replacing it",
            target.display(),
            target.display()
        )));
    }
    Ok(())
}

#[allow(dead_code)]
fn validate_marker(target: &Path, marker: &ReplacementTransactionMarker) -> io::Result<()> {
    if marker.schema_version != REPLACEMENT_TRANSACTION_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported run-directory recovery marker schema {}; supported schema is {}",
                marker.schema_version, REPLACEMENT_TRANSACTION_SCHEMA_VERSION
            ),
        ));
    }
    let expected_target = target_base_name(target);
    if marker.target_name != expected_target {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "run-directory recovery marker targets {:?}, not {:?}",
                marker.target_name, expected_target
            ),
        ));
    }
    for (role, name) in [
        ("staging", marker.staging_name.as_str()),
        ("backup", marker.backup_name.as_str()),
    ] {
        if !is_safe_single_component(name) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("recovery marker has unsafe {role} path component {name:?}"),
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn checked_directory_exists(path: &Path, role: &str) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{role} may not be a symbolic link during recovery: {}",
                path.display()
            ),
        )),
        Ok(metadata) if !metadata.is_dir() => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{role} is not a directory during recovery: {}",
                path.display()
            ),
        )),
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn legacy_remnants(target: &Path) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let parent = usable_parent(target);
    if !parent.exists() {
        return Ok((Vec::new(), Vec::new()));
    }
    let base = target_base_name(target);
    let stage_prefix = format!(".{base}.anthrosim-stage-");
    let backup_prefix = format!(".{base}.anthrosim-backup-");
    let mut stages = Vec::new();
    let mut backups = Vec::new();
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if name.starts_with(&stage_prefix) {
            stages.push(entry.path());
        } else if name.starts_with(&backup_prefix) {
            backups.push(entry.path());
        }
    }
    stages.sort();
    backups.sort();
    Ok((stages, backups))
}

fn materialize_original_founders_for_resumed_bundle(staging: &Path) -> io::Result<()> {
    let resume_population_path = staging.join("resume-start-population.json");
    let initial_population_path = staging.join("initial-population.json");
    if !resume_population_path.is_file() || initial_population_path.is_file() {
        return Ok(());
    }

    let world: World = read_json(&staging.join("world.json"))?;
    world.validate().map_err(invalid_data)?;
    let checkpoint: SimulationCheckpoint = read_json(&staging.join("checkpoint.json"))?;
    if checkpoint.world_digest64 != world.digest64() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot reconstruct original founders: world.json does not match checkpoint.json",
        ));
    }

    let resume_population: Population = read_json(&resume_population_path)?;
    resume_population.validate(&world).map_err(invalid_data)?;

    // resume-start-population.json is a boundary snapshot, not a day-zero
    // population. Reconstruct the actual founders from immutable experiment
    // identity and the authoritative world, matching bundle/M8.5 semantics.
    let initial_population = Population::initialize(
        checkpoint.experiment.population,
        &world,
        RngFactory::new(checkpoint.experiment.seed),
    )
    .map_err(invalid_data)?;
    initial_population.validate(&world).map_err(invalid_data)?;
    write_json(&initial_population_path, &initial_population)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(invalid_data)
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(invalid_data)?;
    fs::write(path, format!("{json}\n"))
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn require_fresh_target(target: &Path) -> io::Result<()> {
    if target_is_nonempty_directory_without_recovery_check(target)? {
        return Err(nonempty_target_error(target));
    }
    Ok(())
}

fn require_nonempty_directory(target: &Path) -> io::Result<()> {
    require_no_unresolved_transaction(target)?;
    require_nonempty_directory_without_recovery_check(target)
}

fn require_nonempty_directory_without_recovery_check(target: &Path) -> io::Result<()> {
    if !target_is_nonempty_directory_without_recovery_check(target)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "verified in-place resume requires an existing non-empty run directory: {}",
                target.display()
            ),
        ));
    }
    Ok(())
}

fn target_is_nonempty_directory_without_recovery_check(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    reject_symlink_target(path)?;
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "run directory target exists and is not a directory: {}",
                path.display()
            ),
        ));
    }
    directory_has_entries(path)
}

fn nonempty_target_error(target: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "run directory is not empty: {}; choose a new directory, or use resume with the checkpoint stored in this exact run directory",
            target.display()
        ),
    )
}

fn reject_symlink_target(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "run directory target may not be a symbolic link: {}",
                path.display()
            ),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn directory_has_entries(path: &Path) -> io::Result<bool> {
    Ok(fs::read_dir(path)?.next().transpose()?.is_some())
}

fn usable_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn target_base_name(target: &Path) -> &str {
    target
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("run")
}

fn transaction_marker_path(target: &Path) -> PathBuf {
    usable_parent(target).join(format!(
        ".{}.anthrosim-transaction.json",
        target_base_name(target)
    ))
}

fn unique_sibling_path(target: &Path, role: &str) -> PathBuf {
    let parent = usable_parent(target);
    let base = target_base_name(target);
    loop {
        let id = NEXT_TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{base}.anthrosim-{role}-{}-{id}",
            std::process::id()
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
}

fn file_name_string(path: &Path) -> io::Result<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "transaction path has no UTF-8 file name",
            )
        })
}

#[allow(dead_code)]
fn is_safe_single_component(value: &str) -> bool {
    let path = Path::new(value);
    path.file_name().and_then(|name| name.to_str()) == Some(value)
        && path
            .parent()
            .is_some_and(|parent| parent.as_os_str().is_empty())
}

#[cfg(test)]
mod tests {
    use std::{
        mem,
        sync::atomic::{AtomicU64, Ordering},
    };

    use anthrosim_core::{ExperimentConfig, PopulationConfig, Simulation, WorldConfig};

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn fresh_synthetic_run_rejects_existing_spatial_bundle() {
        let target = test_dir("spatial-to-synthetic");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("spatial-mechanisms.json"), "old-spatial\n").unwrap();

        let error = RunDirectoryTransaction::fresh(&target).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(target.join("spatial-mechanisms.json")).unwrap(),
            "old-spatial\n"
        );
        cleanup(&target);
    }

    #[test]
    fn fresh_spatial_run_rejects_existing_synthetic_bundle() {
        let target = test_dir("synthetic-to-spatial");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("manifest.json"), "old-synthetic\n").unwrap();

        let error = RunDirectoryTransaction::fresh(&target).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(target.join("manifest.json")).unwrap(),
            "old-synthetic\n"
        );
        cleanup(&target);
    }

    #[test]
    fn fresh_bundle_is_invisible_until_commit() {
        let target = test_dir("fresh-promote");
        let transaction = RunDirectoryTransaction::fresh(&target).unwrap();
        fs::write(transaction.staging_dir().join("manifest.json"), "new\n").unwrap();

        assert!(!target.exists());
        transaction.commit().unwrap();
        assert_eq!(
            fs::read_to_string(target.join("manifest.json")).unwrap(),
            "new\n"
        );
        cleanup(&target);
    }

    #[test]
    fn resumed_bundle_materializes_true_original_founders_before_promotion() {
        let target = test_dir("resume-founders");
        let config = ExperimentConfig::new(7_701, 2)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(24).with_target_household_size(4));
        let simulation = Simulation::new(config).unwrap();
        let world = simulation.world().clone();
        let expected_founders = simulation.population().clone();
        let checkpoint = simulation.checkpoint_at_year(1).unwrap();

        let transaction = RunDirectoryTransaction::fresh(&target).unwrap();
        let staging = transaction.staging_dir();
        write_json(&staging.join("world.json"), &world).unwrap();
        write_json(&staging.join("checkpoint.json"), &checkpoint).unwrap();
        write_json(
            &staging.join("resume-start-population.json"),
            &checkpoint.population,
        )
        .unwrap();

        transaction.commit().unwrap();
        let actual: Population = read_json(&target.join("initial-population.json")).unwrap();
        assert_eq!(actual, expected_founders);
        assert!(target.join("resume-start-population.json").is_file());
        cleanup(&target);
    }

    #[test]
    fn verified_replacement_drops_stale_artifacts_as_one_bundle() {
        let target = test_dir("verified-replace");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("checkpoint.json"), "old-checkpoint\n").unwrap();
        fs::write(target.join("spatial-observability.json"), "stale\n").unwrap();

        let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
        fs::write(
            transaction.staging_dir().join("checkpoint.json"),
            "new-checkpoint\n",
        )
        .unwrap();
        transaction.commit().unwrap();

        assert_eq!(
            fs::read_to_string(target.join("checkpoint.json")).unwrap(),
            "new-checkpoint\n"
        );
        assert!(!target.join("spatial-observability.json").exists());
        assert!(!transaction_marker_path(&target).exists());
        cleanup(&target);
    }

    #[test]
    fn abandoned_transaction_keeps_existing_bundle_unchanged() {
        let target = test_dir("abandon");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("checkpoint.json"), "old\n").unwrap();
        {
            let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
            fs::write(
                transaction.staging_dir().join("checkpoint.json"),
                "partial\n",
            )
            .unwrap();
        }

        assert_eq!(
            fs::read_to_string(target.join("checkpoint.json")).unwrap(),
            "old\n"
        );
        assert!(!transaction_marker_path(&target).exists());
        cleanup(&target);
    }

    #[test]
    fn recovery_removes_hard_crashed_pre_promotion_stage() {
        let target = populated_target("recover-stage", "old");
        let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
        let staging = transaction.staging.clone();
        mem::forget(transaction);

        assert!(staging.is_dir());
        assert!(transaction_marker_path(&target).is_file());
        assert_eq!(
            recover_interrupted_replacement(&target).unwrap(),
            RecoveryOutcome::AbandonedStageRemoved
        );
        assert_eq!(
            fs::read_to_string(target.join("checkpoint.json")).unwrap(),
            "old\n"
        );
        assert!(!staging.exists());
        assert!(!transaction_marker_path(&target).exists());
        cleanup(&target);
    }

    #[test]
    fn recovery_restores_previous_bundle_after_backup_rename() {
        let target = populated_target("recover-backup", "old");
        let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
        let staging = transaction.staging.clone();
        let backup = transaction.backup.clone().unwrap();
        fs::write(staging.join("checkpoint.json"), "new\n").unwrap();
        fs::rename(&target, &backup).unwrap();
        mem::forget(transaction);

        assert_eq!(
            recover_interrupted_replacement(&target).unwrap(),
            RecoveryOutcome::PreviousBundleRestored
        );
        assert_eq!(
            fs::read_to_string(target.join("checkpoint.json")).unwrap(),
            "old\n"
        );
        assert!(!staging.exists());
        assert!(!backup.exists());
        assert!(!transaction_marker_path(&target).exists());
        cleanup(&target);
    }

    #[test]
    fn recovery_keeps_promoted_bundle_and_removes_stale_backup() {
        let target = populated_target("recover-promoted", "old");
        let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
        let staging = transaction.staging.clone();
        let backup = transaction.backup.clone().unwrap();
        fs::write(staging.join("checkpoint.json"), "new\n").unwrap();
        fs::rename(&target, &backup).unwrap();
        fs::rename(&staging, &target).unwrap();
        mem::forget(transaction);

        assert_eq!(
            recover_interrupted_replacement(&target).unwrap(),
            RecoveryOutcome::PromotedBundleKept
        );
        assert_eq!(
            fs::read_to_string(target.join("checkpoint.json")).unwrap(),
            "new\n"
        );
        assert!(!backup.exists());
        assert!(!transaction_marker_path(&target).exists());
        cleanup(&target);
    }

    #[test]
    fn unresolved_transaction_is_detected_before_new_work() {
        let target = populated_target("detect", "old");
        let transaction = RunDirectoryTransaction::replace_verified(&target).unwrap();
        mem::forget(transaction);

        let error = target_is_nonempty_directory(&target).unwrap_err();
        assert!(error.to_string().contains("anthrosim-recover"));
        recover_interrupted_replacement(&target).unwrap();
        cleanup(&target);
    }

    #[test]
    fn ambiguous_legacy_remnants_fail_closed() {
        let target = test_dir("legacy-ambiguous");
        let parent = usable_parent(&target);
        fs::create_dir_all(parent).unwrap();
        let first = parent.join(format!(
            ".{}.anthrosim-stage-one",
            target_base_name(&target)
        ));
        let second = parent.join(format!(
            ".{}.anthrosim-stage-two",
            target_base_name(&target)
        ));
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();

        let error = recover_interrupted_replacement(&target).unwrap_err();
        assert!(error.to_string().contains("ambiguous"));
        assert!(first.exists());
        assert!(second.exists());
        cleanup(&target);
    }

    fn populated_target(label: &str, checkpoint: &str) -> PathBuf {
        let target = test_dir(label);
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("checkpoint.json"), format!("{checkpoint}\n")).unwrap();
        target
    }

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-run-directory-{label}-{}-{id}",
            std::process::id()
        ))
    }

    fn cleanup(target: &Path) {
        let _ = fs::remove_dir_all(target);
        let parent = usable_parent(target);
        let prefix = format!(".{}.anthrosim-", target_base_name(target));
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&prefix))
                {
                    let path = entry.path();
                    if path.is_dir() {
                        let _ = fs::remove_dir_all(path);
                    } else {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    }
}
