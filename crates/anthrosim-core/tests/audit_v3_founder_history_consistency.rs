use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    time::DAYS_PER_YEAR,
};

fn contradictory_founders() -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    FounderPopulationDefinition::new(
        "audit-v3-contradictory-last-birth-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Female,
                household,
                female_parent: None,
                male_parent: None,
                // Individually plausible at about age 24.5, but contradicted by the explicitly
                // declared child below, who is only 100 days old at the epoch.
                last_birth_day: Some(-2_000),
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: -100,
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: Some(PersonId::new(1)),
                male_parent: Some(PersonId::new(2)),
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn config() -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        if band.annual_probability_per_million > 0 {
            band.annual_probability_per_million = 1_000_000;
        }
    }

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(74_001, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(contradictory_founders())
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn declared_last_birth_cannot_predate_a_known_later_declared_child() {
    let run = Simulation::new(config())
        .expect("current initializer accepts the individually valid founder fields")
        .run_recorded()
        .expect("accepted contradictory founder state must execute deterministically");

    let births = run
        .events()
        .events
        .iter()
        .filter(|record| matches!(record.event, EventKind::Birth { .. }))
        .count();

    // The explicit child proves that the mother's most recent known birth is at least day -100.
    // At day 365 only 465 days have elapsed, below the default executable spacing of 1460 days.
    // A coherent founder state must therefore block a new first-boundary birth. If the stale
    // lastBirthDay=-2000 is trusted independently, the model instead records an artificial birth.
    assert_eq!(
        births, 0,
        "a declared lastBirthDay must not override a later explicitly declared child"
    );
}
