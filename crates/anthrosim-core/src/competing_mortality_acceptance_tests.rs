use crate::{
    AgeProbabilityBand, DeathCause, DemographyConfig, EventKind, ExperimentConfig, MigrationConfig,
    ParameterProvenance, PopulationConfig, ResourceConfig, Simulation, WorldConfig,
    time::DAYS_PER_YEAR,
};

fn certain_background_demography() -> DemographyConfig {
    DemographyConfig {
        schema_version: DemographyConfig::CURRENT_SCHEMA_VERSION,
        schedule_id: "issue-208-certain-background".to_owned(),
        provenance: ParameterProvenance::SyntheticValidation,
        mortality_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 1_000_000)],
        fertility_bands: vec![AgeProbabilityBand::new(0, u32::MAX, 0)],
        minimum_birth_spacing_days: 0,
        male_birth_permille: 500,
        male_parent_min_age_years: 18,
        male_parent_max_age_years_exclusive: 70,
    }
}

#[test]
fn certain_background_death_is_resolved_once_before_same_day_m4() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let mut migration = MigrationConfig::synthetic_validation_v1();
    migration.enabled = true;
    migration.decision_periods_per_year = 1;

    let config = ExperimentConfig::new(208_001, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(certain_background_demography())
        .with_resources(resources)
        .with_migration(migration);
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();

    let deaths = recorded
        .checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::Death {
                cause,
                probability_per_million,
                ..
            } => Some((record.day, *cause, *probability_per_million)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        deaths,
        vec![(DAYS_PER_YEAR, DeathCause::DemographicMortality, 1_000_000)]
    );
    assert!(
        recorded
            .checkpoint
            .events
            .events
            .iter()
            .all(|record| !matches!(record.event, EventKind::HouseholdMigration { .. })),
        "the elapsed-interval death must be resolved before the coincident M4 opportunity"
    );
}

#[test]
fn certain_background_risk_is_partitioned_over_m3_boundaries_not_redrawn_at_year_end() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 4;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(208_002, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(certain_background_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();

    let background_deaths = recorded
        .checkpoint
        .events
        .events
        .iter()
        .filter(|record| {
            matches!(
                record.event,
                EventKind::Death {
                    cause: DeathCause::DemographicMortality,
                    ..
                }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(background_deaths.len(), 1);
    assert!(background_deaths[0].day > 0);
    assert!(background_deaths[0].day <= DAYS_PER_YEAR);
}

#[test]
fn dual_certain_causes_produce_one_authoritative_death_not_scheduler_priority_duplicates() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.productivity_scale_permille = 0;
    resources.annual_need_units_per_person = 1;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 1_000;
    resources.max_scarcity_mortality_probability_per_million = 1_000_000;

    let config = ExperimentConfig::new(208_003, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(certain_background_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();

    let deaths = recorded
        .checkpoint
        .events
        .events
        .iter()
        .filter(|record| matches!(record.event, EventKind::Death { .. }))
        .collect::<Vec<_>>();
    assert_eq!(deaths.len(), 1);
    assert_eq!(deaths[0].day, DAYS_PER_YEAR);
    let EventKind::Death {
        cause,
        probability_per_million,
        ..
    } = deaths[0].event
    else {
        unreachable!();
    };
    assert!(matches!(
        cause,
        DeathCause::DemographicMortality | DeathCause::ResourceScarcity
    ));
    assert_eq!(probability_per_million, 1_000_000);
}
