use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

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
        if !path.is_file() {
            return Err(format!(
                "{} is not a usable run bundle: missing {}",
                run_dir.display(),
                path.file_name().unwrap_or_default().to_string_lossy()
            )
            .into());
        }
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
    if events_path.is_file() {
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
        if !output.is_file() {
            return Err(format!("missing derived report {}", output.display()).into());
        }
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
    if initial_path.is_file() {
        let population: Population = read_json(&initial_path)?;
        population.validate(world)?;
        return Ok(population);
    }

    let resume_path = run_dir.join("resume-start-population.json");
    if !resume_path.is_file() {
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
        if checkpoint_path.is_file() && directory.join("world.json").is_file() {
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
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{json}\n"))?;
    Ok(())
}
