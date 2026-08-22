use thiserror::Error;

use crate::{
    config::ExperimentConfig,
    demography::{
        DemographyConfigError, DemographyRngs, DemographyStepOutcome, process_demographic_year,
        validate_demography_config,
    },
    manifest::{RunManifest, StopReason},
    migration::{
        MigrationConfigError, MigrationError, MigrationRngs, MigrationSystem,
        validate_migration_config,
    },
    population::{Population, PopulationError},
    resources::{
        ResourceConfigError, ResourceError, ResourceRngs, ResourceStepOutcome, ResourceSystem,
        validate_resource_config,
    },
    rng::RngFactory,
    time::{DAYS_PER_YEAR, SimTime},
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
    resources: ResourceSystem,
    migration: MigrationSystem,
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
        validate_resource_config(&config.resources)?;
        validate_migration_config(&config.migration)?;

        let rng_factory = RngFactory::new(config.seed);
        let world = World::generate(config.world, rng_factory)?;
        let population = Population::initialize(config.population, &world, rng_factory)?;
        let resources = ResourceSystem::initialize(&world, &config.resources)?;
        let migration = MigrationSystem::initialize(&population, &world, &config.migration)?;

        Ok(Self {
            rng_factory,
            config,
            time: SimTime::ZERO,
            world,
            population,
            resources,
            migration,
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

    #[must_use]
    pub const fn resources(&self) -> &ResourceSystem {
        &self.resources
    }

    #[must_use]
    pub const fn migration(&self) -> &MigrationSystem {
        &self.migration
    }

    /// Run the configured M4 resource-migration-demographic lifecycle.
    ///
    /// Each resource period first regenerates and shares local resources, then
    /// updates condition and scarcity mortality. Surviving households evaluate
    /// bounded local migration alternatives against that experienced state and
    /// selected moves complete atomically at the same decision boundary with an
    /// explicit travel condition cost. The annual demographic boundary then
    /// evaluates baseline mortality and fertility.
    pub fn run(mut self) -> Result<RunManifest, SimulationError> {
        let mut demography_rngs = DemographyRngs::new(self.rng_factory);
        let mut resource_rngs = ResourceRngs::new(self.rng_factory);
        let mut migration_rngs = MigrationRngs::new(self.rng_factory);

        let mut stop_reason = StopReason::DurationReached;
        if self.population.living_count() == 0 {
            stop_reason = StopReason::PopulationExtinct;
        } else {
            'years: for year in 1..=self.config.duration_years {
                let periods = u64::from(self.config.resources.periods_per_year);
                let year_start_day = (year - 1).saturating_mul(DAYS_PER_YEAR);

                for period_index in 0..self.config.resources.periods_per_year {
                    let period_number = u64::from(period_index) + 1;
                    let day = year_start_day
                        .saturating_add(period_number.saturating_mul(DAYS_PER_YEAR) / periods);
                    self.time = SimTime::from_days(day);
                    let outcome = self.resources.process_period(
                        &mut self.population,
                        &self.world,
                        &self.config.resources,
                        period_index,
                        day,
                        &mut resource_rngs.scarcity_mortality,
                    )?;
                    if outcome == ResourceStepOutcome::PopulationExtinct {
                        stop_reason = StopReason::PopulationExtinct;
                        break 'years;
                    }
                    self.migration.process_boundary(
                        &mut self.population,
                        &self.world,
                        &self.resources,
                        &self.config.migration,
                        self.config.resources.annual_need_units_per_person,
                        self.config.resources.periods_per_year,
                        day,
                        &mut migration_rngs,
                    )?;
                }

                self.time = SimTime::from_years(year);
                let outcome = process_demographic_year(
                    &mut self.population,
                    &self.world,
                    &self.config.demography,
                    self.time.days(),
                    &mut demography_rngs,
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
            resources: self.resources.summary(&self.population),
            migration: self.migration.summary(),
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
    ResourceConfig(#[from] ResourceConfigError),
    #[error(transparent)]
    MigrationConfig(#[from] MigrationConfigError),
    #[error(transparent)]
    Resources(#[from] ResourceError),
    #[error(transparent)]
    Migration(#[from] MigrationError),
    #[error(transparent)]
    World(#[from] WorldError),
    #[error(transparent)]
    Population(#[from] PopulationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DemographyConfig, MigrationConfig, PROBABILITY_PER_MILLION, PopulationConfig,
        ResourceConfig, WorldConfig,
    };

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

    fn no_pressure_resources() -> ResourceConfig {
        let mut config = ResourceConfig::synthetic_validation_v1();
        config.annual_need_units_per_person = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        config
    }

    fn disabled_migration() -> MigrationConfig {
        MigrationConfig::synthetic_validation_v1().with_enabled(false)
    }

    #[test]
    fn run_reaches_configured_duration_when_no_stop_condition_occurs() {
        let config = ExperimentConfig::new(7, 10)
            .with_demography(no_event_demography())
            .with_resources(no_pressure_resources());
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.end_time, SimTime::from_years(10));
        assert_eq!(manifest.stop_reason, StopReason::DurationReached);
        assert_eq!(manifest.world.cell_count, 128 * 128);
        assert_eq!(manifest.population.initial_population, 10_000);
        assert_eq!(manifest.population.living_population, 10_000);
        assert_eq!(manifest.population.births_since_start, 0);
        assert_eq!(manifest.population.deaths_since_start, 0);
        assert_eq!(manifest.resources.periods_processed, 40);
        assert_eq!(manifest.resources.unmet_need, 0);
        assert_eq!(manifest.migration.moves_completed, 0);
    }

    #[test]
    fn default_schedule_produces_resource_demographic_and_migration_state() {
        let config = ExperimentConfig::new(81, 5)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(2_000).with_max_person_records(100_000));
        let manifest = Simulation::new(config).unwrap().run().unwrap();

        assert!(manifest.population.deaths_since_start > 0);
        assert!(manifest.resources.periods_processed > 0);
        assert!(manifest.resources.harvested_food > 0 || manifest.resources.unmet_need > 0);
        assert!(manifest.migration.decision_boundaries > 0);
        assert_eq!(
            u64::from(manifest.population.initial_population)
                + manifest.population.births_since_start
                - manifest.population.deaths_since_start,
            manifest.population.living_population
        );
    }

    #[test]
    fn certain_demographic_mortality_records_population_extinction() {
        let mut demography = no_event_demography();
        for band in &mut demography.mortality_bands {
            band.annual_probability_per_million = PROBABILITY_PER_MILLION;
        }
        let config = ExperimentConfig::new(91, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100))
            .with_demography(demography)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration());

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
            .with_demography(demography)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration());
        let manifest = Simulation::new(config).unwrap().run().unwrap();

        assert_eq!(manifest.stop_reason, StopReason::PersonRecordLimitReached);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.population.person_records, 101);
        assert_eq!(manifest.population.births_since_start, 1);
    }

    #[test]
    fn severe_resource_scarcity_can_extinguish_before_migration_and_annual_demography() {
        let mut resources = ResourceConfig::synthetic_validation_v1()
            .with_productivity_scale_permille(0)
            .with_annual_need_units_per_person(100);
        resources.periods_per_year = 1;
        resources.max_condition_loss_per_period = 1_000;
        resources.max_scarcity_mortality_probability_per_million = PROBABILITY_PER_MILLION;
        let config = ExperimentConfig::new(111, 10)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100))
            .with_demography(no_event_demography())
            .with_resources(resources);

        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::from_years(1));
        assert_eq!(manifest.resources.scarcity_deaths, 100);
        assert!(manifest.resources.unmet_need > 0);
        assert_eq!(manifest.migration.moves_completed, 0);
    }

    #[test]
    fn empty_initial_population_is_extinct_at_epoch() {
        let config = ExperimentConfig::new(121, 10).with_population(PopulationConfig::new(0));
        let manifest = Simulation::new(config).unwrap().run().unwrap();
        assert_eq!(manifest.stop_reason, StopReason::PopulationExtinct);
        assert_eq!(manifest.end_time, SimTime::ZERO);
        assert_eq!(manifest.resources.periods_processed, 0);
        assert_eq!(manifest.migration.decision_boundaries, 0);
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
