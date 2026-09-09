use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig,
    MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const SOURCE_HOUSEHOLD: HouseholdId = HouseholdId::new(1);
const WEST_PARENT_HOUSEHOLD: HouseholdId = HouseholdId::new(2);
const EAST_PARENT_HOUSEHOLD: HouseholdId = HouseholdId::new(3);
const WEST_PARENT: PersonId = PersonId::new(5);
const EAST_PARENT: PersonId = PersonId::new(6);
const DAYS_PER_YEAR: i64 = 365;

fn neutral_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    // The external mothers were age 25 at the source adults' births, so retain positive support
    // only for that founder-validation chronology while leaving every living founder ineligible
    // for model-period fertility.
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 30, 1),
        AgeProbabilityBand::new(30, u32::MAX, 0),
    ];
    config
}

fn neutral_resources() -> ResourceConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    resources
}

fn source_adult(id: PersonId, female_parent: Option<PersonId>) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(30 * DAYS_PER_YEAR),
        reproductive_sex: ReproductiveSex::Male,
        household: SOURCE_HOUSEHOLD,
        female_parent,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn external_mother(id: PersonId, household: HouseholdId) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(55 * DAYS_PER_YEAR),
        reproductive_sex: ReproductiveSex::Female,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(swapped_linked_adult_ids: bool) -> (FounderPopulationDefinition, PersonId, PersonId) {
    let west_linked = if swapped_linked_adult_ids {
        PersonId::new(4)
    } else {
        PersonId::new(3)
    };
    let east_linked = if swapped_linked_adult_ids {
        PersonId::new(3)
    } else {
        PersonId::new(4)
    };

    let parent_for = |id: PersonId| {
        if id == west_linked {
            Some(WEST_PARENT)
        } else if id == east_linked {
            Some(EAST_PARENT)
        } else {
            None
        }
    };

    (
        FounderPopulationDefinition::new(
            if swapped_linked_adult_ids {
                "audit-v6-area-c-external-kin-relabel-swapped"
            } else {
                "audit-v6-area-c-external-kin-relabel-baseline"
            },
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![
                FounderHousehold {
                    id: SOURCE_HOUSEHOLD,
                    location: CellId::new(2),
                },
                FounderHousehold {
                    id: WEST_PARENT_HOUSEHOLD,
                    location: CellId::new(1),
                },
                FounderHousehold {
                    id: EAST_PARENT_HOUSEHOLD,
                    location: CellId::new(3),
                },
            ],
            vec![
                source_adult(PersonId::new(1), None),
                source_adult(PersonId::new(2), None),
                source_adult(PersonId::new(3), parent_for(PersonId::new(3))),
                source_adult(PersonId::new(4), parent_for(PersonId::new(4))),
                external_mother(WEST_PARENT, WEST_PARENT_HOUSEHOLD),
                external_mother(EAST_PARENT, EAST_PARENT_HOUSEHOLD),
            ],
        ),
        west_linked,
        east_linked,
    )
}

fn fission_outcome(swapped_linked_adult_ids: bool) -> (bool, bool, Vec<PersonId>) {
    let (founders, west_linked, east_linked) = founders(swapped_linked_adult_ids);
    let config = ExperimentConfig::new(6_301, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(6)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders)
        .with_demography(neutral_demography())
        .with_resources(neutral_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();

    let fissions = run
        .events()
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::HouseholdFission {
                source_household,
                residence,
                people_reassigned,
                ..
            } if *source_household == SOURCE_HOUSEHOLD => {
                assert_eq!(record.day, 365);
                assert_eq!(*residence, CellId::new(2));
                Some(people_reassigned.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        fissions.len(),
        1,
        "the source household must fission exactly once"
    );
    let moved = fissions.into_iter().next().unwrap();
    assert_eq!(moved.len(), 2, "four adults with target two must split 2+2");

    let west_moved = moved.contains(&west_linked);
    let east_moved = moved.contains(&east_linked);
    assert_ne!(
        west_moved, east_moved,
        "exactly one of the two cross-household-kin adults should enter the daughter household"
    );
    (west_moved, east_moved, moved)
}

#[test]
fn external_kin_context_is_not_reassigned_by_canonical_person_id() {
    let baseline = fission_outcome(false);
    let relabelled = fission_outcome(true);

    println!(
        "external-kin fission controls: baseline west_moved={} east_moved={} moved={:?}; relabelled west_moved={} east_moved={} moved={:?}",
        baseline.0, baseline.1, baseline.2, relabelled.0, relabelled.1, relabelled.2
    );

    assert_eq!(
        (baseline.0, baseline.1),
        (relabelled.0, relabelled.1),
        "pure canonical PersonId relabelling must not decide whether the adult linked to the west versus east external living parent retains source-household continuity"
    );
}