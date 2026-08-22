use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use anthrosim_core::{
    ExperimentConfig, MigrationConfig, Population, PopulationConfig, RecordedRun, ResourceConfig,
    Simulation, SimulationCheckpoint, World, WorldConfig,
};
use clap::{Parser, Subcommand};
use serde::Serialize;

const ENSEMBLE_PLAN_SCHEMA_VERSION: u32 = 1;
const ENSEMBLE_COMPLETION_SCHEMA_VERSION: u32 = 1;

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

    /// Execute many ordinary completed AnthroSim runs over deterministic seed variation.
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

        /// New or empty directory that will contain the ensemble plan and child run bundles.
        #[arg(long)]
        run_dir: PathBuf,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsembleRunSettings {
    years: u64,
    world_width: u32,
    world_height: u32,
    population: u32,
    household_size: u16,
    max_person_records: u64,
    resource_productivity_scale_permille: u16,
    annual_food_need: u32,
    disable_migration: bool,
    migration_radius: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsemblePlan {
    schema_version: u32,
    definition: EnsembleDefinition,
    runs: Vec<PlannedRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsembleDefinition {
    seeds: Vec<u64>,
    settings: EnsembleRunSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlannedRun {
    seed: u64,
    relative_run_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EnsembleRunCompletion {
    schema_version: u32,
    seed: u64,
    status: &'static str,
    manifest: &'static str,
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
            let settings = EnsembleRunSettings {
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
            annual_food_need,
            disable_migration,
            migration_radius,
            run_dir,
        } => {
            let seeds = resolve_ensemble_seeds(seeds, seed_start, seed_count)?;
            let settings = EnsembleRunSettings {
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
            };
            execute_ensemble(&run_dir, settings, seeds)?;
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

fn experiment_config(seed: u64, settings: &EnsembleRunSettings) -> ExperimentConfig {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(settings.resource_productivity_scale_permille)
        .with_annual_need_units_per_person(settings.annual_food_need);
    let migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(!settings.disable_migration)
        .with_candidate_radius_cells(settings.migration_radius);
    ExperimentConfig::new(seed, settings.years)
        .with_world(WorldConfig::new(
            settings.world_width,
            settings.world_height,
        ))
        .with_population(
            PopulationConfig::new(settings.population)
                .with_target_household_size(settings.household_size)
                .with_max_person_records(settings.max_person_records),
        )
        .with_resources(resources)
        .with_migration(migration)
}

fn resolve_ensemble_seeds(
    explicit_seeds: Vec<u64>,
    seed_start: Option<u64>,
    seed_count: Option<u32>,
) -> Result<Vec<u64>, io::Error> {
    if !explicit_seeds.is_empty() {
        validate_unique_seeds(&explicit_seeds)?;
        return Ok(explicit_seeds);
    }

    let (start, count) = match (seed_start, seed_count) {
        (Some(start), Some(count)) => (start, count),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "ensemble requires either --seeds or both --seed-start and --seed-count",
            ));
        }
    };

    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--seed-count must be greater than zero",
        ));
    }

    let final_offset = u64::from(count - 1);
    start.checked_add(final_offset).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "seed range exceeds the maximum u64 seed",
        )
    })?;

    Ok((0..count).map(|offset| start + u64::from(offset)).collect())
}

fn validate_unique_seeds(seeds: &[u64]) -> Result<(), io::Error> {
    let mut seen = HashSet::with_capacity(seeds.len());
    for &seed in seeds {
        if !seen.insert(seed) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("duplicate ensemble seed {seed} would target the same run directory"),
            ));
        }
    }
    Ok(())
}

fn plan_ensemble(
    settings: EnsembleRunSettings,
    seeds: Vec<u64>,
) -> Result<EnsemblePlan, io::Error> {
    if seeds.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ensemble must contain at least one seed",
        ));
    }
    validate_unique_seeds(&seeds)?;

    let runs = seeds
        .iter()
        .map(|&seed| PlannedRun {
            seed,
            relative_run_dir: format!("runs/seed-{seed:020}"),
        })
        .collect();

    Ok(EnsemblePlan {
        schema_version: ENSEMBLE_PLAN_SCHEMA_VERSION,
        definition: EnsembleDefinition { seeds, settings },
        runs,
    })
}

fn execute_ensemble(
    directory: &Path,
    settings: EnsembleRunSettings,
    seeds: Vec<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan_ensemble(settings.clone(), seeds)?;
    require_empty_ensemble_directory(directory)?;
    fs::create_dir_all(directory)?;
    write_json(&directory.join("ensemble-plan.json"), &plan)?;

    for planned in &plan.runs {
        let run_directory = directory.join(&planned.relative_run_dir);
        let simulation = Simulation::new(experiment_config(planned.seed, &settings))?;
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded()?;
        write_completed_bundle(&run_directory, &world, &initial_population, &recorded)?;
        write_json(
            &run_directory.join("completion.json"),
            &EnsembleRunCompletion {
                schema_version: ENSEMBLE_COMPLETION_SCHEMA_VERSION,
                seed: planned.seed,
                status: "completed",
                manifest: "manifest.json",
            },
        )?;
        println!(
            "completed ensemble seed {} -> {}",
            planned.seed,
            run_directory.display()
        );
    }

    println!(
        "completed ensemble with {} runs in {}",
        plan.runs.len(),
        directory.display()
    );
    Ok(())
}

fn require_empty_ensemble_directory(directory: &Path) -> Result<(), io::Error> {
    if !directory.exists() {
        return Ok(());
    }
    if !directory.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "ensemble output path {} exists and is not a directory",
                directory.display()
            ),
        ));
    }
    if fs::read_dir(directory)?.next().transpose()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "ensemble output directory {} is not empty; refusing to mix or overwrite run artifacts",
                directory.display()
            ),
        ));
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

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn small_settings() -> EnsembleRunSettings {
        EnsembleRunSettings {
            years: 0,
            world_width: 4,
            world_height: 4,
            population: 12,
            household_size: 4,
            max_person_records: 100,
            resource_productivity_scale_permille: 1_000,
            annual_food_need: 100,
            disable_migration: false,
            migration_radius: 3,
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("anthrosim-{label}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn explicit_seed_plan_is_deterministic_and_separated() {
        let settings = small_settings();
        let first = plan_ensemble(settings.clone(), vec![9, 2, 42]).expect("valid plan");
        let second = plan_ensemble(settings, vec![9, 2, 42]).expect("valid plan");

        assert_eq!(first, second);
        assert_eq!(first.definition.seeds, vec![9, 2, 42]);
        assert_eq!(
            first.runs[0].relative_run_dir,
            "runs/seed-00000000000000000009"
        );
        assert_eq!(
            first.runs[1].relative_run_dir,
            "runs/seed-00000000000000000002"
        );
        assert_ne!(
            first.runs[0].relative_run_dir,
            first.runs[1].relative_run_dir
        );
    }

    #[test]
    fn consecutive_seed_range_is_stable_and_overflow_checked() {
        assert_eq!(
            resolve_ensemble_seeds(Vec::new(), Some(100), Some(4)).expect("valid range"),
            vec![100, 101, 102, 103]
        );
        assert!(resolve_ensemble_seeds(Vec::new(), Some(7), Some(0)).is_err());
        assert!(resolve_ensemble_seeds(Vec::new(), Some(u64::MAX), Some(2)).is_err());
    }

    #[test]
    fn duplicate_explicit_seeds_are_rejected() {
        assert!(resolve_ensemble_seeds(vec![5, 8, 5], None, None).is_err());
    }

    #[test]
    fn ensemble_writes_isolated_ordinary_completed_run_bundles() {
        let root = temp_path("ensemble-test");
        execute_ensemble(&root, small_settings(), vec![11, 12]).expect("ensemble completes");

        let plan: serde_json::Value = read_json(&root.join("ensemble-plan.json")).expect("plan");
        assert_eq!(plan["definition"]["seeds"], serde_json::json!([11, 12]));

        for seed in [11_u64, 12] {
            let run_dir = root.join(format!("runs/seed-{seed:020}"));
            for artifact in [
                "manifest.json",
                "world.json",
                "initial-population.json",
                "events.json",
                "metrics.json",
                "checkpoint.json",
                "completion.json",
            ] {
                assert!(run_dir.join(artifact).is_file(), "missing {artifact}");
            }

            let manifest: serde_json::Value =
                read_json(&run_dir.join("manifest.json")).expect("manifest");
            assert_eq!(manifest["experiment"]["seed"], seed);
            let completion: serde_json::Value =
                read_json(&run_dir.join("completion.json")).expect("completion");
            assert_eq!(completion["seed"], seed);
            assert_eq!(completion["status"], "completed");
        }

        assert!(execute_ensemble(&root, small_settings(), vec![11, 12]).is_err());
        fs::remove_dir_all(root).expect("cleanup");
    }
}
