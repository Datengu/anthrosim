use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[allow(dead_code)]
#[path = "../run_directory.rs"]
mod run_directory;

use run_directory::recover_interrupted_replacement;

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-recover",
    version,
    about = "Safely recover an interrupted AnthroSim run-directory replacement"
)]
struct Cli {
    /// Canonical run directory whose sibling transaction remnants should be reconciled.
    #[arg(long)]
    run_dir: PathBuf,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match recover_interrupted_replacement(&cli.run_dir) {
        Ok(outcome) => {
            println!(
                "AnthroSim run-directory recovery for {}: {}.",
                cli.run_dir.display(),
                outcome
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "anthrosim-recover: could not safely recover {}: {error}",
                cli.run_dir.display()
            );
            ExitCode::FAILURE
        }
    }
}
