use anthrosim_core::ids::{CellId, HouseholdId, PersonId};
use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig,
};

fn certain_fertility_no_mortality() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config.minimum_birth_spacing_days = 0;
    config
}

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn founders(female_cell: CellId, male_cell: CellId) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v3-m2-temporary-presence-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: female_cell,
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: male_cell,
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(25 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn base_config(female_cell: CellId, male_cell: CellId) -> ExperimentConfig {
    ExperimentConfig::new(32_001, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(2).with_max_person_records(10))
        .with_founder_population(founders(female_cell, male_cell))
        .with_demography(certain_fertility_no_mortality())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

/// Fresh audit-v3 Area B limiting case. M9 temporary physical co-presence is deliberately excluded
/// from M2 parentage: a male who spends the fertility boundary visiting the female's cell must not
/// become a local parentage candidate when their persistent residences differ.
#[test]
fn temporary_physical_copresence_does_not_redefine_m2_parentage_locality() {
    let female_cell = CellId::new(1);
    let male_residence = CellId::new(2);
    let region = FocalRegion::new(
        "audit-v3-female-cell",
        FocalRegionSource::Synthetic,
        vec![female_cell],
    )
    .unwrap();
    let mobility = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            "audit-v3-full-year-visit",
            TemporaryTriggerTiming::DepartureDay,
            vec![0],
            400,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();

    let run =
        Simulation::new(base_config(female_cell, male_residence).with_temporary_mobility(mobility))
            .unwrap()
            .run_recorded()
            .unwrap();

    assert_eq!(
        run.checkpoint
            .temporary_mobility
            .current_cell(HouseholdId::new(2), &run.checkpoint.population),
        Some(female_cell),
        "male household must be physically visiting the female's cell at the annual boundary"
    );
    assert_eq!(
        run.checkpoint.population.summary().births_since_start,
        0,
        "M2 must use persistent-residence exposure rather than M9 physical co-presence"
    );
    assert!(run.events().events.iter().any(|record| {
        matches!(
            record.event,
            EventKind::TemporaryJourneyArrived {
                household,
                destination,
                ..
            } if household == HouseholdId::new(2) && destination == female_cell
        )
    }));
    run.validate_invariants().unwrap();
}

#[test]
fn persistent_copresence_allows_the_same_certain_fertility_opportunity() {
    let shared_cell = CellId::new(1);
    let run = Simulation::new(base_config(shared_cell, shared_cell))
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(run.checkpoint.population.summary().births_since_start, 1);
    run.validate_invariants().unwrap();
}
