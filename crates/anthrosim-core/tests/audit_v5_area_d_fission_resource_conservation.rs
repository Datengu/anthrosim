use anthrosim_core::{
    DemographyConfig, ExperimentConfig, HouseholdLifecycleConfig, MigrationConfig, PopulationConfig,
    ResourceConfig, Simulation, WorldConfig, config::ParameterProvenance,
    founder_initialization::{
        FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    },
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
    validate_recorded_run_invariants,
};

const DAYS_PER_YEAR: i64 = 365;

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-d-fission-resource-conservation",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        (0..4)
            .map(|index| FounderPerson {
                id: PersonId::new(index + 1),
                birth_day: -(30 + i64::try_from(index).unwrap()) * DAYS_PER_YEAR,
                reproductive_sex: if index % 2 == 0 {
                    ReproductiveSex::Female
                } else {
                    ReproductiveSex::Male
                },
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            })
            .collect(),
    )
}

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

fn resource_config() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 100_000;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config.seasonality_scale_permille = 0;
    config
}

fn config(with_fission: bool) -> ExperimentConfig {
    let mut config = ExperimentConfig::new(50_501, 2)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(4).with_target_household_size(4))
        .with_founder_population(founders())
        .with_demography(neutral_demography())
        .with_resources(resource_config())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    if with_fission {
        config = config.with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(2, 18),
        );
    }
    config
}

#[test]
fn same_cell_fission_preserves_next_period_cell_resource_accounting() {
    let fixed = Simulation::new(config(false)).unwrap().run_recorded().unwrap();
    let split = Simulation::new(config(true)).unwrap().run_recorded().unwrap();
    validate_recorded_run_invariants(&fixed).unwrap();
    validate_recorded_run_invariants(&split).unwrap();

    assert_eq!(fixed.checkpoint.population.household_count(), 1);
    assert_eq!(split.checkpoint.population.household_count(), 2);

    let fixed_periods = fixed.checkpoint.resources.period_observations();
    let split_periods = split.checkpoint.resources.period_observations();
    assert_eq!(fixed_periods.len(), 2);
    assert_eq!(split_periods.len(), 2);

    let a = &fixed_periods[1];
    let b = &split_periods[1];
    let a_cell = a.scarce_cells.first().expect("year 2 must be scarce");
    let b_cell = b.scarce_cells.first().expect("year 2 must be scarce");

    eprintln!(
        "year2 fixed=(stock_before={}, regenerated={}, need={}, supplied={}, unmet={}, stock_after={}); split=(stock_before={}, regenerated={}, need={}, supplied={}, unmet={}, stock_after={})",
        a.stock_before_regeneration,
        a.regenerated,
        a.total_need,
        a.supplied,
        a.unmet,
        a.stock_after_harvest,
        b.stock_before_regeneration,
        b.regenerated,
        b.total_need,
        b.supplied,
        b.unmet,
        b.stock_after_harvest,
    );

    assert_eq!(a.stock_before_regeneration, b.stock_before_regeneration);
    assert_eq!(a.regenerated, b.regenerated);
    assert_eq!(a.stock_after_regeneration, b.stock_after_regeneration);
    assert_eq!(a.home_need, b.home_need);
    assert_eq!(a.visitor_need, b.visitor_need);
    assert_eq!(a.total_need, b.total_need);
    assert_eq!(a.supplied, b.supplied);
    assert_eq!(a.unmet, b.unmet);
    assert_eq!(a.stock_after_harvest, b.stock_after_harvest);

    assert_eq!(a_cell.stock_before_regeneration, b_cell.stock_before_regeneration);
    assert_eq!(a_cell.regenerated, b_cell.regenerated);
    assert_eq!(a_cell.stock_after_regeneration, b_cell.stock_after_regeneration);
    assert_eq!(a_cell.home_need, b_cell.home_need);
    assert_eq!(a_cell.visitor_need, b_cell.visitor_need);
    assert_eq!(a_cell.total_need, b_cell.total_need);
    assert_eq!(a_cell.supplied, b_cell.supplied);
    assert_eq!(a_cell.unmet, b_cell.unmet);
    assert_eq!(a_cell.stock_after_harvest, b_cell.stock_after_harvest);

    let fixed_summary = fixed.checkpoint.resources.summary(&fixed.checkpoint.population);
    let split_summary = split.checkpoint.resources.summary(&split.checkpoint.population);
    assert_eq!(fixed_summary.initial_food_stock, split_summary.initial_food_stock);
    assert_eq!(fixed_summary.regenerated_food, split_summary.regenerated_food);
    assert_eq!(fixed_summary.harvested_food, split_summary.harvested_food);
    assert_eq!(fixed_summary.unmet_need, split_summary.unmet_need);
    assert_eq!(fixed_summary.final_food_stock, split_summary.final_food_stock);
}
