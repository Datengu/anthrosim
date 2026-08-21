use std::{fs, path::PathBuf, process::ExitCode};

use anthrosim_core::{ExperimentConfig, Simulation};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "anthrosim", version, about = "Headless AnthroSim experiment runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute the deterministic Milestone 0 simulation lifecycle.
    Run {
        /// Master seed for all named deterministic random streams.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Number of simulated years to execute.
        #[arg(long, default_value_t = 1_000)]
        years: u64,

        /// Optional path to write the JSON run manifest.
        #[arg(long)]
        output: Option<PathBuf>,
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
            output,
        } => {
            let config = ExperimentConfig::new(seed, years);
            let manifest = Simulation::new(config).run();
            let json = serde_json::to_string_pretty(&manifest)?;

            if let Some(path) = output {
                if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&path, format!("{json}\n"))?;
                println!("wrote {}", path.display());
            } else {
                println!("{json}");
            }
        }
    }

    Ok(())
}
