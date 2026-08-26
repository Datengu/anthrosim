use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[path = "../artifact_fs.rs"]
mod artifact_fs;

use anthrosim_core::{
    EventLog, Population, SimulationCheckpoint, TemporaryMobilityObservabilityReport, World,
    derive_temporary_mobility_observability, rng::RngFactory,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-temporary-observability",
    version,
    about = "Regenerate downstream M9 temporary-mobility observability from preserved run artifacts"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Derive or verify temporary-observability.json for one M9 run bundle.
    Run {
        /// Completed or paused run directory containing an M9 temporary-mobility program.
        #[arg(long)]
        run_dir: PathBuf,
        /// Validate that an existing report exactly matches regeneration without writing.
        #[arg(long, default_value_t = false)]
        check: bool,
    },
    /// Discover and derive/verify every M9 run beneath an experiment or sweep root.
    Tree {
        /// Experiment/sweep root or any directory containing nested run bundles.
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
            eprintln!("anthrosim-temporary-observability: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Run { run_dir, check } => process_run(&run_dir, check)?,
        Command::Tree { root, check } => {
            let run_dirs = discover_temporary_run_dirs(&root)?;
            if run_dirs.is_empty() {
                return Err(format!(
                    "no M9 temporary-mobility run bundles found beneath {}",
                    root.display()
                )
                .into());
            }
            for run_dir in &run_dirs {
                process_run(run_dir, check)?;
            }
            println!(
                "{} {} temporary-mobility observability report(s) beneath {}",
                if check { "verified" } else { "derived" },
                run_dirs.len(),
                root.display()
            );
        }
    }
    Ok(())
}

fn process_run(run_dir: &Path, check: bool) -> Result<(), Box<dyn std::error::Error>> {
    let world_path = run_dir.join("world.json");
    let checkpoint_path = run_dir.join("checkpoint.json");
    for path in [&world_path, &checkpoint_path] {
        artifact_fs::require_regular_file(path, "required temporary-observability artifact")?;
    }

    let world: World = read_json(&world_path)?;
    let checkpoint: SimulationCheckpoint = read_json(&checkpoint_path)?;
    world.validate()?;
    if checkpoint.world_digest64 != world.digest64() {
        return Err(format!(
            "{} world.json digest does not match checkpoint.json",
            run_dir.display()
        )
        .into());
    }
    if checkpoint.temporary_mobility.program().is_none() {
        return Err(format!(
            "{} does not contain a configured M9 temporary-mobility program",
            run_dir.display()
        )
        .into());
    }

    let events_path = run_dir.join("events.json");
    if artifact_fs::regular_file_exists(&events_path, "temporary-observability events artifact")? {
        let events: EventLog = read_json(&events_path)?;
        if events != checkpoint.events {
            return Err(format!(
                "{} events.json does not match checkpoint.json",
                run_dir.display()
            )
            .into());
        }
    }

    let initial_population = resolve_initial_population(run_dir, &world, &checkpoint)?;
    let report = derive_temporary_mobility_observability(&world, &initial_population, &checkpoint)?;
    let output = run_dir.join("temporary-observability.json");
    if check {
        artifact_fs::require_regular_file(&output, "temporary-observability derived report")?;
        let existing: TemporaryMobilityObservabilityReport = read_json(&output)?;
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

    // The resume-start population is not the day-zero state when checkpoint events retain the
    // pre-resume history. Validate that artifact, then reconstruct founders from immutable run
    // identity and the exact authoritative world.
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

fn discover_temporary_run_dirs(root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        if !directory.is_dir() {
            continue;
        }
        let checkpoint_path = directory.join("checkpoint.json");
        let world_path = directory.join("world.json");
        let has_checkpoint = artifact_fs::regular_file_exists(
            &checkpoint_path,
            "temporary-observability discovery checkpoint",
        )?;
        let has_world = artifact_fs::regular_file_exists(
            &world_path,
            "temporary-observability discovery world",
        )?;
        if has_checkpoint && has_world {
            let checkpoint: SimulationCheckpoint = read_json(&checkpoint_path)?;
            if checkpoint.temporary_mobility.program().is_some() {
                found.push(directory);
                continue;
            }
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
    let content = artifact_fs::read_to_string(path, "temporary observability source artifact")?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(value)?;
    let payload = format!("{json}\n");
    artifact_fs::atomic_write(path, payload.as_bytes(), "temporary observability output")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use anthrosim_core::{
        ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig, PopulationConfig,
        ResourceConfig, Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule,
        TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig, ids::CellId,
    };

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn paused_run_with_resume_population_derives_and_verifies_exactly() {
        let region = FocalRegion::new(
            "paused-observability-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4)],
        )
        .expect("region");
        let schedule = TemporaryMobilitySchedule::new(
            "paused-observability-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![360],
            10,
        )
        .expect("schedule");
        let temporary = TemporaryMobilityConfig::new(
            region,
            schedule,
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .expect("temporary mobility");
        let config = ExperimentConfig::new(96_601, 2)
            .with_world(WorldConfig::new(4, 1))
            .with_population(
                PopulationConfig::new(12)
                    .with_target_household_size(2)
                    .with_max_person_records(128),
            )
            .with_resources(ResourceConfig::synthetic_validation_v1())
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
            .with_temporary_mobility(temporary);
        let simulation = Simulation::new(config).expect("simulation");
        let world = simulation.world().clone();
        let checkpoint = simulation.checkpoint_at_year(1).expect("paused checkpoint");
        assert!(checkpoint.temporary_mobility.program().is_some());

        let root = test_dir("paused");
        fs::create_dir_all(&root).expect("create test directory");
        write_json(&root.join("world.json"), &world).expect("world");
        write_json(&root.join("checkpoint.json"), &checkpoint).expect("checkpoint");
        write_json(&root.join("events.json"), &checkpoint.events).expect("events");
        write_json(
            &root.join("resume-start-population.json"),
            &checkpoint.population,
        )
        .expect("resume population");

        process_run(&root, false).expect("derive paused report");
        assert!(root.join("temporary-observability.json").is_file());
        process_run(&root, true).expect("verify paused report");

        let report: TemporaryMobilityObservabilityReport =
            read_json(&root.join("temporary-observability.json")).expect("report");
        assert_eq!(report.source.end_day, checkpoint.time.days());
        assert_eq!(report.source.run_state_digest64, checkpoint.state_digest64);
        assert_eq!(
            report.summary.persistent_residence_person_days,
            report.summary.total_living_person_days
        );
        assert_eq!(
            report.summary.at_residence_person_days
                + report.summary.visitor_person_days
                + report.summary.transit_person_days,
            report.summary.total_living_person_days
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn report_writer_rejects_symlink_without_touching_target() {
        use std::os::unix::fs::symlink;

        let root = test_dir("output-symlink");
        fs::create_dir_all(&root).unwrap();
        let outside = root.with_extension("outside-report.json");
        fs::write(&outside, "outside sentinel\n").unwrap();
        let output = root.join("temporary-observability.json");
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
    fn broken_population_symlink_is_rejected_instead_of_falling_back() {
        use std::os::unix::fs::symlink;

        let root = test_dir("broken-population-symlink");
        fs::create_dir_all(&root).unwrap();
        let missing = root.with_extension("missing-population.json");
        symlink(&missing, root.join("initial-population.json")).unwrap();

        let error = artifact_fs::regular_file_exists(
            &root.join("initial-population.json"),
            "initial population artifact",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("symbolic link"));

        let _ = fs::remove_dir_all(root);
    }

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-temporary-observability-{label}-{}-{id}",
            std::process::id()
        ))
    }
}
