use crate::{
    config::ExperimentConfig,
    manifest::{RunManifest, StopReason},
    rng::RngFactory,
    time::SimTime,
};

/// Authoritative headless simulation host.
///
/// Milestone 0 contains no human/environment systems yet. It establishes the
/// deterministic lifecycle that those systems will run inside.
#[derive(Debug)]
pub struct Simulation {
    config: ExperimentConfig,
    time: SimTime,
    rng_factory: RngFactory,
}

impl Simulation {
    #[must_use]
    pub fn new(config: ExperimentConfig) -> Self {
        Self {
            rng_factory: RngFactory::new(config.seed),
            config,
            time: SimTime::ZERO,
        }
    }

    #[must_use]
    pub const fn time(&self) -> SimTime {
        self.time
    }

    #[must_use]
    pub const fn config(&self) -> &ExperimentConfig {
        &self.config
    }

    /// Run the empty Milestone 0 lifecycle to the configured duration.
    #[must_use]
    pub fn run(mut self) -> RunManifest {
        // Touch the named factory in the skeleton so future systems inherit an
        // explicit source of randomness rather than introducing ambient RNG.
        let _world_rng = self.rng_factory.stream("world");

        self.time = SimTime::from_years(self.config.duration_years);

        RunManifest {
            schema_version: RunManifest::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            experiment: self.config,
            start_time: SimTime::ZERO,
            end_time: self.time,
            stop_reason: StopReason::DurationReached,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_reaches_configured_duration() {
        let manifest = Simulation::new(ExperimentConfig::new(7, 10_000)).run();
        assert_eq!(manifest.end_time, SimTime::from_years(10_000));
        assert_eq!(manifest.stop_reason, StopReason::DurationReached);
    }
}
