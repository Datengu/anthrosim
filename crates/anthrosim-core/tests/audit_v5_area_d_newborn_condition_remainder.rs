use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    validate_recorded_run_invariants,
};

const DAYS_PER_YEAR: i64 = 365;

fn certain_birth_no_mortality() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-d-newborn-condition-remainder",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * DAYS_PER_YEAR),
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

fn mild_partial_supply_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 200;
    config.annual_regeneration_units_per_productivity = 1;
    config.initial_stock_units_per_productivity = 0;
    config.productivity_scale_permille = 1_000;
    config.seasonality_scale_permille = 0;
    config.cell_stock_capacity_years = 10;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 1;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

#[test]
fn newborn_inherits_visible_maternal_condition_but_not_latent_m3_remainder() {
    let config = ExperimentConfig::new(50_403, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(2).with_target_household_size(2))
        .with_founder_population(founders())
        .with_demography(certain_birth_no_mortality())
        .with_resources(mild_partial_supply_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&run).unwrap();

    let periods = run.checkpoint.resources.period_observations();
    assert_eq!(periods.len(), 1);
    let period = &periods[0];
    assert_eq!(period.total_need, 400);
    assert_eq!(period.supplied, 350);
    assert_eq!(period.unmet, 50);

    // 350/400 = 875 permille supply, so deficit=125 permille. With a reference-quarter
    // max loss of 1, a complete year contributes an exact maximum-loss budget of 4 points:
    // 125 * 4 = 500 thousandths. No whole point materializes, so both adults remain at
    // visible condition 1000 while carrying a latent deterioration remainder of 500.
    assert_eq!(
        run.checkpoint.population.person(PersonId::new(1)).unwrap().condition_permille,
        1_000
    );
    assert_eq!(
        run.checkpoint.population.person(PersonId::new(2)).unwrap().condition_permille,
        1_000
    );

    let births = run
        .checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::Birth {
                person,
                female_parent,
                male_parent,
                ..
            } => Some((record.day, *person, *female_parent, *male_parent)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        births,
        vec![(365, PersonId::new(3), PersonId::new(1), PersonId::new(2))]
    );
    assert_eq!(
        run.checkpoint.population.person(PersonId::new(3)).unwrap().condition_permille,
        1_000
    );

    let population_json = serde_json::to_value(&run.checkpoint.population).unwrap();
    let remainders = population_json["conditionLossRemainderThousandths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap())
        .collect::<Vec<_>>();

    eprintln!(
        "resource=(need={}, supplied={}, unmet={}); births={births:?}; remainders={remainders:?}",
        period.total_need, period.supplied, period.unmet
    );

    assert_eq!(
        remainders,
        vec![500, 500, 0],
        "v20 declares the maternal latent M3 deterioration remainder non-heritable: the newborn inherits visible integer condition only and starts with zero remainder"
    );
}
