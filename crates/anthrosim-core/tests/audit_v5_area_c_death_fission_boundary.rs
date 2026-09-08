use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, HouseholdLifecycleConfig, MigrationConfig,
    PopulationConfig, ResourceConfig, Simulation, WorldConfig, validate_recorded_run_invariants,
    config::{AgeProbabilityBand, ParameterProvenance},
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
};

const DAYS_PER_YEAR: i64 = 365;

fn founder(
    id: u64,
    age_years: i64,
    sex: ReproductiveSex,
    female_parent: Option<u64>,
    male_parent: Option<u64>,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -age_years * DAYS_PER_YEAR,
        reproductive_sex: sex,
        household: HouseholdId::new(1),
        female_parent: female_parent.map(PersonId::new),
        male_parent: male_parent.map(PersonId::new),
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn boundary_mortality() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.mortality_bands = vec![
        AgeProbabilityBand::new(0, 45, 0),
        AgeProbabilityBand::new(45, u32::MAX, 1_000_000),
    ];
    config
}

fn one_period_no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

#[test]
fn parent_dying_on_annual_boundary_cannot_anchor_same_day_household_fission() {
    let founders = FounderPopulationDefinition::new(
        "audit-v5-area-c-death-fission-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            // Oldest surviving independent anchor below the certain-mortality threshold -> group 0.
            founder(1, 44, ReproductiveSex::Male, None, None),
            // Child's mother; certain to die at the day-365 M3 mortality boundary.
            founder(2, 50, ReproductiveSex::Female, None, None),
            // Surviving father -> group 1 after the mother's death.
            founder(3, 40, ReproductiveSex::Male, None, None),
            // Dependent child must follow the surviving father, not the dead mother/source identity.
            founder(4, 10, ReproductiveSex::Female, Some(2), Some(3)),
        ],
    );

    let config = ExperimentConfig::new(50_402, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(4).with_target_household_size(4))
        .with_founder_population(founders)
        .with_demography(boundary_mortality())
        .with_resources(one_period_no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();

    let deaths = run
        .checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::Death { person, .. } => Some((record.day, *person)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let fissions = run
        .checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::HouseholdFission {
                source_household,
                new_household,
                people_reassigned,
                ..
            } => Some((
                record.day,
                *source_household,
                *new_household,
                people_reassigned.clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    eprintln!("deaths={deaths:?}; fissions={fissions:?}");

    assert_eq!(deaths, vec![(365, PersonId::new(2))]);
    assert_eq!(fissions.len(), 1, "expected exactly one same-day fission");
    assert_eq!(fissions[0].0, 365);
    assert_eq!(fissions[0].1, HouseholdId::new(1));
    assert_eq!(fissions[0].2, HouseholdId::new(2));
    assert_eq!(
        fissions[0].3,
        vec![PersonId::new(3), PersonId::new(4)],
        "the child must be reassigned with the surviving father; a parent killed earlier on the same boundary must not remain a dependency anchor"
    );
}
