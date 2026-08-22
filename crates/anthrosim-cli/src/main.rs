use std::{fs, path::Path, path::PathBuf, process::ExitCode};

use anthrosim_core::{
    ExperimentConfig, PopulationConfig, ResourceConfig, Simulation, WorldConfig,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim",
    version,
    about = "Headless AnthroSim experiment runner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute the deterministic AnthroSim lifecycle.
    Run {
        /// Master seed for all named deterministic random streams.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Number of simulated years to execute unless a stop condition occurs.
        #[arg(long, default_value_t = 1_000)]
        years: u64,

        /// Synthetic world width in cells.
        #[arg(long, default_value_t = 128)]
        world_width: u32,

        /// Synthetic world height in cells.
        #[arg(long, default_value_t = 128)]
        world_height: u32,

        /// Number of persistent synthetic founder records to initialize.
        #[arg(long, default_value_t = 10_000)]
        population: u32,

        /// Target number of co-resident founders per synthetic household.
        #[arg(long, default_value_t = 5)]
        household_size: u16,

        /// Operational ceiling for persistent person records; this is not a carrying capacity.
        #[arg(long, default_value_t = 1_000_000)]
        max_person_records: u64,

        /// Synthetic M3 environmental productivity scale, in permille (0..=1000).
        #[arg(long, default_value_t = 1_000)]
        resource_productivity_scale_permille: u16,

        /// Synthetic annual resource need per living person, in abstract units.
        #[arg(long, default_value_t = 100)]
        annual_food_need: u32,

        /// Optional path to write the JSON run manifest.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Optional path to write the full versioned synthetic world as JSON.
        #[arg(long)]
        world_output: Option<PathBuf>,

        /// Optional path to write full initialized population state as JSON.
        #[arg(long)]
        population_output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Run {
            seed,
            years,
            world_width,
            world_height,
            population,
            household_size,
            max_person_records,
            resource_productivity_scale_permille,
            annual_food_need,
            output,
            world_output,
            population_output,
        } => {
            let resources = ResourceConfig::synthetic_validation_v1()
                .with_productivity_scale_permille(resource_productivity_scale_permille)
                .with_annual_need_units_per_person(annual_food_need);
            let config = ExperimentConfig::new(seed, years)
                .with_world(WorldConfig::new(world_width, world_height))
                .with_population(
                    PopulationConfig::new(population)
                        .with_target_household_size(household_size)
                        .with_max_person_records(max_person_records),
                )
                .with_resources(resources);
            let simulation = Simulation::new(config)?;

            if let Some(path) = world_output {
                write_json(&path, simulation.world())?;
                println!("wrote world {}", path.display());
            }
            if let Some(path) = population_output {
                write_json(&path, simulation.population())?;
                println!("wrote population {}", path.display());
            }

            let manifest = simulation.run()?;
            if let Some(path) = output {
                write_json(&path, &manifest)?;
                println!("wrote manifest {}", path.display());
            } else {
                println!("{}", serde_json::to_string_pretty(&manifest)?);
            }
        }
    }

    Ok(())
}

fn write_json<T: serde::Serialize>(
    path: &Path,
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
