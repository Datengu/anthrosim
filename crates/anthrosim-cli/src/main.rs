use std::{fs, path::Path, path::PathBuf, process::ExitCode};

use anthrosim_core::{ExperimentConfig, Simulation, WorldConfig};
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

        /// Number of simulated years to execute.
        #[arg(long, default_value_t = 1_000)]
        years: u64,

        /// Synthetic world width in cells.
        #[arg(long, default_value_t = 128)]
        world_width: u32,

        /// Synthetic world height in cells.
        #[arg(long, default_value_t = 128)]
        world_height: u32,

        /// Optional path to write the JSON run manifest.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Optional path to write the full versioned synthetic world as JSON.
        #[arg(long)]
        world_output: Option<PathBuf>,
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
            output,
            world_output,
        } => {
            let config = ExperimentConfig::new(seed, years)
                .with_world(WorldConfig::new(world_width, world_height));
            let simulation = Simulation::new(config)?;

            if let Some(path) = world_output {
                write_json(&path, simulation.world())?;
                println!("wrote world {}", path.display());
            }

            let manifest = simulation.run();
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
