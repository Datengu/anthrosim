use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const FEMALE: PersonId = PersonId::new(1);
const MALE: PersonId = PersonId::new(2);
const CHILD: PersonId = PersonId::new(3);
const DAYS_PER_YEAR: i64 = 365;
const SHIFTED_CHILD_BIRTH_DAY: i64 = -1;

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 45, 1_000_000),
        AgeProbabilityBand::new(45, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn neutral_resources() -> ResourceConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    resources
}

fn dynamic_founders(male_age_days_at_interval_start: i64) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-b-dynamic-male-parent-age",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: FEMALE,
                birth_day: -(30 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: MALE,
                birth_day: -male_age_days_at_interval_start,
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn dynamic_birth_male_parent(male_age_days_at_interval_start: i64) -> Option<PersonId> {
    let config = ExperimentConfig::new(9_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(4),
        )
        .with_founder_population(dynamic_founders(male_age_days_at_interval_start))
        .with_demography(demography())
        .with_resources(neutral_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();
    run.events().events.iter().find_map(|record| {
        if let EventKind::Birth {
            person,
            female_parent,
            male_parent,
            ..
        } = record.event
            && person == CHILD
            && female_parent == FEMALE
        {
            Some(male_parent)
        } else {
            None
        }
    })
}

fn shifted_founder_relation_is_accepted(male_age_days_at_child_birth: i64) -> bool {
    let definition = FounderPopulationDefinition::new(
        "audit-v6-area-b-shifted-parent-age-relation",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: FEMALE,
                // Keep the comparison entirely pre-epoch while preserving exactly the dynamic
                // female's age 31 at the child birth. Day zero is reserved for model-time state.
                birth_day: SHIFTED_CHILD_BIRTH_DAY - 31 * DAYS_PER_YEAR,
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: MALE,
                // Translation by the same child-birth offset preserves the exact male age at the
                // parentage event while avoiding an epoch-boundary founder fixture artifact.
                birth_day: SHIFTED_CHILD_BIRTH_DAY - male_age_days_at_child_birth,
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: CHILD,
                birth_day: SHIFTED_CHILD_BIRTH_DAY,
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: Some(FEMALE),
                male_parent: Some(MALE),
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    );

    let config = ExperimentConfig::new(9_002, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(4),
        )
        .with_founder_population(definition)
        .with_demography(demography())
        .with_resources(neutral_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config).is_ok()
}

#[test]
fn dynamic_and_declared_genealogy_use_one_male_parent_age_time_reference() {
    let upper_start_age = 69 * DAYS_PER_YEAR + 200;
    let upper_birth_age = upper_start_age + DAYS_PER_YEAR;
    let lower_start_age = 17 * DAYS_PER_YEAR + 200;
    let lower_birth_age = lower_start_age + DAYS_PER_YEAR;

    let upper_dynamic = dynamic_birth_male_parent(upper_start_age);
    let upper_declared = shifted_founder_relation_is_accepted(upper_birth_age);
    let lower_dynamic = dynamic_birth_male_parent(lower_start_age);
    let lower_declared = shifted_founder_relation_is_accepted(lower_birth_age);

    println!(
        "male-parent age time-reference controls: upper dynamic={upper_dynamic:?}, upper shifted-founder accepted={upper_declared}, lower dynamic={lower_dynamic:?}, lower shifted-founder accepted={lower_declared}"
    );

    // Positive controls prove both sides of the boundary discrepancy are actually exercised.
    assert_eq!(
        upper_dynamic,
        Some(MALE),
        "a male aged 69y200d at interval start should exercise the current dynamic upper-bound path"
    );
    assert!(
        !upper_declared,
        "the same male is 70y200d at child birth and should exercise the founder validator's configured upper-bound rejection"
    );
    assert_eq!(
        lower_dynamic, None,
        "a male aged 17y200d at interval start should exercise the current dynamic lower-bound exclusion"
    );
    assert!(
        lower_declared,
        "the same male is 18y200d at child birth and should exercise the founder validator's configured lower-bound acceptance"
    );

    assert_eq!(
        upper_dynamic.is_some(),
        upper_declared,
        "the configured male-parent age window changes time reference between dynamic M2 parentage and declared founder genealogy at the upper boundary"
    );
    assert_eq!(
        lower_dynamic.is_some(),
        lower_declared,
        "the configured male-parent age window changes time reference between dynamic M2 parentage and declared founder genealogy at the lower boundary"
    );
}
