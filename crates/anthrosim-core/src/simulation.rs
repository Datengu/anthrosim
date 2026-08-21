use thiserror::Error;

use crate::{
    config::ExperimentConfig,
    manifest::{RunManifest, StopReason},
    population::{Population, PopulationError},
    rng::RngFactory,
    time::SimTime,
    world::{World, WorldError},
};

/// Authoritative headless simulation host.
#[derive(Debug)]
pub struct Simulation {
    config: ExperimentConfig,
    time: SimTime,
    rng_factory: RngFactory,
    world: World,
    population: Population,
}

impl Simulation {
    pub fn new(config: ExperimentConfig) -> Result<Self, SimulationError> {
        if config.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
            return Err(SimulationError::UnsupportedExperimentSchema {
                found: config.schema_version,
                supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
            });
        }

        let rng_factory = RngFactory::new(config.seed);
        let world = World::generate(config.world, rng_factory)?;
        let population = Population::initialize(config.population, &world, rng_factory)?;

        Ok(Self {
            rng_factory,
            config,
            time: SimTime::ZERO,
            world,
            population,
        })
    }

    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.time
    }

    #[must_use]
    pub const fn config(&self) -> &ExperimentConfig {
        &self.config
    }

    #[must_use]
    pub const fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub const fn population(&self) -> &Population {
        &self.population
    }

    /// Run the current simulation lifecycle to the configured duration.
    ///
    /// M2-A establishes persistent people and households. Dynamic life-history
    /// events are introduced in the next M2 step; until then population state
    /// remains unchanged while the authoritative clock advances.
    #[must_use]
    pub fn run(mut self) -> RunManifest {
        // Reserve independent deterministic stream boundaries for upcoming M2
        // processes without coupling their future draws to world generation.
        let _mortality_rng = self.rng_factory.stream("demography/mortality");
        let _fertility_rng = self.rng_factory.stream("demography/fertility");
        let _parentage_rng = self.rng_factory.stream("demography/parentage");

        self.time = SimTime::from_years(self.config.duration_years);

        RunManifest {
            schema_version: RunManifest::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            experiment: self.config,
            world: self.world.summary(),
            population: self.population.summary(),
            start_time: SimTime::ZERO,
            end_time: self.time,
            stop_reason: StopReason::DurationReached,
        }
    }
}

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("experiment schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedExperimentSchema { found: u32, supported: u32 },
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_reaches_configured_duration() {
        let manifest = Simulation::new(ExperimentConfig::new(7, 10_000))
            .unwrap()
            .run();
        assert_eq!(manifest.end_time, SimTime::from_years(10_000));
        assert_eq!(manifest.stop_reason, StopReason::DurationReached);
        assert_eq!(manifest.world.cell_count, 128 * 128);
        assert_eq!(manifest.population.initial_population, 10_000);
        assert_eq!(manifest.population.living_population, 10_000);
    }

    #[test]
    fn rejects_unsupported_experiment_schema() {
        let mut config = ExperimentConfig::new(7, 100);
        config.schema_version = 999;
        assert!(matches!(
            Simulation::new(config),
            Err(SimulationError::UnsupportedExperimentSchema { .. })
        ));
    }
}
