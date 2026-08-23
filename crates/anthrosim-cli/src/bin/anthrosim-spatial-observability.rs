use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anthrosim_core::{
    LandscapeBundle, Population, SimulationCheckpoint, SpatialLandscapeCheckpoint,
    SpatialObservabilityReport, World, derive_spatial_observability,
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
    let initial_population_path = run_dir.join("initial-population.json");
    let checkpoint_path = run_dir.join("checkpoint.json");
    for path in [
        &landscape_path,
        &world_path,
        &initial_population_path,
        &checkpoint_path,
    ] {
        if !path.is_file() {
            return Err(format!(
                "{} is not a complete landscape-bound run: missing {}",
                run_dir.display(),
                path.file_name().unwrap_or_default().to_string_lossy()
            )
            .into());
        }
    }

    let landscape: LandscapeBundle = read_json(&landscape_path)?;
    let world: World = read_json(&world_path)?;
    let initial_population: Population = read_json(&initial_population_path)?;
    let checkpoint: SimulationCheckpoint = read_json(&checkpoint_path)?;
    let spatial_checkpoint_path = run_dir.join("landscape-checkpoint.json");
    let spatial_checkpoint = if spatial_checkpoint_path.is_file() {
        let value: serde_json::Value = read_json(&spatial_checkpoint_path)?;
        if value.get("spatial").is_some() {
            Some(serde_json::from_value::<SpatialLandscapeCheckpoint>(value)?)
        } else {
            None
        }
    } else {
        None
    };
    if let Some(wrapper) = &spatial_checkpoint {
        if wrapper.core_checkpoint != checkpoint {
            return Err(format!(
                "{} landscape checkpoint wrapper disagrees with checkpoint.json",
                run_dir.display()
            )
            .into());
        }
        wrapper.landscape.validate_bundle(&landscape)?;
    }

    let report = derive_spatial_observability(
        &landscape,
        &world,
        &initial_population,
        &checkpoint,
        spatial_checkpoint.as_ref().map(|wrapper| &wrapper.spatial),
    )?;
    let output = run_dir.join("spatial-observability.json");
    if check {
        if !output.is_file() {
            return Err(format!("missing derived report {}", output.display()).into());
        }
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

fn discover_spatial_run_dirs(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut pending = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = pending.pop() {
        if !directory.is_dir() {
            continue;
        }
        if directory.join("landscape.json").is_file()
            && directory.join("checkpoint.json").is_file()
            && directory.join("world.json").is_file()
            && directory.join("initial-population.json").is_file()
        {
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
