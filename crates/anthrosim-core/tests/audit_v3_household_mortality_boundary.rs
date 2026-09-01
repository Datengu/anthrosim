use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn boundary_mortality_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.mortality_bands = vec![
        AgeProbabilityBand::new(0, 75, 0),
        AgeProbabilityBand::new(75, u32::MAX, 1_000_000),
    ];
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

fn founder_definition() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v3-household-mortality-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(80 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: -(10 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: Some(PersonId::new(2)),
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(4),
                birth_day: -(8 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: Some(PersonId::new(2)),
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(5),
                birth_day: -(5 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: Some(PersonId::new(2)),
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

/// Fresh Audit-v3 Area C adversary: the household begins the model year above the lifecycle target
/// with two independent-age members, which would be enough to split. One anchor is guaranteed to
/// die during the year's competing-mortality exposure. Annual fission occurs after demographic
/// mortality and must therefore use the post-mortality living structure rather than a stale
/// pre-year household snapshot.
#[test]
fn same_year_mortality_removes_dead_fission_anchor_before_lifecycle_transition() {
    let config = ExperimentConfig::new(33_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(5).with_max_person_records(20))
        .with_founder_population(founder_definition())
        .with_demography(boundary_mortality_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(3, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();

    assert_eq!(run.checkpoint.population.living_count(), 4);
    assert_eq!(run.checkpoint.population.household_count(), 1);
    assert_eq!(
        run.events()
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::HouseholdFission { .. }))
            .count(),
        0,
        "a dead annual-boundary anchor must not make an otherwise dependent household fission"
    );
    assert!(run.events().events.iter().any(|record| {
        matches!(
            record.event,
            EventKind::Death {
                person,
                ..
            } if person == PersonId::new(1)
        )
    }));
    run.validate_invariants().unwrap();
}
