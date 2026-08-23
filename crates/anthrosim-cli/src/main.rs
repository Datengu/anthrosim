use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anthrosim_core::{Population, RecordedRun, Simulation, SimulationCheckpoint, World};
use clap::{Parser, Subcommand};

mod ensemble;
mod sweep;

use ensemble::{
    EnsembleRunSettings, execute_ensemble, experiment_config, load_spatial_run_settings,
    resolve_ensemble_seeds,
};
use sweep::{SweepDimensions, execute_sweep};

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

        /// Synthetic seasonal-amplitude scale for renewable productivity, in permille (0..=1000).
        #[arg(long, default_value_t = 1_000)]
        resource_seasonality_scale_permille: u16,

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

    /// Execute or retry many runs under one immutable experiment identity.
    Ensemble {
        /// Explicit deterministic seeds. Comma-separated values are accepted.
        #[arg(
            long,
            value_delimiter = ',',
            num_args = 1..,
            conflicts_with_all = ["seed_start", "seed_count"]
        )]
        seeds: Vec<u64>,

        /// First seed in an inclusive deterministic range.
        #[arg(long, requires = "seed_count", conflicts_with = "seeds")]
        seed_start: Option<u64>,

        /// Number of consecutive seeds beginning at --seed-start.
        #[arg(long, requires = "seed_start", conflicts_with = "seeds")]
        seed_count: Option<u32>,

        /// Number of simulated years to execute unless a stop condition occurs.
        #[arg(long, default_value_t = 1_000)]
        years: u64,

        /// World width in cells; spatial runs must match the normalized landscape.
        #[arg(long, default_value_t = 128)]
        world_width: u32,

        /// World height in cells; spatial runs must match the normalized landscape.
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

        /// M3 environmental productivity scale, in permille (0..=1000).
        #[arg(long, default_value_t = 1_000)]
        resource_productivity_scale_permille: u16,

        /// Seasonal-amplitude scale for renewable productivity, in permille (0..=1000).
        #[arg(long, default_value_t = 1_000)]
        resource_seasonality_scale_permille: u16,

        /// Annual resource need per living person, in abstract units.
        #[arg(long, default_value_t = 100)]
        annual_food_need: u32,

        /// Disable M4 household migration while retaining all other systems.
        #[arg(long, default_value_t = false)]
        disable_migration: bool,

        /// Manhattan-radius local knowledge used for migration destination discovery.
        #[arg(long, default_value_t = 3)]
        migration_radius: u16,

        /// Optional normalized M8.1 LandscapeBundle JSON; requires --mechanisms.
        #[arg(long, requires = "mechanisms")]
        landscape: Option<PathBuf>,

        /// Optional versioned M8.4 spatial mechanism JSON; requires --landscape.
        #[arg(long, requires = "landscape")]
        mechanisms: Option<PathBuf>,

        /// Optional M8 evidence catalogue included in immutable experiment identity; requires --landscape.
        #[arg(long, requires = "landscape")]
        evidence: Option<PathBuf>,

        /// Experiment root containing immutable provenance, statuses and child run bundles.
        #[arg(long)]
        run_dir: PathBuf,

        /// Reconcile and retry only unsuccessful/incomplete runs in this exact experiment.
        #[arg(long, default_value_t = false)]
        retry: bool,
    },

    /// Expand an explicit parameter grid into M7.2 experiments and derived analysis tables.
    Sweep {
        /// Explicit deterministic seeds. Comma-separated values are accepted.
        #[arg(
            long,
            value_delimiter = ',',
            num_args = 1..,
            conflicts_with_all = ["seed_start", "seed_count"]
        )]
        seeds: Vec<u64>,

        /// First seed in an inclusive deterministic range.
        #[arg(long, requires = "seed_count", conflicts_with = "seeds")]
        seed_start: Option<u64>,

        /// Number of consecutive seeds beginning at --seed-start.
        #[arg(long, requires = "seed_start", conflicts_with = "seeds")]
        seed_count: Option<u32>,

        /// Base duration in simulated years for every sweep point.
        #[arg(long, default_value_t = 1_000)]
        years: u64,

        /// Base world width in cells; spatial sweeps must match the normalized landscape.
        #[arg(long, default_value_t = 128)]
        world_width: u32,

        /// Base world height in cells; spatial sweeps must match the normalized landscape.
        #[arg(long, default_value_t = 128)]
        world_height: u32,

        /// Base founder population when --sweep-population is not supplied.
        #[arg(long, default_value_t = 10_000)]
        population: u32,

        /// Base target household size when --sweep-household-size is not supplied.
        #[arg(long, default_value_t = 5)]
        household_size: u16,

        /// Operational ceiling for persistent person records across the sweep.
        #[arg(long, default_value_t = 1_000_000)]
        max_person_records: u64,

        /// Base M3 productivity scale when its sweep dimension is not supplied.
        #[arg(long, default_value_t = 1_000)]
        resource_productivity_scale_permille: u16,

        /// Seasonal-amplitude scale for renewable productivity, in permille (0..=1000).
        #[arg(long, default_value_t = 1_000)]
        resource_seasonality_scale_permille: u16,

        /// Base annual resource need when its sweep dimension is not supplied.
        #[arg(long, default_value_t = 100)]
        annual_food_need: u32,

        /// Base migration enable/disable setting when its sweep dimension is not supplied.
        #[arg(long, default_value_t = false)]
        disable_migration: bool,

        /// Base migration radius when its sweep dimension is not supplied.
        #[arg(long, default_value_t = 3)]
        migration_radius: u16,

        /// Optional normalized M8.1 LandscapeBundle JSON shared by every point; requires --mechanisms.
        #[arg(long, requires = "mechanisms")]
        landscape: Option<PathBuf>,

        /// Optional versioned M8.4 spatial mechanism JSON shared by every point; requires --landscape.
        #[arg(long, requires = "landscape")]
        mechanisms: Option<PathBuf>,

        /// Optional M8 evidence catalogue included in every point's immutable experiment identity; requires --landscape.
        #[arg(long, requires = "landscape")]
        evidence: Option<PathBuf>,

        /// Explicit founder-population values for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_population: Vec<u32>,

        /// Explicit target-household-size values for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_household_size: Vec<u16>,

        /// Explicit M3 productivity-scale values for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_resource_productivity_scale_permille: Vec<u16>,

        /// Explicit seasonal-amplitude scales for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_resource_seasonality_scale_permille: Vec<u16>,

        /// Explicit annual-resource-need values for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_annual_food_need: Vec<u32>,

        /// Explicit migration enabled/disabled values, e.g. false,true.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_disable_migration: Vec<bool>,

        /// Explicit local migration-radius values for the Cartesian parameter grid.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        sweep_migration_radius: Vec<u16>,

        /// Sweep root containing immutable provenance, point experiments and derived analysis.
        #[arg(long)]
        run_dir: PathBuf,

        /// Reconcile and retry this exact immutable sweep without changing its definition.
        #[arg(long, default_value_t = false)]
        retry: bool,
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
            resource_seasonality_scale_permille,
            annual_food_need,
            disable_migration,
            migration_radius,
            output,
            run_dir,
            checkpoint_year,
            world_output,
            population_output,
        } => {
            let settings = EnsembleRunSettings {
                years,
                world_width,
                world_height,
                population,
                household_size,
                max_person_records,
                resource_productivity_scale_permille,
                resource_seasonality_scale_permille,
                annual_food_need,
                disable_migration,
                migration_radius,
                spatial: None,
            };
            let config = experiment_config(seed, &settings);
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
        Command::Ensemble {
            seeds,
            seed_start,
            seed_count,
            years,
            world_width,
            world_height,
            population,
            household_size,
            max_person_records,
            resource_productivity_scale_permille,
            resource_seasonality_scale_permille,
            annual_food_need,
            disable_migration,
            migration_radius,
            landscape,
            mechanisms,
            evidence,
            run_dir,
            retry,
        } => {
            let seeds = resolve_ensemble_seeds(seeds, seed_start, seed_count)?;
            let spatial = match (landscape, mechanisms) {
                (Some(landscape), Some(mechanisms)) => Some(load_spatial_run_settings(
                    &landscape,
                    &mechanisms,
                    evidence.as_deref(),
                )?),
                (None, None) => None,
                _ => unreachable!("clap requires landscape and mechanisms together"),
            };
            let settings = EnsembleRunSettings {
                years,
                world_width,
                world_height,
                population,
                household_size,
                max_person_records,
                resource_productivity_scale_permille,
                resource_seasonality_scale_permille,
                annual_food_need,
                disable_migration,
                migration_radius,
                spatial,
            };
            execute_ensemble(&run_dir, settings, seeds, retry)?;
        }
        Command::Sweep {
            seeds,
            seed_start,
            seed_count,
            years,
            world_width,
            world_height,
            population,
            household_size,
            max_person_records,
            resource_productivity_scale_permille,
            resource_seasonality_scale_permille,
            annual_food_need,
            disable_migration,
            migration_radius,
            landscape,
            mechanisms,
            evidence,
            sweep_population,
            sweep_household_size,
            sweep_resource_productivity_scale_permille,
            sweep_resource_seasonality_scale_permille,
            sweep_annual_food_need,
            sweep_disable_migration,
            sweep_migration_radius,
            run_dir,
            retry,
        } => {
            let seeds = resolve_ensemble_seeds(seeds, seed_start, seed_count)?;
            let spatial = match (landscape, mechanisms) {
                (Some(landscape), Some(mechanisms)) => Some(load_spatial_run_settings(
                    &landscape,
                    &mechanisms,
                    evidence.as_deref(),
                )?),
                (None, None) => None,
                _ => unreachable!("clap requires landscape and mechanisms together"),
            };
            let settings = EnsembleRunSettings {
                years,
                world_width,
                world_height,
                population,
                household_size,
                max_person_records,
                resource_productivity_scale_permille,
                resource_seasonality_scale_permille,
                annual_food_need,
                disable_migration,
                migration_radius,
                spatial,
            };
            let dimensions = SweepDimensions {
                population: sweep_population,
                household_size: sweep_household_size,
                resource_productivity_scale_permille: sweep_resource_productivity_scale_permille,
                resource_seasonality_scale_permille: sweep_resource_seasonality_scale_permille,
                annual_food_need: sweep_annual_food_need,
                disable_migration: sweep_disable_migration,
                migration_radius: sweep_migration_radius,
            };
            execute_sweep(&run_dir, settings, seeds, dimensions, retry)?;
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

fn write_json<T: serde::Serialize + ?Sized>(
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
