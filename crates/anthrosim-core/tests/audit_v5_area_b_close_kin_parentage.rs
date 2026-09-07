use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const ONE_MILLION: u32 = 1_000_000;

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    // Only the 20-year-old daughter is fertility-eligible at the model-year start. Her 40-year-old
    // mother remains in the genealogy but cannot independently produce a same-boundary birth.
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 25, ONE_MILLION),
        AgeProbabilityBand::new(25, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-b-close-kin-parentage",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(40 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: Some(-(20 * 365)),
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(40 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: -(20 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: Some(PersonId::new(1)),
                male_parent: Some(PersonId::new(2)),
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

#[test]
fn complete_genealogy_does_not_prevent_father_from_being_selected_for_daughters_child() {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(50_004, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();

    let births = run
        .events()
        .events
        .iter()
        .filter_map(|record| {
            if let EventKind::Birth {
                person,
                female_parent,
                male_parent,
                ..
            } = record.event
            {
                Some((record.day, person, female_parent, male_parent))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    println!("births={births:?}");
    assert_eq!(births.len(), 1, "the certain-fertility daughter should expose exactly one parentage decision");
    assert_eq!(births[0].0, 365);
    assert_eq!(births[0].2, PersonId::new(3));
    assert_eq!(
        births[0].3,
        PersonId::new(2),
        "with her living father as the only local eligible male, v33 selects that father again as the daughter's child's male parent"
    );

    // The adversary is evidence-oriented: success means the executable model does allow the
    // first-degree pairing. Classification is performed by the audit after checking whether that
    // behavior is explicitly scoped/documented; this test does not assert a repair policy.
}
