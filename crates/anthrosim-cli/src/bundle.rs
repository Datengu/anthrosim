use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

#[path = "artifact_fs.rs"]
pub(crate) mod artifact_fs;

use anthrosim_core::{
    DemographyObservabilityReport, EventLog, EvidenceCatalog, LandscapeBundle, LandscapeCheckpoint,
    LandscapeRecordedRun, LandscapeRunManifest, MetricSeries, Population, PopulationInitialization,
    RecordedRun, RunManifest, SimulationCheckpoint, SpatialLandscapeCheckpoint,
    SpatialLandscapeRecordedRun, SpatialLandscapeRunManifest, SpatialMechanismBinding,
    SpatialMechanismConfig, SpatialObservabilityReport, TemporaryMobilityObservabilityReport,
    World, derive_demography_observability, derive_spatial_observability,
    derive_temporary_mobility_observability, rng::RngFactory,
    validate_landscape_recorded_run_invariants, validate_recorded_run_invariants,
    validate_spatial_landscape_recorded_run,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;

pub(crate) const REQUIRED_JSON: &[&str] = &[
    "checkpoint.json",
    "events.json",
    "manifest.json",
    "metrics.json",
    "world.json",
];

pub(crate) const POPULATION_JSON: &[&str] =
    &["initial-population.json", "resume-start-population.json"];

pub(crate) const OPTIONAL_JSON: &[&str] = &[
    "completion.json",
    "demography-observability.json",
    "evidence.json",
    "landscape-checkpoint.json",
    "landscape-manifest.json",
    "landscape.json",
    "spatial-mechanisms.json",
    "spatial-observability.json",
    "temporary-observability.json",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BundleValidationError(String);

impl fmt::Display for BundleValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for BundleValidationError {}

pub(crate) fn validated_bundle_files(
    run_dir: &Path,
) -> Result<Vec<(String, PathBuf)>, BundleValidationError> {
    match fs::symlink_metadata(run_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(invalid(format!(
                "run directory may not be a symbolic link: {}",
                run_dir.display()
            )));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(invalid(format!(
                "run directory does not exist or is not a directory: {}",
                run_dir.display()
            )));
        }
        Ok(_) => {}
        Err(error) => {
            return Err(invalid(format!(
                "unable to inspect run directory {}: {error}",
                run_dir.display()
            )));
        }
    }

    let mut names = Vec::new();
    for name in REQUIRED_JSON {
        require_regular_file(run_dir, name)?;
        names.push((*name).to_owned());
    }

    let mut population_names = Vec::new();
    for name in POPULATION_JSON {
        let path = run_dir.join(name);
        if artifact_fs::regular_file_exists(&path, "bundle artifact")
            .map_err(|error| invalid(error.to_string()))?
        {
            names.push((*name).to_owned());
            population_names.push(*name);
        }
    }
    if population_names.is_empty() {
        return Err(invalid(
            "completed run bundle must contain initial-population.json or resume-start-population.json",
        ));
    }

    for name in OPTIONAL_JSON {
        let path = run_dir.join(name);
        if artifact_fs::regular_file_exists(&path, "bundle artifact")
            .map_err(|error| invalid(error.to_string()))?
        {
            names.push((*name).to_owned());
        }
    }

    let has_landscape = names.iter().any(|name| name == "landscape.json");
    let has_landscape_manifest = names.iter().any(|name| name == "landscape-manifest.json");
    let has_landscape_checkpoint = names.iter().any(|name| name == "landscape-checkpoint.json");
    let has_spatial_mechanisms = names.iter().any(|name| name == "spatial-mechanisms.json");
    let has_spatial_observability = names
        .iter()
        .any(|name| name == "spatial-observability.json");

    if has_landscape && (!has_landscape_manifest || !has_landscape_checkpoint) {
        return Err(invalid(
            "completed landscape-bound run must contain landscape-manifest.json and landscape-checkpoint.json",
        ));
    }
    if !has_landscape
        && (has_landscape_manifest
            || has_landscape_checkpoint
            || has_spatial_mechanisms
            || has_spatial_observability)
    {
        return Err(invalid(
            "landscape/spatial artifacts require landscape.json in the same completed run bundle",
        ));
    }

    validate_semantics(run_dir, &population_names, has_landscape)?;

    names.sort_unstable();
    names.dedup();
    Ok(names
        .into_iter()
        .map(|name| {
            let path = run_dir.join(&name);
            (name, path)
        })
        .collect())
}

fn validate_semantics(
    run_dir: &Path,
    population_names: &[&str],
    has_landscape: bool,
) -> Result<(), BundleValidationError> {
    let manifest: RunManifest = read_json(&run_dir.join("manifest.json"))?;
    let checkpoint: SimulationCheckpoint = read_json(&run_dir.join("checkpoint.json"))?;
    let world: World = read_json(&run_dir.join("world.json"))?;
    let events: EventLog = read_json(&run_dir.join("events.json"))?;
    let metrics: MetricSeries = read_json(&run_dir.join("metrics.json"))?;

    world
        .validate()
        .map_err(|error| invalid(format!("world.json failed validation: {error}")))?;
    if checkpoint.world_digest64 != world.digest64() {
        return Err(invalid(format!(
            "world.json digest does not match checkpoint.json: checkpoint {}, world {}",
            checkpoint.world_digest64,
            world.digest64()
        )));
    }
    if checkpoint.events != events {
        return Err(invalid("events.json does not match checkpoint.json"));
    }
    if checkpoint.metrics != metrics {
        return Err(invalid("metrics.json does not match checkpoint.json"));
    }

    let mut initial_population = None;
    for name in population_names {
        let population: Population = read_json(&run_dir.join(name))?;
        population.validate(&world).map_err(|error| {
            invalid(format!(
                "{name} failed validation against world.json: {error}"
            ))
        })?;
        if *name == "initial-population.json" {
            initial_population = Some(population);
        }
    }
    let reconstructed_initial_population =
        reconstruct_initial_population(run_dir, &checkpoint, &world)?;
    if let Some(recorded_initial_population) = initial_population.as_ref() {
        if recorded_initial_population != &reconstructed_initial_population {
            return Err(invalid(
                "initial-population.json does not match deterministic founder reconstruction",
            ));
        }
    } else {
        initial_population = Some(reconstructed_initial_population);
    }

    let (landscape, spatial_binding) = if has_landscape {
        let (landscape, spatial) =
            validate_landscape_artifacts(run_dir, &manifest, &checkpoint, &world)?;
        (Some(landscape), spatial)
    } else {
        validate_recorded_run_invariants(&RecordedRun {
            manifest: manifest.clone(),
            checkpoint: checkpoint.clone(),
        })
        .map_err(|error| invalid(format!("core run invariants failed: {error}")))?;
        (None, None)
    };

    validate_optional_evidence(run_dir, &checkpoint)?;
    validate_optional_completion(run_dir, &checkpoint)?;

    let initial_population = initial_population
        .as_ref()
        .ok_or_else(|| invalid("run bundle has no resolvable original founder population"))?;
    validate_optional_demography_observability(run_dir, initial_population, &checkpoint)?;
    validate_optional_temporary_observability(run_dir, &world, initial_population, &checkpoint)?;

    if let Some(landscape) = landscape.as_ref() {
        validate_optional_spatial_observability(
            run_dir,
            landscape,
            &world,
            initial_population,
            &checkpoint,
            spatial_binding.as_ref(),
        )?;
    }

    Ok(())
}

fn reconstruct_initial_population(
    run_dir: &Path,
    checkpoint: &SimulationCheckpoint,
    world: &World,
) -> Result<Population, BundleValidationError> {
    let config = checkpoint.experiment.population;
    // Spatial split-realization runs initialize synthetic founders from the bound
    // population seed, not the dynamic process seed. The spatial artifacts are fully
    // cross-validated later in this function, so reading the declared mechanisms here
    // only selects the deterministic reconstruction seed; it does not weaken binding.
    let population_seed = if run_dir.join("spatial-mechanisms.json").is_file() {
        let mechanisms: SpatialMechanismConfig =
            read_json(&run_dir.join("spatial-mechanisms.json"))?;
        mechanisms
            .run_realization
            .map_or(checkpoint.experiment.seed, |realization| {
                realization.population_seed
            })
    } else {
        checkpoint.experiment.seed
    };
    let population = match config.initialization {
        PopulationInitialization::SyntheticValidationV1 => Population::initialize(
            config,
            world,
            RngFactory::new(population_seed),
        ),
        PopulationInitialization::DeclaredFounderStateV1 => {
            let definition = checkpoint
                .experiment
                .founder_population
                .as_ref()
                .ok_or_else(|| {
                    invalid(
                        "declared founder initialization is missing founderPopulation in checkpoint experiment",
                    )
                })?;
            Population::initialize_declared_founder_state_v1(config, definition, world)
        }
    }
    .map_err(|error| {
        invalid(format!(
            "unable to reconstruct original founder population: {error}"
        ))
    })?;
    population.validate(world).map_err(|error| {
        invalid(format!(
            "reconstructed original founder population failed validation: {error}"
        ))
    })?;
    Ok(population)
}

fn validate_landscape_artifacts(
    run_dir: &Path,
    manifest: &RunManifest,
    checkpoint: &SimulationCheckpoint,
    world: &World,
) -> Result<(LandscapeBundle, Option<SpatialMechanismBinding>), BundleValidationError> {
    let landscape: LandscapeBundle = read_json(&run_dir.join("landscape.json"))?;
    landscape
        .validate()
        .map_err(|error| invalid(format!("landscape.json failed validation: {error}")))?;

    let manifest_value: serde_json::Value = read_json(&run_dir.join("landscape-manifest.json"))?;
    let checkpoint_value: serde_json::Value =
        read_json(&run_dir.join("landscape-checkpoint.json"))?;
    let manifest_is_spatial = manifest_value.get("spatial").is_some();
    let checkpoint_is_spatial = checkpoint_value.get("spatial").is_some();
    if manifest_is_spatial != checkpoint_is_spatial {
        return Err(invalid(
            "landscape manifest/checkpoint disagree about spatial transformation mode",
        ));
    }

    if manifest_is_spatial {
        let wrapper_manifest: SpatialLandscapeRunManifest =
            from_value("landscape-manifest.json", manifest_value)?;
        let wrapper_checkpoint: SpatialLandscapeCheckpoint =
            from_value("landscape-checkpoint.json", checkpoint_value)?;
        if wrapper_manifest.core_manifest != *manifest {
            return Err(invalid(
                "manifest.json does not match the core manifest embedded in landscape-manifest.json",
            ));
        }
        if wrapper_checkpoint.core_checkpoint != *checkpoint {
            return Err(invalid(
                "checkpoint.json does not match the core checkpoint embedded in landscape-checkpoint.json",
            ));
        }

        let mechanisms_path = run_dir.join("spatial-mechanisms.json");
        if !mechanisms_path.is_file() {
            return Err(invalid(
                "transformed spatial run is missing spatial-mechanisms.json",
            ));
        }
        let mechanisms: SpatialMechanismConfig = read_json(&mechanisms_path)?;
        if mechanisms != wrapper_checkpoint.spatial.config {
            return Err(invalid(
                "spatial-mechanisms.json does not match landscape-checkpoint.json",
            ));
        }

        let run = SpatialLandscapeRecordedRun {
            manifest: wrapper_manifest,
            checkpoint: wrapper_checkpoint,
        };
        validate_spatial_landscape_recorded_run(&run, &landscape)
            .map_err(|error| invalid(format!("spatial run invariants failed: {error}")))?;
        if run.checkpoint.spatial.transformed_world_digest64 != world.digest64() {
            return Err(invalid(
                "world.json does not match the transformed world bound by landscape-checkpoint.json",
            ));
        }
        Ok((landscape, Some(run.checkpoint.spatial)))
    } else {
        if run_dir.join("spatial-mechanisms.json").is_file() {
            return Err(invalid(
                "spatial-mechanisms.json is present but landscape wrappers do not describe a transformed spatial run",
            ));
        }
        let wrapper_manifest: LandscapeRunManifest =
            from_value("landscape-manifest.json", manifest_value)?;
        let wrapper_checkpoint: LandscapeCheckpoint =
            from_value("landscape-checkpoint.json", checkpoint_value)?;
        if wrapper_manifest.core_manifest != *manifest {
            return Err(invalid(
                "manifest.json does not match the core manifest embedded in landscape-manifest.json",
            ));
        }
        if wrapper_checkpoint.core_checkpoint != *checkpoint {
            return Err(invalid(
                "checkpoint.json does not match the core checkpoint embedded in landscape-checkpoint.json",
            ));
        }
        let run = LandscapeRecordedRun {
            manifest: wrapper_manifest,
            checkpoint: wrapper_checkpoint,
        };
        validate_landscape_recorded_run_invariants(&run)
            .map_err(|error| invalid(format!("landscape run invariants failed: {error}")))?;
        Ok((landscape, None))
    }
}

fn validate_optional_evidence(
    run_dir: &Path,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), BundleValidationError> {
    let path = run_dir.join("evidence.json");
    if !path.is_file() {
        return Ok(());
    }
    let evidence: EvidenceCatalog = read_json(&path)?;
    evidence
        .validate()
        .map_err(|error| invalid(format!("evidence.json failed validation: {error}")))?;
    if checkpoint.experiment.evidence.as_ref() != Some(&evidence) {
        return Err(invalid(
            "evidence.json does not match the evidence catalogue embedded in checkpoint.json",
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionMarker {
    schema_version: u32,
    seed: u64,
    status: String,
    manifest: String,
}

fn validate_optional_completion(
    run_dir: &Path,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), BundleValidationError> {
    let path = run_dir.join("completion.json");
    if !path.is_file() {
        return Ok(());
    }
    let completion: CompletionMarker = read_json(&path)?;
    if completion.schema_version != 1
        || completion.seed != checkpoint.experiment.seed
        || completion.status != "completed"
        || completion.manifest != "manifest.json"
    {
        return Err(invalid(
            "completion.json does not describe this completed run bundle",
        ));
    }
    Ok(())
}

fn validate_optional_demography_observability(
    run_dir: &Path,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), BundleValidationError> {
    let path = run_dir.join("demography-observability.json");
    if !path.is_file() {
        return Ok(());
    }
    let report: DemographyObservabilityReport = read_json(&path)?;
    let regenerated =
        derive_demography_observability(initial_population, checkpoint).map_err(|error| {
            invalid(format!(
                "demography-observability.json could not be regenerated: {error}"
            ))
        })?;
    if regenerated != report {
        return Err(invalid(
            "demography-observability.json does not match deterministic regeneration",
        ));
    }
    Ok(())
}

fn validate_optional_temporary_observability(
    run_dir: &Path,
    world: &World,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), BundleValidationError> {
    let path = run_dir.join("temporary-observability.json");
    if !path.is_file() {
        return Ok(());
    }
    let report: TemporaryMobilityObservabilityReport = read_json(&path)?;
    let program = checkpoint
        .temporary_mobility
        .program()
        .ok_or_else(|| invalid(
            "temporary-observability.json is present but checkpoint.json has no configured temporary-mobility program",
        ))?;
    if report.schema_version != TemporaryMobilityObservabilityReport::CURRENT_SCHEMA_VERSION
        || report.source.model_version != checkpoint.model_version
        || report.source.model_semantics_id != checkpoint.model_semantics_id
        || report.source.git_commit != checkpoint.git_commit
        || report.source.seed != checkpoint.experiment.seed
        || report.source.end_day != checkpoint.time.days()
        || report.source.run_state_digest64 != checkpoint.state_digest64
        || report.source.world_digest64 != world.digest64()
        || report.source.temporary_mobility_program_identity != program.identity()
        || report.source.region_id != program.region.region_id
        || report.source.region_identity != program.region.identity()
    {
        return Err(invalid(
            "temporary-observability.json provenance does not match the run bundle",
        ));
    }

    let regenerated =
        derive_temporary_mobility_observability(world, initial_population, checkpoint).map_err(
            |error| {
                invalid(format!(
                    "temporary-observability.json could not be regenerated: {error}"
                ))
            },
        )?;
    if regenerated != report {
        return Err(invalid(
            "temporary-observability.json does not match deterministic regeneration",
        ));
    }
    Ok(())
}

fn validate_optional_spatial_observability(
    run_dir: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    initial_population: &Population,
    checkpoint: &SimulationCheckpoint,
    spatial: Option<&SpatialMechanismBinding>,
) -> Result<(), BundleValidationError> {
    let path = run_dir.join("spatial-observability.json");
    if !path.is_file() {
        return Ok(());
    }
    let report: SpatialObservabilityReport = read_json(&path)?;
    if report.schema_version != SpatialObservabilityReport::CURRENT_SCHEMA_VERSION
        || report.source.model_version != checkpoint.model_version
        || report.source.model_semantics_id != checkpoint.model_semantics_id
        || report.source.git_commit != checkpoint.git_commit
        || report.source.seed != checkpoint.experiment.seed
        || report.source.end_day != checkpoint.time.days()
        || report.source.run_state_digest64 != checkpoint.state_digest64
        || report.source.landscape_identity != landscape.identity()
        || report.source.landscape_digest64 != landscape.digest64()
        || report.source.transformed_world_digest64 != world.digest64()
        || report.source.spatial_model_semantics_id
            != spatial.map(|binding| binding.spatial_model_semantics_id.clone())
        || report.source.spatial_config_identity
            != spatial.map(|binding| binding.config_identity.clone())
    {
        return Err(invalid(
            "spatial-observability.json provenance does not match the run bundle",
        ));
    }

    let regenerated =
        derive_spatial_observability(landscape, world, initial_population, checkpoint, spatial)
            .map_err(|error| {
                invalid(format!(
                    "spatial-observability.json could not be regenerated: {error}"
                ))
            })?;
    if regenerated != report {
        return Err(invalid(
            "spatial-observability.json does not match deterministic regeneration",
        ));
    }
    Ok(())
}

fn require_regular_file(run_dir: &Path, name: &str) -> Result<(), BundleValidationError> {
    let path = run_dir.join(name);
    artifact_fs::require_regular_file(&path, "required bundle artifact")
        .map_err(|error| invalid(error.to_string()))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, BundleValidationError> {
    let content = artifact_fs::read_to_string(path, "bundle artifact")
        .map_err(|error| invalid(format!("unable to read {}: {error}", path.display())))?;
    serde_json::from_str(&content).map_err(|error| {
        invalid(format!(
            "invalid AnthroSim JSON in {}: {error}",
            path.display()
        ))
    })
}

fn from_value<T: DeserializeOwned>(
    artifact: &str,
    value: serde_json::Value,
) -> Result<T, BundleValidationError> {
    serde_json::from_value(value)
        .map_err(|error| invalid(format!("invalid AnthroSim JSON in {artifact}: {error}")))
}

fn invalid(message: impl Into<String>) -> BundleValidationError {
    BundleValidationError(message.into())
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    use anthrosim_core::{
        ExperimentConfig, FocalRegion, FocalRegionSource, PopulationConfig, Simulation,
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming, WorldConfig, ids::CellId,
    };

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn real_completed_core_bundle_is_accepted() {
        let root = test_dir("valid");
        write_real_completed_bundle(&root);

        let files = validated_bundle_files(&root).unwrap();
        assert!(files.iter().any(|(name, _)| name == "manifest.json"));
        assert!(files.iter().any(|(name, _)| name == "checkpoint.json"));
        cleanup(&root);
    }

    #[cfg(unix)]
    #[test]
    fn required_artifact_symlink_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = test_dir("required-symlink");
        write_real_completed_bundle(&root);
        let outside = root.with_extension("outside-checkpoint.json");
        fs::rename(root.join("checkpoint.json"), &outside).unwrap();
        symlink(&outside, root.join("checkpoint.json")).unwrap();

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("symbolic link"));
        cleanup(&root);
        let _ = fs::remove_file(outside);
    }

    #[cfg(unix)]
    #[test]
    fn optional_artifact_symlink_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = test_dir("optional-symlink");
        write_real_completed_bundle(&root);
        let outside = root.with_extension("outside-evidence.json");
        fs::write(&outside, "{}\n").unwrap();
        symlink(&outside, root.join("evidence.json")).unwrap();

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("symbolic link"));
        cleanup(&root);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn demography_observability_is_validated_and_packed() {
        let root = test_dir("demography-observability-valid");
        write_real_completed_bundle(&root);
        let checkpoint: SimulationCheckpoint = read_json(&root.join("checkpoint.json")).unwrap();
        let initial_population: Population =
            read_json(&root.join("initial-population.json")).unwrap();
        let report = derive_demography_observability(&initial_population, &checkpoint).unwrap();
        write_json(&root.join("demography-observability.json"), &report);

        let files = validated_bundle_files(&root).unwrap();
        assert!(
            files
                .iter()
                .any(|(name, _)| name == "demography-observability.json")
        );
        cleanup(&root);
    }

    #[test]
    fn tampered_demography_observability_is_rejected() {
        let root = test_dir("demography-observability-tampered");
        write_real_completed_bundle(&root);
        let checkpoint: SimulationCheckpoint = read_json(&root.join("checkpoint.json")).unwrap();
        let initial_population: Population =
            read_json(&root.join("initial-population.json")).unwrap();
        let mut report = derive_demography_observability(&initial_population, &checkpoint).unwrap();
        report.summary.age_schedule_eligible =
            report.summary.age_schedule_eligible.saturating_add(1);
        write_json(&root.join("demography-observability.json"), &report);

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("does not match deterministic regeneration"));
        cleanup(&root);
    }

    #[test]
    fn temporary_observability_is_validated_and_packed() {
        let root = test_dir("temporary-observability-valid");
        write_real_temporary_bundle(&root);

        let files = validated_bundle_files(&root).unwrap();
        assert!(
            files
                .iter()
                .any(|(name, _)| name == "temporary-observability.json")
        );
        cleanup(&root);
    }

    #[test]
    fn tampered_temporary_observability_is_rejected() {
        let root = test_dir("temporary-observability-tampered");
        write_real_temporary_bundle(&root);
        let path = root.join("temporary-observability.json");
        let mut report: TemporaryMobilityObservabilityReport = read_json(&path).unwrap();
        report.summary.trigger_outcomes += 1;
        write_json(&path, &report);

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("does not match deterministic regeneration"));
        cleanup(&root);
    }

    #[test]
    fn json_placeholders_are_not_accepted_as_anthrosim_artifacts() {
        let root = test_dir("placeholders");
        fs::create_dir_all(&root).unwrap();
        for name in REQUIRED_JSON {
            fs::write(root.join(name), "{}\n").unwrap();
        }
        fs::write(root.join("initial-population.json"), "{}\n").unwrap();

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("manifest.json"));
        cleanup(&root);
    }

    #[test]
    fn typed_but_mismatched_metrics_are_rejected() {
        let root = test_dir("mismatched-metrics");
        write_real_completed_bundle(&root);
        write_json(&root.join("metrics.json"), &MetricSeries::annual());

        let error = validated_bundle_files(&root).unwrap_err().to_string();
        assert!(error.contains("metrics.json does not match checkpoint.json"));
        cleanup(&root);
    }

    #[test]
    fn founder_population_reconstructs_from_experiment_identity() {
        let config = ExperimentConfig::new(72, 0)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(8).with_target_household_size(2));
        let simulation = Simulation::new(config).unwrap();
        let world = simulation.world().clone();
        let expected = simulation.population().clone();
        let checkpoint = simulation.run_recorded().unwrap().checkpoint;

        let reconstructed = reconstruct_initial_population(&checkpoint, &world).unwrap();
        assert_eq!(reconstructed, expected);
    }

    fn write_real_temporary_bundle(root: &Path) {
        let region = FocalRegion::new(
            "bundle-temporary-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(16)],
        )
        .unwrap();
        let schedule = TemporaryMobilitySchedule::new(
            "bundle-temporary-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![10],
            5,
        )
        .unwrap();
        let temporary = TemporaryMobilityConfig::new(
            region,
            schedule,
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .unwrap();
        let config = ExperimentConfig::new(79, 1)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(8)
                    .with_target_household_size(2)
                    .with_max_person_records(64),
            )
            .with_temporary_mobility(temporary);
        let simulation = Simulation::new(config).unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();
        let report = derive_temporary_mobility_observability(
            &world,
            &initial_population,
            &recorded.checkpoint,
        )
        .unwrap();

        fs::create_dir_all(root).unwrap();
        write_json(&root.join("world.json"), &world);
        write_json(&root.join("initial-population.json"), &initial_population);
        write_json(&root.join("manifest.json"), &recorded.manifest);
        write_json(&root.join("events.json"), recorded.events());
        write_json(&root.join("metrics.json"), recorded.metrics());
        write_json(&root.join("checkpoint.json"), &recorded.checkpoint);
        write_json(&root.join("temporary-observability.json"), &report);
    }

    fn write_real_completed_bundle(root: &Path) {
        let config = ExperimentConfig::new(71, 0)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(8)
                    .with_target_household_size(2)
                    .with_max_person_records(64),
            );
        let simulation = Simulation::new(config).unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();

        fs::create_dir_all(root).unwrap();
        write_json(&root.join("world.json"), &world);
        write_json(&root.join("initial-population.json"), &initial_population);
        write_json(&root.join("manifest.json"), &recorded.manifest);
        write_json(&root.join("events.json"), recorded.events());
        write_json(&root.join("metrics.json"), recorded.metrics());
        write_json(&root.join("checkpoint.json"), &recorded.checkpoint);
    }

    fn write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) {
        let json = serde_json::to_string_pretty(value).unwrap();
        fs::write(path, format!("{json}\n")).unwrap();
    }

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-bundle-{label}-{}-{id}",
            std::process::id()
        ))
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
    }
}
