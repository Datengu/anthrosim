use anthrosim_core::ids::{CellId, HouseholdId, PersonId};
use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, MigrationConfig, ParameterProvenance, PopulationConfig,
    ReproductiveSex, ResourceConfig, Simulation, World, WorldConfig,
};

fn no_mortality_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn effective_annual_regeneration(base_productivity: u16, environmental_stress: u16) -> u64 {
    let unstressed = u64::from(base_productivity);
    let stress_factor = u64::from(1_000_u16.saturating_sub(environmental_stress));
    unstressed * stress_factor / 1_000
}

fn one_male_founder(origin: CellId) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v3-zero-recovery-remainder",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: origin,
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(50_i64 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        }],
    )
}

#[test]
fn zero_recovery_full_supply_must_not_erase_latent_partial_supply_deterioration() {
    let world_config = WorldConfig::new(8, 8);

    // Pick a deterministic generated world whose second-highest distinct annual effective
    // regeneration is at least four units. Starting there and asking for exactly one unit more
    // makes year 1 slightly under-supplied while every strictly better M4 destination can fully
    // provision the household after relocation.
    let (seed, origin, origin_regeneration) = (91_000_u64..91_100)
        .find_map(|seed| {
            let world = World::generate(world_config, RngFactory::new(seed)).ok()?;
            let mut cells = world
                .cells()
                .iter()
                .enumerate()
                .map(|(index, cell)| {
                    (
                        CellId::new(index as u64 + 1),
                        effective_annual_regeneration(
                            cell.base_productivity,
                            cell.environmental_stress,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            cells.sort_unstable_by_key(|(_, regeneration)| *regeneration);
            let maximum = cells.last()?.1;
            cells
                .iter()
                .rev()
                .find(|(_, regeneration)| *regeneration >= 4 && *regeneration < maximum)
                .copied()
                .map(|(origin, regeneration)| (seed, origin, regeneration))
        })
        .expect("audit fixture requires one positive cell below a strictly richer cell");

    let annual_need = u32::try_from(origin_regeneration + 1).unwrap();

    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_initial_stock_units_per_productivity(0)
        .with_annual_regeneration_units_per_productivity(1)
        .with_annual_need_units_per_person(annual_need)
        .with_productivity_scale_permille(1_000)
        .with_seasonality_scale_permille(0);
    resources.periods_per_year = 1;
    resources.cell_stock_capacity_years = 10;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 1;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let mut migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_candidate_radius_cells(14)
        .with_decision_periods_per_year(1);
    migration.condition_pressure_threshold_permille = 1_000;
    migration.resource_pressure_threshold_permille = 1_000;
    migration.minimum_utility_improvement = 0;
    migration.resource_weight = 10;
    migration.water_security_weight = 0;
    migration.kin_weight = 0;
    migration.travel_cost_weight = 0;
    migration.max_uncertainty_penalty_permille = 0;
    migration.relocation_risk_base_penalty_permille = 0;
    migration.relocation_risk_per_cell_permille = 0;
    migration.travel_condition_cost_per_cell = 0;
    migration.max_recorded_decision_traces = 64;

    let config = ExperimentConfig::new(seed, 2)
        .with_world(world_config)
        .with_population(PopulationConfig::new(1).with_max_person_records(10))
        .with_founder_population(one_male_founder(origin))
        .with_demography(no_mortality_demography())
        .with_resources(resources)
        .with_migration(migration);

    let run = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let observations = run.checkpoint.resources.period_observations();
    assert_eq!(observations.len(), 2);

    let first = &observations[0];
    let second = &observations[1];
    assert_eq!(first.total_need, u64::from(annual_need));
    assert_eq!(first.supplied, origin_regeneration);
    assert_eq!(first.unmet, 1);
    assert!(run.manifest.migration.moves_completed >= 1);
    assert_eq!(second.total_need, u64::from(annual_need));
    assert_eq!(second.supplied, second.total_need);

    // With P=1 and maxConditionLossPerPeriod=1, the elapsed full-year maximum-loss budget is 4.
    // The one-unit shortage is deliberately mild enough that year 1 leaves visible condition at
    // 1000 and stores only a non-zero fixed-point deterioration remainder.
    let supplied_permille = first.supplied * 1_000 / first.total_need;
    let deficit_permille = 1_000 - supplied_permille;
    let expected_remainder = deficit_permille * 4;
    assert!(supplied_permille > 750 && supplied_permille < 1_000);
    assert!(expected_remainder > 0 && expected_remainder < 1_000);
    assert_eq!(
        first.condition_after_resource_response.maximum_permille,
        Some(1_000)
    );
    assert_eq!(
        second.condition_after_resource_response.maximum_permille,
        Some(1_000)
    );

    // The second year is fully supplied, but the configured recovery coefficient is exactly zero.
    // Therefore it has no positive condition-response budget with which to cancel the latent
    // first-year deterioration. The v20 causal remainder should remain unchanged.
    let population_json = serde_json::to_value(&run.checkpoint.population).unwrap();
    let actual_remainder = population_json["conditionLossRemainderThousandths"][0]
        .as_u64()
        .unwrap();
    assert_eq!(
        actual_remainder, expected_remainder,
        "a full-supply interval with exactly zero configured recovery must not erase latent M3 deterioration"
    );

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let checkpoint_json = serde_json::to_value(&checkpoint.population).unwrap();
    assert_eq!(
        checkpoint_json["conditionLossRemainderThousandths"][0]
            .as_u64()
            .unwrap(),
        expected_remainder,
        "year-1 checkpoint must preserve the latent deterioration before the full-supply interval"
    );
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    let resumed_json = serde_json::to_value(&resumed.checkpoint.population).unwrap();
    assert_eq!(
        resumed_json["conditionLossRemainderThousandths"][0]
            .as_u64()
            .unwrap(),
        expected_remainder,
        "checkpoint/resume must preserve the same hidden condition state through zero-recovery full supply"
    );
}
