use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, Simulation, TemporaryJourneyIneligibility,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTravelResolution, TemporaryTriggerTiming, World, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
};

const ORIGIN: CellId = CellId::new(1);
const MIGRATION_DESTINATION: CellId = CellId::new(2);
const FOCAL_DESTINATION: CellId = CellId::new(3);
const FIRST_M4_BOUNDARY: u64 = 91;
const TARGET_ARRIVAL_DAY: u64 = 92;

fn no_event_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn controlled_world() -> (u64, World, u16) {
    for seed in 1..=100_000_u64 {
        let world = World::generate(WorldConfig::new(3, 1), RngFactory::new(seed))
            .expect("controlled core world");
        let origin = world.cell(ORIGIN).expect("origin cell");
        let migration = world
            .cell(MIGRATION_DESTINATION)
            .expect("migration destination cell");
        let focal = world.cell(FOCAL_DESTINATION).expect("focal cell");
        let traversable_ceiling = migration.movement_cost.max(focal.movement_cost);

        // Cell 1 must be M9-impassable while Cells 2/3 remain traversable, and M4's only
        // radius-one alternative from the edge origin must have strictly better water utility.
        if origin.movement_cost > traversable_ceiling
            && migration.water_access > origin.water_access
        {
            return (seed, world, traversable_ceiling);
        }
    }
    panic!("no deterministic 3-cell core world satisfied the AV6-001 fixture constraints");
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "av6-001-core-host-parity",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: ORIGIN,
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Female,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 0,
        }],
    )
}

fn migration() -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_decision_periods_per_year(4)
        .with_candidate_radius_cells(1);
    config.condition_pressure_threshold_permille = 1_000;
    config.resource_pressure_threshold_permille = 0;
    config.minimum_utility_improvement = 0;
    config.resource_weight = 0;
    config.water_security_weight = 100;
    config.kin_weight = 0;
    config.travel_cost_weight = 0;
    config.max_uncertainty_penalty_permille = 0;
    config.relocation_risk_base_penalty_permille = 0;
    config.relocation_risk_per_cell_permille = 0;
    config.travel_condition_cost_per_cell = 0;
    config
}

#[test]
fn core_host_matches_spatial_host_same_day_reconsideration_contract() {
    let (seed, world, traversable_ceiling) = controlled_world();
    let region = FocalRegion::new(
        "av6-001-core-target",
        FocalRegionSource::Synthetic,
        vec![FOCAL_DESTINATION],
    )
    .expect("focal region");
    let travel_model = TemporaryTravelModel::new(
        "av6-001-core-one-day-travel",
        ParameterProvenance::SyntheticValidation,
        1_000_000,
        traversable_ceiling,
    )
    .expect("travel model");
    let travel = travel_model
        .derive_table(&region, &world)
        .expect("controlled travel table");
    assert_eq!(
        travel.resolution(ORIGIN),
        Some(TemporaryTravelResolution::Unreachable)
    );
    assert!(matches!(
        travel.resolution(MIGRATION_DESTINATION),
        Some(TemporaryTravelResolution::Reachable {
            destination: FOCAL_DESTINATION,
            outbound_travel_days: 1,
            return_travel_days: 1,
        })
    ));

    let schedule = TemporaryMobilitySchedule::new(
        "av6-001-core-target-arrival",
        TemporaryTriggerTiming::TargetArrivalDay,
        vec![TARGET_ARRIVAL_DAY],
        1,
    )
    .expect("target-arrival schedule");
    let temporary = TemporaryMobilityConfig::new(region, schedule, travel_model)
        .expect("temporary mobility");

    let mut resources = ResourceConfig::synthetic_validation_v1();
    // The first M3 settlement is day 91, coincident with the controlled M9/M4 boundary.
    resources.periods_per_year = 4;
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders())
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(migration())
        .with_temporary_mobility(temporary);

    let run = Simulation::new(config)
        .expect("controlled core simulation")
        .run_recorded()
        .expect("controlled core run");

    let migration = run
        .events()
        .events
        .iter()
        .find(|record| {
            record.day == FIRST_M4_BOUNDARY
                && matches!(
                    record.event,
                    EventKind::HouseholdMigration {
                        household,
                        origin,
                        destination,
                        ..
                    } if household == HouseholdId::new(1)
                        && origin == ORIGIN
                        && destination == MIGRATION_DESTINATION
                )
        })
        .expect("M4 must move the controlled household from Cell 1 to Cell 2 on day 91");

    assert!(run.events().events.iter().all(|record| {
        !(record.day == FIRST_M4_BOUNDARY
            && matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    }));

    let missed = run
        .events()
        .events
        .iter()
        .find(|record| {
            record.day == TARGET_ARRIVAL_DAY
                && matches!(
                    record.event,
                    EventKind::TemporaryJourneyNotStarted {
                        household,
                        trigger_day,
                        reason: TemporaryJourneyIneligibility::DepartureWindowMissed,
                        ..
                    } if household == HouseholdId::new(1)
                        && trigger_day == TARGET_ARRIVAL_DAY
                )
        })
        .expect("post-M4 same-day departure must become an explicit missed window");
    assert!(missed.sequence > migration.sequence);

    // Four configured M3 periods must still produce exactly four settlements: excluding the
    // retroactive M9 pass must neither duplicate nor skip coincident day-91 resource work.
    assert_eq!(run.manifest.resources.periods_processed, 4);
    assert_eq!(run.manifest.statistics.resource_periods_processed, 4);
}
