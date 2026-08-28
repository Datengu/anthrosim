use std::{path::PathBuf, process::ExitCode};

#[path = "../artifact_fs.rs"]
mod artifact_fs;

use anthrosim_core::{
    HouseholdObservabilityReport, SimulationCheckpoint, derive_household_observability,
};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-household-observability",
    version,
    about = "Derive or verify household topology observability for a run bundle"
)]
struct Cli {
    #[arg(long)]
    run_dir: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, conflicts_with = "output")]
    check: Option<PathBuf>,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-household-observability: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let checkpoint: SimulationCheckpoint = read_json(&cli.run_dir.join("checkpoint.json"))?;
    let report = derive_household_observability(
        &checkpoint.population,
        &checkpoint.experiment,
        checkpoint.time.days(),
    )?;

    if let Some(path) = cli.check {
        let expected: HouseholdObservabilityReport = read_json(&path)?;
        if expected != report {
            return Err(format!(
                "derived household observability does not match {}",
                path.display()
            )
            .into());
        }
        println!("verified {}", path.display());
        return Ok(());
    }

    let output = cli
        .output
        .unwrap_or_else(|| cli.run_dir.join("household-observability.json"));
    write_json(&output, &report)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> Result<T, Box<dyn std::error::Error>> {
    let content = artifact_fs::read_to_string(path, "household observability source artifact")?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(value)?;
    artifact_fs::atomic_write(
        path,
        format!("{json}\n").as_bytes(),
        "household observability output",
    )?;
    Ok(())
}
