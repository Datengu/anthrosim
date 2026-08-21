use thiserror::Error;

use crate::{
    config::ExperimentConfig,
    manifest::{RunManifest, StopReason},
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

        Ok(Self {
            rng_factory,
            config,
            time: SimTime::ZERO,
            world,
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

    /// Run the current simulation lifecycle to the configured duration.
    #[must_use]
    pub fn run(mut self) -> RunManifest {
        // Reserve a separate deterministic stream boundary for future
        // demography without coupling it to world generation draws.
        let _demography_rng = self.rng_factory.stream("demography");

        self.time = SimTime::from_years(self.config.duration_years);

        RunManifest {
            schema_version: RunManifest::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            experiment: self.config,
            world: self.world.summary(),
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
