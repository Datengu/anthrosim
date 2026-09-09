use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const DAYS_PER_YEAR: i64 = 365;
const MOTHER: PersonId = PersonId::new(1);

fn founders() -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    FounderPopulationDefinition::new(
        "audit-v6-area-g-founder-history-supersession",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: MOTHER,
                birth_day: -(25 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Female,
                household,
                female_parent: None,
                male_parent: None,
                // Valid pre-run reproductive history. The first model-period birth must supersede
                // this value for all later spacing decisions, including after checkpoint/resume.
                last_birth_day: Some(-1_000),
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 25, 1_000_000),
        AgeProbabilityBand::new(25, 35, 1_000_000),
        AgeProbabilityBand::new(35, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 500;
    config
}

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.seasonality_scale_permille = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn config() -> ExperimentConfig {
    ExperimentConfig::new(723_001, 3)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(2).with_max_person_records(10))
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn maternal_birth_days(checkpoint: &anthrosim_core::SimulationCheckpoint) -> Vec<u64> {
    checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::Birth { female_parent, .. } if *female_parent == MOTHER => Some(record.day),
            _ => None,
        })
        .collect()
}

#[test]
fn model_period_birth_supersedes_founder_history_exactly_across_checkpoint_resume() {
    let experiment = config();

    let uninterrupted = Simulation::new(experiment.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = Simulation::new(experiment)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();

    // The immutable founder definition must retain the pre-run evidence/provenance value, while
    // causal population state must now carry the first model-period birth at day 365.
    assert_eq!(
        checkpoint.population.person(MOTHER).unwrap().last_birth_day,
        Some(365)
    );
    assert_eq!(
        checkpoint
            .experiment
            .founder_population
            .as_ref()
            .unwrap()
            .person(MOTHER)
            .unwrap()
            .last_birth_day,
        Some(-1_000)
    );

    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();

    let uninterrupted_births = maternal_birth_days(&uninterrupted.checkpoint);
    let resumed_births = maternal_birth_days(&resumed.checkpoint);
    eprintln!(
        "uninterrupted_births={uninterrupted_births:?}; resumed_births={resumed_births:?}; final_last_birth={:?}",
        resumed.checkpoint.population.person(MOTHER).unwrap().last_birth_day
    );

    // Spacing is 500 days. The first certain model birth occurs at day 365, so day 730 must be
    // blocked by the model-period birth (365 elapsed days), and day 1095 must be eligible again
    // (730 elapsed days). Falling back to founder lastBirthDay=-1000 after resume would incorrectly
    // permit the day-730 birth.
    assert_eq!(uninterrupted_births, vec![365, 1_095]);
    assert_eq!(resumed_births, uninterrupted_births);
    assert_eq!(
        resumed.checkpoint.population.person(MOTHER).unwrap().last_birth_day,
        Some(1_095)
    );

    // Resume lineage is intentionally different, but all causal state and authoritative retained
    // histories must match uninterrupted execution exactly.
    assert_eq!(
        resumed.checkpoint.state_digest64,
        uninterrupted.checkpoint.state_digest64
    );
    assert_eq!(resumed.checkpoint.population, uninterrupted.checkpoint.population);
    assert_eq!(resumed.checkpoint.resources, uninterrupted.checkpoint.resources);
    assert_eq!(resumed.checkpoint.migration, uninterrupted.checkpoint.migration);
    assert_eq!(
        resumed.checkpoint.temporary_mobility,
        uninterrupted.checkpoint.temporary_mobility
    );
    assert_eq!(resumed.checkpoint.rng, uninterrupted.checkpoint.rng);
    assert_eq!(resumed.checkpoint.events, uninterrupted.checkpoint.events);
    assert_eq!(resumed.checkpoint.metrics, uninterrupted.checkpoint.metrics);
}
