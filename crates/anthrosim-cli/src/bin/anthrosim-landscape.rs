use std::{fs, path::{Path, PathBuf}, process::ExitCode};

use anthrosim_core::{
    ExperimentConfig, LandscapeBundle, LandscapeCheckpoint, LandscapeRecordedRun,
    LandscapeSimulation, MigrationConfig, Population, PopulationConfig, ResourceConfig, World,
    WorldConfig, validate_landscape_recorded_run_invariants,
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-landscape",
    version,
    about = "M8.3 deterministic normalized-landscape runner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run AnthroSim with a validated immutable normalized landscape binding.
    Run {
        /// Normalized M8.1 LandscapeBundle JSON.
        #[arg(long)]
        landscape: PathBuf,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long, default_value_t = 1_000)]
        years: u64,
        #[arg(long, default_value_t = 10_000)]
        population: u32,
        #[arg(long, default_value_t = 5)]
        household_size: u16,
        #[arg(long, default_value_t = 1_000_000)]
        max_person_records: u64,
        #[arg(long, default_value_t = 1_000)]
        resource_productivity_scale_permille: u16,
        #[arg(long, default_value_t = 1_000)]
        resource_seasonality_scale_permille: u16,
        #[arg(long, default_value_t = 100)]
        annual_food_need: u32,
        #[arg(long, default_value_t = false)]
        disable_migration: bool,
        #[arg(long, default_value_t = 3)]
        migration_radius: u16,
        /// Controlled output directory containing core and landscape-bound artifacts.
        #[arg(long)]
        run_dir: PathBuf,
        /// Pause at this completed annual boundary and emit a landscape-bound checkpoint.
        #[arg(long)]
        checkpoint_year: Option<u64>,
    },

    /// Resume only when the supplied normalized landscape exactly matches the checkpoint binding.
    Resume {
        /// M8.3 landscape-checkpoint.json wrapper.
        #[arg(long)]
        checkpoint: PathBuf,
        /// Normalized M8.1 LandscapeBundle JSON. It must match the stored identity exactly.
        #[arg(long)]
        landscape: PathBuf,
        /// Controlled output directory for the resumed run.
        #[arg(long)]
        run_dir: PathBuf,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("anthrosim-landscape: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Run {
            landscape,
            seed,
            years,
            population,
            household_size,
            max_person_records,
            resource_productivity_scale_permille,
            resource_seasonality_scale_permille,
            annual_food_need,
            disable_migration,
            migration_radius,
            run_dir,
            checkpoint_year,
        } => {
            let landscape: LandscapeBundle = read_json(&landscape)?;
            landscape.validate()?;
            let config = experiment_config(
                seed,
                years,
                landscape.width,
                landscape.height,
                population,
                household_size,
                max_person_records,
                resource_productivity_scale_permille,
                resource_seasonality_scale_permille,
                annual_food_need,
                disable_migration,
                migration_radius,
            );
            let simulation = LandscapeSimulation::new(config, landscape.clone())?;
            let world = simulation.world().clone();
            let initial_population = simulation.population().clone();

            if let Some(target_year) = checkpoint_year {
                let checkpoint = simulation.checkpoint_at_year(target_year)?;
                write_landscape_checkpoint_bundle(
                    &run_dir,
                    &landscape,
                    &world,
                    &initial_population,
                    &checkpoint,
                )?;
                println!(
                    "wrote landscape-bound checkpoint at year {target_year} to {}",
                    run_dir.display()
                );
            } else {
                let recorded = simulation.run_recorded()?;
                write_completed_landscape_bundle(
                    &run_dir,
                    &landscape,
                    &world,
                    &initial_population,
                    &recorded,
                )?;
                println!("wrote landscape-bound run bundle {}", run_dir.display());
            }
        }
        Command::Resume {
            checkpoint,
            landscape,
            run_dir,
        } => {
            let checkpoint: LandscapeCheckpoint = read_json(&checkpoint)?;
            let landscape: LandscapeBundle = read_json(&landscape)?;
            let simulation = LandscapeSimulation::from_checkpoint(checkpoint, landscape.clone())?;
            let world = simulation.world().clone();
            let resume_population = simulation.population().clone();
            let recorded = simulation.run_recorded()?;

            fs::create_dir_all(&run_dir)?;
            let initial_population_path = run_dir.join("initial-population.json");
            if !initial_population_path.exists() {
                write_json(&run_dir.join("resume-start-population.json"), &resume_population)?;
            }
            write_completed_landscape_bundle(
                &run_dir,
                &landscape,
                &world,
                &resume_population,
                &recorded,
            )?;
            println!("wrote resumed landscape-bound run bundle {}", run_dir.display());
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn experiment_config(
    seed: u64,
    years: u64,
    width: u32,
    height: u32,
    population: u32,
    household_size: u16,
    max_person_records: u64,
    productivity_scale: u16,
    seasonality_scale: u16,
    annual_food_need: u32,
    disable_migration: bool,
    migration_radius: u16,
) -> ExperimentConfig {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(productivity_scale)
        .with_seasonality_scale_permille(seasonality_scale)
        .with_annual_need_units_per_person(annual_food_need);
    let migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(!disable_migration)
        .with_candidate_radius_cells(migration_radius);
    ExperimentConfig::new(seed, years)
        .with_world(WorldConfig::new(width, height))
        .with_population(
            PopulationConfig::new(population)
                .with_target_household_size(household_size)
                .with_max_person_records(max_person_records),
        )
        .with_resources(resources)
        .with_migration(migration)
}

fn write_completed_landscape_bundle(
    directory: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    initial_population: &Population,
    recorded: &LandscapeRecordedRun,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_landscape_recorded_run_invariants(recorded)?;
    fs::create_dir_all(directory)?;
    write_json(&directory.join("landscape.json"), landscape)?;
    write_json(&directory.join("world.json"), world)?;
    write_json(&directory.join("initial-population.json"), initial_population)?;
    write_json(&directory.join("manifest.json"), recorded.core_manifest())?;
    write_json(&directory.join("landscape-manifest.json"), &recorded.manifest)?;
    write_json(&directory.join("events.json"), recorded.events())?;
    write_json(&directory.join("metrics.json"), recorded.metrics())?;
    write_json(&directory.join("checkpoint.json"), recorded.core_checkpoint())?;
    write_json(
        &directory.join("landscape-checkpoint.json"),
        &recorded.checkpoint,
    )?;
    Ok(())
}

fn write_landscape_checkpoint_bundle(
    directory: &Path,
    landscape: &LandscapeBundle,
    world: &World,
    initial_population: &Population,
    checkpoint: &LandscapeCheckpoint,
) -> Result<(), Box<dyn std::error::Error>> {
    checkpoint.landscape.validate_bundle(landscape)?;
    fs::create_dir_all(directory)?;
    write_json(&directory.join("landscape.json"), landscape)?;
    write_json(&directory.join("world.json"), world)?;
    write_json(&directory.join("initial-population.json"), initial_population)?;
    write_json(&directory.join("events.json"), &checkpoint.core_checkpoint.events)?;
    write_json(&directory.join("metrics.json"), &checkpoint.core_checkpoint.metrics)?;
    write_json(&directory.join("checkpoint.json"), &checkpoint.core_checkpoint)?;
    write_json(&directory.join("landscape-checkpoint.json"), checkpoint)?;
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_json<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{json}\n"))?;
    Ok(())
}