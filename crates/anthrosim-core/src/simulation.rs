use thiserror::Error;

use crate::{
    config::ExperimentConfig,
    demography::{
        DemographyConfigError, DemographyStepOutcome, process_demographic_year,
        validate_demography_config,
    },
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
        validate_demography_config(&config.demography)?;

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

    /// Run the configured M2 demographic lifecycle.
    ///
    /// v0.1 demographic events are evaluated at explicit annual boundaries.
    /// Population extinction and the persistent-record safety ceiling are
    /// operational stop conditions and are recorded distinctly from the
    /// requested-duration stop.
    pub fn run(mut self) -> Result<RunManifest, SimulationError> {
        let mut mortality_rng = self.rng_factory.stream("demography/mortality");
        let mut fertility_rng = self.rng_factory.stream("demography/fertility");
        let mut parentage_rng = self.rng_factory.stream("demography/parentage");
        let mut newborn_sex_rng = self.rng_factory.stream("demography/newborn_sex");

        let mut stop_reason = StopReason::DurationReached;
        if self.population.living_count() == 0 {
            stop_reason = StopReason::PopulationExtinct;
        } else {
            for year in 1..=self.config.duration_years {
                self.time = SimTime::from_years(year);
                let outcome = process_demographic_year(
                    &mut self.population,
                    &self.world,
                    &self.config.demography,
                    self.time.days(),
                    &mut mortality_rng,
                    &mut fertility_rng,
                    &mut parentage_rng,
                    &mut newborn_sex_rng,
                )?;

                match outcome {
                    DemographyStepOutcome::Continue => {}
                    DemographyStepOutcome::PopulationExtinct => {
                        stop_reason = StopReason::PopulationExtinct;
                        break;
                    }
                    DemographyStepOutcome::PersonRecordLimitReached => {
                        stop_reason = StopReason::PersonRecordLimitReached;
                        break;
                    }
                }
            }
        }

        self.population
            .validate(&self.world)
            .map_err(PopulationError::from)?;

        Ok(RunManifest {
            schema_version: RunManifest::CURRENT_SCHEMA_VERSION,
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
            experiment: self.config,
            world: self.world.summary(),
            population: self.population.summary(),
            start_time: SimTime::ZERO,
            end_time: self.time,
            stop_reason,
        })
    }
}

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("experiment schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedExperimentSchema { found: u32, supported: u32 },
    #[error(transparent)]
    Demography(#[from] DemographyConfigError),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DemographyConfig, PROBABILITY_PER_MILLION, PopulationConfig, WorldConfig};

    fn no_event_demography() -> DemographyConfig {
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }
        config
    }

    #[test]
    fn run_reaches_configured_duration_when_no_stop_condition_occurs() {
        let config = ExperimentConfig::new(7, 10).with_demography(no_event_demography());
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.end_time, SimTime::from_years(10));
        assert_eq!(manifest.stop_reason, StopReason::DurationReached);
        assert_eq!(manifest.world.cell_count, 128 * 128);
        assert_eq!(manifest.population.initial_population, 10_000);
        assert_eq!(manifest.population.living_population, 10_000);
        assert_eq!(manifest.population.births_since_start, 0);
        assert_eq!(manifest.population.deaths_since_start, 0);
    }

    #[test]
    fn default_schedule_produces_births_and_deaths() {
        let config = ExperimentConfig::new(81, 5)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(2_000).with_max_person_records(100_000));
        let manifest = Simulation::new(config).unwrap().run().unwrap();

        assert!(manifest.population.births_since_start > 0);
        assert!(manifest.population.deaths_since_start > 0);
        assert_eq!(
            u64::from(manifest.population.initial_population)
                + manifest.population.births_since_start
                - manifest.population.deaths_since_start,
            manifest.population.living_population
        );
    }

    #[test]
    fn certain_mortality_records_population_extinction() {
        let mut demography = no_event_demography();
        for band in &mut demography.mortality_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        let config = ExperimentConfig::new(91, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100))
            .with_demography(demography);

        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.population.living_population, 0);
        assert_eq!(manifest.population.deaths_since_start, 100);
    }

    #[test]
    fn record_limit_is_an_explicit_operational_stop() {
        let mut demography = no_event_demography();
        for band in &mut demography.fertility_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        demography.minimum_birth_spacing_days = 0;
        demography.male_parent_min_age_years = 0;
        demography.male_parent_max_age_years_exclusive = 100;

        let config = ExperimentConfig::new(101, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100).with_max_person_records(101))
            .with_demography(demography);
        let manifest = Simulation::new(config).unwrap().run().unwrap();

        assert_eq!(manifest.stop_reason, StopReason::PersonRecordLimitReached);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.population.person_records, 101);
        assert_eq!(manifest.population.births_since_start, 1);
    }

    #[test]
    fn empty_initial_population_is_extinct_at_epoch() {
        let config = ExperimentConfig::new(111, 10).with_population(PopulationConfig::new(0));
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::ZERO);
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
