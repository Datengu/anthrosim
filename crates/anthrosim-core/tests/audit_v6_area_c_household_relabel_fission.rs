use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const WEST: CellId = CellId::new(1);
const EAST: CellId = CellId::new(3);
const H1: HouseholdId = HouseholdId::new(1);
const H2: HouseholdId = HouseholdId::new(2);
const DAYS_PER_YEAR: i64 = 365;

fn neutral_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn neutral_resources() -> ResourceConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    resources
}

fn adult(id: u64, age_years: i64, household: HouseholdId) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(age_years * DAYS_PER_YEAR),
        reproductive_sex: ReproductiveSex::Male,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(swapped_household_ids: bool) -> FounderPopulationDefinition {
    let west_household = if swapped_household_ids { H2 } else { H1 };
    let east_household = if swapped_household_ids { H1 } else { H2 };

    FounderPopulationDefinition::new(
        if swapped_household_ids {
            "audit-v6-area-c-household-relabel-swapped"
        } else {
            "audit-v6-area-c-household-relabel-baseline"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: west_household,
                location: WEST,
            },
            FounderHousehold {
                id: east_household,
                location: EAST,
            },
        ],
        vec![
            adult(1, 60, west_household),
            adult(2, 50, west_household),
            adult(3, 40, west_household),
            adult(4, 30, west_household),
            adult(5, 65, east_household),
            adult(6, 55, east_household),
            adult(7, 45, east_household),
            adult(8, 35, east_household),
            adult(9, 25, east_household),
            adult(10, 20, east_household),
        ],
    )
}

fn fission_signature(swapped_household_ids: bool) -> Vec<(CellId, Vec<PersonId>)> {
    let config = ExperimentConfig::new(6_302, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(10)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(12),
        )
        .with_founder_population(founders(swapped_household_ids))
        .with_demography(neutral_demography())
        .with_resources(neutral_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();

    let mut fissions = run
        .events()
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::HouseholdFission {
                residence,
                people_reassigned,
                ..
            } => {
                assert_eq!(record.day, 365);
                let mut moved = people_reassigned.clone();
                moved.sort_by_key(|person| person.0);
                Some((*residence, moved))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    fissions.sort_by_key(|(residence, moved)| {
        (
            residence.0,
            moved.iter().map(|person| person.0).collect::<Vec<_>>(),
        )
    });

    assert_eq!(
        fissions.len(),
        3,
        "4+6 adults at target size 2 must create three daughter households"
    );
    assert_eq!(
        fissions
            .iter()
            .filter(|(residence, _)| *residence == WEST)
            .count(),
        1,
        "the four-person west source must create one daughter"
    );
    assert_eq!(
        fissions
            .iter()
            .filter(|(residence, _)| *residence == EAST)
            .count(),
        2,
        "the six-person east source must create two daughters"
    );

    fissions
}

#[test]
fn simultaneous_fission_is_invariant_to_source_household_id_relabelling() {
    let baseline = fission_signature(false);
    let relabelled = fission_signature(true);

    println!(
        "household-relabel fission controls: baseline={baseline:?}; relabelled={relabelled:?}"
    );

    assert_eq!(
        baseline, relabelled,
        "pure canonical HouseholdId relabelling must not change the physical/member partition produced by simultaneous heterogeneous household fissions"
    );
}
