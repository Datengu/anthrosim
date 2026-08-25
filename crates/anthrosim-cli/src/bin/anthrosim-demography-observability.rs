use std::{fs, path::PathBuf, process::ExitCode};

use anthrosim_core::{
    DemographyObservabilityReport, Population, PopulationInitialization, SimulationCheckpoint,
    World, derive_demography_observability, rng::RngFactory,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-demography-observability",
    version,
    about = "Derive or verify the versioned M2 demographic validation report for a run bundle"
)]
struct Cli {
    /// Completed or annual-boundary AnthroSim run directory.
    #[arg(long)]
    run_dir: PathBuf,

    /// Optional output path. Defaults to <run-dir>/demography-observability.json.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Compare the freshly derived report with this existing JSON and fail on any difference.
    #[arg(long, conflicts_with = "output")]
    check: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-demography-observability: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: SimulationCheckpoint = read_json(&cli.run_dir.join("checkpoint.json"))?;
    let world: World = read_json(&cli.run_dir.join("world.json"))?;
    world.validate()?;
    if world.digest64() != checkpoint.world_digest64 {
        return Err("world.json does not match checkpoint.json".into());
    }
    let initial_population = resolve_initial_population(&cli.run_dir, &checkpoint, &world)?;
    let report = derive_demography_observability(&initial_population, &checkpoint)?;

    if let Some(path) = cli.check {
        let expected: DemographyObservabilityReport = read_json(&path)?;
        if expected != report {
            return Err(format!(
                "derived demographic observability does not match {}",
                path.display()
            )
            .into());
        }
        println!("verified {}", path.display());
        return Ok(());
    }

    let output = cli
        .output
        .unwrap_or_else(|| cli.run_dir.join("demography-observability.json"));
    write_json(&output, &report)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn resolve_initial_population(
    run_dir: &std::path::Path,
    checkpoint: &SimulationCheckpoint,
    world: &World,
) -> Result<Population, Box<dyn std::error::Error>> {
    let path = run_dir.join("initial-population.json");
    if path.is_file() {
        let population: Population = read_json(&path)?;
        population.validate(world)?;
        return Ok(population);
    }

    let config = checkpoint.experiment.population;
    let population = match config.initialization {
        PopulationInitialization::SyntheticValidationV1 => {
            Population::initialize(config, world, RngFactory::new(checkpoint.experiment.seed))?
        }
        PopulationInitialization::DeclaredFounderStateV1 => {
            let definition = checkpoint
                .experiment
                .founder_population
                .as_ref()
                .ok_or("declared founder mode has no founderPopulation definition")?;
            Population::initialize_declared_founder_state_v1(config, definition, world)?
        }
    };
    Ok(population)
}

fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{json}\n"))?;
    Ok(())
}
