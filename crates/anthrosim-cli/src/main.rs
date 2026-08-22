use std::{fs, path::Path, path::PathBuf, process::ExitCode};

use anthrosim_core::{
    ExperimentConfig, MigrationConfig, Population, PopulationConfig, RecordedRun, ResourceConfig,
    Simulation, SimulationCheckpoint, World, WorldConfig,
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

        /// Disable M4 household migration while retaining all other systems.
        #[arg(long, default_value_t = false)]
        disable_migration: bool,

        /// Manhattan-radius local knowledge used for migration destination discovery.
        #[arg(long, default_value_t = 3)]
        migration_radius: u16,

        /// Optional path to write the JSON run manifest (legacy single-file mode).
        #[arg(long)]
        output: Option<PathBuf>,

        /// Optional M5 controlled run directory containing offline analysis artifacts.
        #[arg(long)]
        run_dir: Option<PathBuf>,

        /// Pause at this completed annual boundary and write a resumable checkpoint.
        #[arg(long, requires = "run_dir")]
        checkpoint_year: Option<u64>,

        /// Optional path to write the full versioned synthetic world as JSON.
        #[arg(long)]
        world_output: Option<PathBuf>,

        /// Optional path to write full initialized population state as JSON.
        #[arg(long)]
        population_output: Option<PathBuf>,
    },

    /// Resume a deterministic M5 annual-boundary checkpoint to its configured duration.
    Resume {
        /// Checkpoint JSON previously written by AnthroSim M5.
        #[arg(long)]
        checkpoint: PathBuf,

        /// Controlled output directory for the completed resumed run.
        #[arg(long)]
        run_dir: PathBuf,

        /// Optional additional path to write the final manifest.
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
            world_width,
            world_height,
            population,
            household_size,
            max_person_records,
            resource_productivity_scale_permille,
            annual_food_need,
            disable_migration,
            migration_radius,
            output,
            run_dir,
            checkpoint_year,
            world_output,
            population_output,
        } => {
            let resources = ResourceConfig::synthetic_validation_v1()
                .with_productivity_scale_permille(resource_productivity_scale_permille)
                .with_annual_need_units_per_person(annual_food_need);
            let migration = MigrationConfig::synthetic_validation_v1()
                .with_enabled(!disable_migration)
                .with_candidate_radius_cells(migration_radius);
            let config = ExperimentConfig::new(seed, years)
                .with_world(WorldConfig::new(world_width, world_height))
                .with_population(
                    PopulationConfig::new(population)
                        .with_target_household_size(household_size)
                        .with_max_person_records(max_person_records),
                )
                .with_resources(resources)
                .with_migration(migration);
            let simulation = Simulation::new(config)?;

            if let Some(path) = world_output {
                write_json(&path, simulation.world())?;
                println!("wrote world {}", path.display());
            }
            if let Some(path) = population_output {
                write_json(&path, simulation.population())?;
                println!("wrote population {}", path.display());
            }

            if let Some(target_year) = checkpoint_year {
                let directory = run_dir.expect("clap requires run-dir with checkpoint-year");
                write_json(&directory.join("world.json"), simulation.world())?;
                write_json(
                    &directory.join("initial-population.json"),
                    simulation.population(),
                )?;
                let checkpoint = simulation.checkpoint_at_year(target_year)?;
                write_checkpoint_bundle(&directory, &checkpoint)?;
                println!(
                    "wrote checkpoint at year {target_year} to {}",
                    directory.display()
                );
                return Ok(());
            }

            if let Some(directory) = run_dir {
                let world = simulation.world().clone();
                let initial_population = simulation.population().clone();
                let recorded = simulation.run_recorded()?;
                write_completed_bundle(&directory, &world, &initial_population, &recorded)?;
                if let Some(path) = output {
                    write_json(&path, &recorded.manifest)?;
                    println!("wrote manifest {}", path.display());
                }
                println!("wrote run bundle {}", directory.display());
            } else {
                let manifest = simulation.run()?;
                if let Some(path) = output {
                    write_json(&path, &manifest)?;
                    println!("wrote manifest {}", path.display());
                } else {
                    println!("{}", serde_json::to_string_pretty(&manifest)?);
                }
            }
        }
        Command::Resume {
            checkpoint,
            run_dir,
            output,
        } => {
            let checkpoint: SimulationCheckpoint = read_json(&checkpoint)?;
            let simulation = Simulation::from_checkpoint(checkpoint)?;
            let world = simulation.world().clone();
            let resume_population = simulation.population().clone();
            let recorded = simulation.run_recorded()?;

            fs::create_dir_all(&run_dir)?;
            write_json(&run_dir.join("world.json"), &world)?;
            let initial_path = run_dir.join("initial-population.json");
            if !initial_path.exists() {
                write_json(
                    &run_dir.join("resume-start-population.json"),
                    &resume_population,
                )?;
            }
            write_recorded_outputs(&run_dir, &recorded)?;
            if let Some(path) = output {
                write_json(&path, &recorded.manifest)?;
                println!("wrote manifest {}", path.display());
            }
            println!("wrote resumed run bundle {}", run_dir.display());
        }
    }

    Ok(())
}

fn write_completed_bundle(
    directory: &Path,
    world: &World,
    initial_population: &Population,
    recorded: &RecordedRun,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    write_json(&directory.join("world.json"), world)?;
    write_json(
        &directory.join("initial-population.json"),
        initial_population,
    )?;
    write_recorded_outputs(directory, recorded)
}

fn write_recorded_outputs(
    directory: &Path,
    recorded: &RecordedRun,
) -> Result<(), Box<dyn std::error::Error>> {
    write_json(&directory.join("manifest.json"), &recorded.manifest)?;
    write_json(&directory.join("events.json"), recorded.events())?;
    write_json(&directory.join("metrics.json"), recorded.metrics())?;
    write_json(&directory.join("checkpoint.json"), &recorded.checkpoint)?;
    Ok(())
}

fn write_checkpoint_bundle(
    directory: &Path,
    checkpoint: &SimulationCheckpoint,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    write_json(&directory.join("events.json"), &checkpoint.events)?;
    write_json(&directory.join("metrics.json"), &checkpoint.metrics)?;
    write_json(&directory.join("checkpoint.json"), checkpoint)?;
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
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
