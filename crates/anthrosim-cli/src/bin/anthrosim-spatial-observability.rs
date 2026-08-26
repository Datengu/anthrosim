use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../artifact_fs.rs"]
mod artifact_fs;

use anthrosim_core::{
    EvidenceCatalog, LandscapeBundle, LandscapeCheckpoint, LandscapeRecordedRun,
    LandscapeRunManifest, LandscapeSimulation, Population, RunManifest, SimulationCheckpoint,
    SpatialLandscapeCheckpoint, SpatialLandscapeRecordedRun, SpatialLandscapeRunManifest,
    SpatialLandscapeSimulation, SpatialMechanismBinding, SpatialMechanismConfig,
    SpatialObservabilityReport, World, derive_spatial_observability, rng::RngFactory,
    validate_landscape_recorded_run_invariants, validate_spatial_landscape_recorded_run,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-spatial-observability",
    version,
    about = "Regenerate downstream M8.5 spatial observability from preserved run artifacts"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Derive or verify spatial-observability.json for one landscape-bound run bundle.
    Run {
        /// Completed or paused landscape-bound run directory.
        #[arg(long)]
        run_dir: PathBuf,
        /// Validate that an existing report exactly matches regeneration without writing.
        #[arg(long, default_value_t = false)]
        check: bool,
    },
    /// Discover and derive/verify every landscape-bound run beneath an M7 experiment or sweep root.
    Tree {
        /// M7 experiment/sweep root or any directory containing nested run bundles.
        #[arg(long)]
        root: PathBuf,
        /// Validate existing reports without writing any files.
        #[arg(long, default_value_t = false)]
        check: bool,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-spatial-observability: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Run { run_dir, check } => {
            process_run(&run_dir, check)?;
        }
        Command::Tree { root, check } => {
            let run_dirs = discover_spatial_run_dirs(&root)?;
            if run_dirs.is_empty() {
                return Err(format!(
                    "no landscape-bound run bundles found beneath {}",
                    root.display()
                )
                .into());
            }
            for run_dir in &run_dirs {
                process_run(run_dir, check)?;
            }
            println!(
                "{} {} spatial observability report(s) beneath {}",
                if check { "verified" } else { "derived" },
                run_dirs.len(),
                root.display()
            );
        }
    }
    Ok(())
}

fn process_run(run_dir: &Path, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let landscape_path = run_dir.join("landscape.json");
    let world_path = run_dir.join("world.json");
    let checkpoint_path = run_dir.join("checkpoint.json");
    for path in [&landscape_path, &world_path, &checkpoint_path] {
        artifact_fs::require_regular_file(path, "required spatial-observability artifact")?;
    }

    let landscape: LandscapeBundle = read_json(&landscape_path)?;
    let world: World = read_json(&world_path)?;
    let checkpoint: SimulationCheckpoint = read_json(&checkpoint_path)?;
    landscape.validate()?;
    world.validate()?;
    if checkpoint.world_digest64 != world.digest64() {
        return Err(format!(
            "{} world.json digest does not match checkpoint.json",
            run_dir.display()
        )
        .into());
    }
    validate_evidence_artifact(run_dir, &checkpoint)?;
    let spatial_binding = validate_landscape_wrapper(run_dir, &landscape, &world, &checkpoint)?;
    let initial_population = resolve_initial_population(run_dir, &world, &checkpoint)?;

    let report = derive_spatial_observability(
        &landscape,
        &world,
        &initial_population,
        &checkpoint,
        spatial_binding.as_ref(),
    )?;
    let output = run_dir.join("spatial-observability.json");
    if check {
        artifact_fs::require_regular_file(&output, "spatial-observability derived report")?;
        let existing: SpatialObservabilityReport = read_json(&output)?;
        if existing != report {
            return Err(format!(
                "derived report {} does not match deterministic regeneration",
                output.display()
            )
            .into());
        }
        println!("verified {}", output.display());
    } else {
        write_json(&output, &report)?;
        println!("wrote {}", output.display());
    }
    Ok(())
}

fn resolve_initial_population(
    run_dir: &Path,
    world: &World,
    checkpoint: &SimulationCheckpoint,
) -> Result<Population, Box<dyn std::error::Error>> {
    let initial_path = run_dir.join("initial-population.json");
    if artifact_fs::regular_file_exists(&initial_path, "initial population artifact")? {
        let population: Population = read_json(&initial_path)?;
        population.validate(world)?;
        return Ok(population);
    }

    let resume_path = run_dir.join("resume-start-population.json");
    if !artifact_fs::regular_file_exists(&resume_path, "resume population artifact")? {
        return Err(format!(
            "{} has no initial-population.json or resume-start-population.json population provenance",
            run_dir.display()
        )
        .into());
    }

    // A resume-boundary population cannot be replayed from day zero when the
    // checkpoint retains pre-resume events. Reconstruct the original founders
    // from immutable experiment identity and the exact authoritative world.
    let resume_population: Population = read_json(&resume_path)?;
    resume_population.validate(world)?;
    let initial_population = Population::initialize(
        checkpoint.experiment.population,
        world,
        RngFactory::new(checkpoint.experiment.seed),
    )?;
    initial_population.validate(world)?;
    Ok(initial_population)
}

fn validate_evidence_artifact(
    run_dir: &Path,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = run_dir.join("evidence.json");
    if !artifact_fs::regular_file_exists(&path, "spatial-observability evidence artifact")? {
        return Ok(());
    }
    let evidence: EvidenceCatalog = read_json(&path)?;
    evidence.validate()?;
    if checkpoint.experiment.evidence.as_ref() != Some(&evidence) {
        return Err(format!(
            "{} evidence.json does not match the evidence catalogue embedded in checkpoint.json",
            run_dir.display()
        )
        .into());
    }
    Ok(())
}

fn validate_landscape_wrapper(
    run_dir: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    checkpoint: &SimulationCheckpoint,
) -> Result<Option<SpatialMechanismBinding>, Box<dyn std::error::Error>> {
    let checkpoint_path = run_dir.join("landscape-checkpoint.json");
    artifact_fs::require_regular_file(
        &checkpoint_path,
        "spatial-observability landscape checkpoint",
    )?;

    let checkpoint_value: serde_json::Value = read_json(&checkpoint_path)?;
    let manifest_path = run_dir.join("manifest.json");
    let wrapper_manifest_path = run_dir.join("landscape-manifest.json");
    let completed =
        artifact_fs::regular_file_exists(&manifest_path, "spatial-observability core manifest")?;
    let has_wrapper_manifest = artifact_fs::regular_file_exists(
        &wrapper_manifest_path,
        "spatial-observability landscape manifest",
    )?;
    if completed && !has_wrapper_manifest {
        return Err(format!(
            "{} completed landscape-bound run is missing landscape-manifest.json",
            run_dir.display()
        )
        .into());
    }
    if !completed && has_wrapper_manifest {
        return Err(format!(
            "{} has landscape-manifest.json without manifest.json",
            run_dir.display()
        )
        .into());
    }

    if checkpoint_value.get("spatial").is_some() {
        validate_spatial_wrapper(
            run_dir,
            landscape,
            world,
            checkpoint,
            checkpoint_value,
            completed,
        )
    } else {
        validate_plain_landscape_wrapper(
            run_dir,
            landscape,
            world,
            checkpoint,
            checkpoint_value,
            completed,
        )?;
        Ok(None)
    }
}

fn validate_spatial_wrapper(
    run_dir: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    checkpoint: &SimulationCheckpoint,
    checkpoint_value: serde_json::Value,
    completed: bool,
) -> Result<Option<SpatialMechanismBinding>, Box<dyn std::error::Error>> {
    let wrapper_checkpoint: SpatialLandscapeCheckpoint = serde_json::from_value(checkpoint_value)?;
    if wrapper_checkpoint.core_checkpoint != *checkpoint {
        return Err(format!(
            "{} landscape checkpoint wrapper disagrees with checkpoint.json",
            run_dir.display()
        )
        .into());
    }

    let mechanisms_path = run_dir.join("spatial-mechanisms.json");
    artifact_fs::require_regular_file(
        &mechanisms_path,
        "spatial-observability mechanism artifact",
    )?;
    let mechanisms: SpatialMechanismConfig = read_json(&mechanisms_path)?;
    if mechanisms != wrapper_checkpoint.spatial.config {
        return Err(format!(
            "{} spatial-mechanisms.json disagrees with landscape-checkpoint.json",
            run_dir.display()
        )
        .into());
    }

    if completed {
        let core_manifest: RunManifest = read_json(&run_dir.join("manifest.json"))?;
        let wrapper_manifest: SpatialLandscapeRunManifest =
            read_json(&run_dir.join("landscape-manifest.json"))?;
        if wrapper_manifest.core_manifest != core_manifest {
            return Err(format!(
                "{} landscape manifest wrapper disagrees with manifest.json",
                run_dir.display()
            )
            .into());
        }
        let run = SpatialLandscapeRecordedRun {
            manifest: wrapper_manifest,
            checkpoint: wrapper_checkpoint.clone(),
        };
        validate_spatial_landscape_recorded_run(&run, landscape)?;
    } else {
        let simulation = SpatialLandscapeSimulation::from_checkpoint(
            wrapper_checkpoint.clone(),
            landscape.clone(),
        )?;
        if simulation.world() != world {
            return Err(format!(
                "{} world.json does not match deterministic transformed-world reconstruction",
                run_dir.display()
            )
            .into());
        }
    }

    if wrapper_checkpoint.spatial.transformed_world_digest64 != world.digest64() {
        return Err(format!(
            "{} world.json does not match the transformed world bound by landscape-checkpoint.json",
            run_dir.display()
        )
        .into());
    }
    Ok(Some(wrapper_checkpoint.spatial))
}

fn validate_plain_landscape_wrapper(
    run_dir: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    checkpoint: &SimulationCheckpoint,
    checkpoint_value: serde_json::Value,
    completed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mechanisms_path = run_dir.join("spatial-mechanisms.json");
    if artifact_fs::regular_file_exists(
        &mechanisms_path,
        "spatial-observability mechanism artifact",
    )? {
        return Err(format!(
            "{} has spatial-mechanisms.json but its landscape checkpoint is not transformed",
            run_dir.display()
        )
        .into());
    }

    let wrapper_checkpoint: LandscapeCheckpoint = serde_json::from_value(checkpoint_value)?;
    if wrapper_checkpoint.core_checkpoint != *checkpoint {
        return Err(format!(
            "{} landscape checkpoint wrapper disagrees with checkpoint.json",
            run_dir.display()
        )
        .into());
    }
    wrapper_checkpoint.landscape.validate_bundle(landscape)?;

    if completed {
        let core_manifest: RunManifest = read_json(&run_dir.join("manifest.json"))?;
        let wrapper_manifest: LandscapeRunManifest =
            read_json(&run_dir.join("landscape-manifest.json"))?;
        if wrapper_manifest.core_manifest != core_manifest {
            return Err(format!(
                "{} landscape manifest wrapper disagrees with manifest.json",
                run_dir.display()
            )
            .into());
        }
        let run = LandscapeRecordedRun {
            manifest: wrapper_manifest,
            checkpoint: wrapper_checkpoint,
        };
        validate_landscape_recorded_run_invariants(&run)?;
    } else {
        let simulation =
            LandscapeSimulation::from_checkpoint(wrapper_checkpoint, landscape.clone())?;
        if simulation.world() != world {
            return Err(format!(
                "{} world.json does not match deterministic landscape-bound reconstruction",
                run_dir.display()
            )
            .into());
        }
    }
    Ok(())
}

fn discover_spatial_run_dirs(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        if !directory.is_dir() {
            continue;
        }
        let landscape_path = directory.join("landscape.json");
        let checkpoint_path = directory.join("checkpoint.json");
        let world_path = directory.join("world.json");
        let landscape_checkpoint_path = directory.join("landscape-checkpoint.json");
        let has_landscape = artifact_fs::regular_file_exists(
            &landscape_path,
            "spatial-observability discovery landscape",
        )?;
        let has_checkpoint = artifact_fs::regular_file_exists(
            &checkpoint_path,
            "spatial-observability discovery checkpoint",
        )?;
        let has_world =
            artifact_fs::regular_file_exists(&world_path, "spatial-observability discovery world")?;
        let has_landscape_checkpoint = artifact_fs::regular_file_exists(
            &landscape_checkpoint_path,
            "spatial-observability discovery landscape checkpoint",
        )?;
        if has_landscape && has_checkpoint && has_world && has_landscape_checkpoint {
            found.push(directory);
            continue;
        }
        let mut children = fs::read_dir(&directory)?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        children.sort();
        children.reverse();
        pending.extend(children);
    }
    found.sort();
    Ok(found)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let content = artifact_fs::read_to_string(path, "spatial observability source artifact")?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(value)?;
    let payload = format!("{json}\n");
    artifact_fs::atomic_write(path, payload.as_bytes(), "spatial observability output")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn report_writer_rejects_symlink_without_touching_target() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "anthrosim-spatial-observability-symlink-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let outside = root.with_extension("outside-report.json");
        fs::write(&outside, "outside sentinel\n").unwrap();
        let output = root.join("spatial-observability.json");
        symlink(&outside, &output).unwrap();

        let error = write_json(&output, &serde_json::json!({"derived": true}))
            .unwrap_err()
            .to_string();
        assert!(error.contains("symbolic link"));
        assert_eq!(fs::read_to_string(&outside).unwrap(), "outside sentinel\n");

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }

    #[cfg(unix)]
    #[test]
    fn broken_wrapper_symlink_is_rejected_instead_of_treated_as_missing() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "anthrosim-spatial-observability-broken-wrapper-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let missing = root.with_extension("missing-landscape-manifest.json");
        symlink(&missing, root.join("landscape-manifest.json")).unwrap();

        let error = artifact_fs::regular_file_exists(
            &root.join("landscape-manifest.json"),
            "spatial-observability landscape manifest",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("symbolic link"));

        let _ = fs::remove_dir_all(root);
    }
}
