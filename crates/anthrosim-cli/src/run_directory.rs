use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anthrosim_core::{Population, SimulationCheckpoint, World, rng::RngFactory};
use serde::de::DeserializeOwned;

static NEXT_TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitMode {
    Fresh,
    ReplaceVerified,
}

/// Builds a run bundle in a clean sibling directory and promotes it only after
/// the caller has finished writing and validating the complete artifact set.
#[derive(Debug)]
pub(crate) struct RunDirectoryTransaction {
    target: PathBuf,
    staging: PathBuf,
    mode: CommitMode,
    committed: bool,
}

impl RunDirectoryTransaction {
    /// Start a new run/checkpoint bundle. Existing non-empty targets are never
    /// overwritten implicitly.
    pub(crate) fn fresh(target: &Path) -> io::Result<Self> {
        require_fresh_target(target)?;
        Self::start(target, CommitMode::Fresh)
    }

    /// Start a replacement after the caller has verified that the existing
    /// target is the source bundle for an explicit in-place resume.
    pub(crate) fn replace_verified(target: &Path) -> io::Result<Self> {
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
        Ok(Self {
            target: target.to_owned(),
            staging,
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
        require_nonempty_directory(&self.target)?;
        let backup = unique_sibling_path(&self.target, "backup");
        fs::rename(&self.target, &backup)?;
        match fs::rename(&self.staging, &self.target) {
            Ok(()) => {
                if let Err(error) = fs::remove_dir_all(&backup) {
                    return Err(io::Error::new(
                        error.kind(),
                        format!(
                            "resumed run was committed to {}, but the verified previous bundle could not be removed from {}: {error}",
                            self.target.display(),
                            backup.display()
                        ),
                    ));
                }
                Ok(())
            }
            Err(error) => {
                let restore = fs::rename(&backup, &self.target);
                match restore {
                    Ok(()) => Err(io::Error::new(
                        error.kind(),
                        format!(
                            "could not promote staged resumed run into {}: {error}; previous verified bundle was restored",
                            self.target.display()
                        ),
                    )),
                    Err(restore_error) => Err(io::Error::other(format!(
                        "could not promote staged resumed run into {}: {error}; also failed to restore previous verified bundle from {}: {restore_error}",
                        self.target.display(),
                        backup.display()
                    ))),
                }
            }
        }
    }
}

impl Drop for RunDirectoryTransaction {
    fn drop(&mut self) {
        if !self.committed && self.staging.exists() {
            let _ = fs::remove_dir_all(&self.staging);
        }
    }
}

pub(crate) fn target_is_nonempty_directory(path: &Path) -> io::Result<bool> {
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

pub(crate) fn same_existing_path(left: &Path, right: &Path) -> io::Result<bool> {
    Ok(fs::canonicalize(left)? == fs::canonicalize(right)?)
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

fn write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(value).map_err(invalid_data)?;
    fs::write(path, format!("{json}\n"))
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn require_fresh_target(target: &Path) -> io::Result<()> {
    if target_is_nonempty_directory(target)? {
        return Err(nonempty_target_error(target));
    }
    Ok(())
}

fn require_nonempty_directory(target: &Path) -> io::Result<()> {
    if !target_is_nonempty_directory(target)? {
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

fn unique_sibling_path(target: &Path, role: &str) -> PathBuf {
    let parent = usable_parent(target);
    let base = target
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("run");
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

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
        cleanup(&target);
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
        let prefix = format!(
            ".{}.anthrosim-",
            target
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("run")
        );
        if let Ok(entries) = fs::read_dir(parent) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with(&prefix))
                {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
    }
}
