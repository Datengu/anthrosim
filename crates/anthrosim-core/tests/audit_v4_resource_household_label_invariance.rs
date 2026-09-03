use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, MigrationConfig, ParameterProvenance, PopulationConfig,
    ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const SEED: u64 = 78_101;

fn quiet_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn resource_config(annual_need: u32) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = annual_need;
    config.annual_regeneration_units_per_productivity = 0;
    config.seasonality_scale_permille = 0;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 200;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn initial_stock() -> u64 {
    let config = ExperimentConfig::new(SEED, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(1).with_max_person_records(10))
        .with_demography(quiet_demography())
        .with_resources(resource_config(0))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .resources
        .initial_food_stock
}

fn founder_definition(household_count: u64, rotate_household_labels: bool) -> FounderPopulationDefinition {
    let households = (1..=household_count)
        .map(|id| FounderHousehold {
            id: HouseholdId::new(id),
            location: CellId::new(1),
        })
        .collect::<Vec<_>>();
    let people = (1..=household_count)
        .map(|id| {
            let household_id = if rotate_household_labels {
                id % household_count + 1
            } else {
                id
            };
            FounderPerson {
                id: PersonId::new(id),
                birth_day: -(30 * 365),
                reproductive_sex: if id % 2 == 0 {
                    ReproductiveSex::Female
                } else {
                    ReproductiveSex::Male
                },
                household: HouseholdId::new(household_id),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            }
        })
        .collect::<Vec<_>>();

    FounderPopulationDefinition::new(
        if rotate_household_labels {
            "audit-v4-resource-label-rotated"
        } else {
            "audit-v4-resource-label-canonical"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unknown,
        households,
        people,
    )
}

fn conditions(household_count: u64, annual_need: u32, rotate: bool) -> Vec<u16> {
    let definition = founder_definition(household_count, rotate);
    let config = ExperimentConfig::new(SEED, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(u32::try_from(household_count).unwrap())
                .with_max_person_records(100),
        )
        .with_founder_population(definition)
        .with_demography(quiet_demography())
        .with_resources(resource_config(annual_need))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    (1..=household_count)
        .map(|id| {
            run.checkpoint
                .population
                .person(PersonId::new(id))
                .unwrap()
                .condition_permille
        })
        .collect()
}

#[test]
fn equal_resource_claims_are_invariant_to_pure_household_label_rotation() {
    let stock = initial_stock();
    assert!(stock > 0);

    let household_count = [2_u64, 3, 5, 7]
        .into_iter()
        .find(|count| stock % count != 0)
        .expect("one tested household count must leave a resource-allocation remainder");
    let annual_need = u32::try_from(stock / household_count + 1).unwrap();

    let canonical = conditions(household_count, annual_need, false);
    let rotated = conditions(household_count, annual_need, true);

    println!(
        "initial_stock={stock} households={household_count} annual_need={annual_need} canonical={canonical:?} rotated={rotated:?}"
    );

    assert_eq!(
        canonical, rotated,
        "pure relabelling of otherwise identical colocated households must not change which fixed physical people receive the largest-remainder resource units"
    );
}
