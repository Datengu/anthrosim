use anthrosim_core::rng::RngFactory;
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, Simulation,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTravelResolution, TemporaryTriggerTiming, World, WorldConfig,
    derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId, PersonId},
};

const SEED: u64 = 717_001;
const DAYS_PER_YEAR: i64 = 365;
const DESTINATION: CellId = CellId::new(7);
const NEAR_RESIDENCE: CellId = CellId::new(6);
const FAR_RESIDENCE: CellId = CellId::new(1);

fn region() -> FocalRegion {
    FocalRegion::new(
        "audit-v6-area-f-handoff-destination",
        FocalRegionSource::Synthetic,
        vec![DESTINATION],
    )
    .unwrap()
}

fn travel_model() -> TemporaryTravelModel {
    TemporaryTravelModel::new(
        "audit_v6_area_f_handoff_travel_v1",
        ParameterProvenance::SyntheticValidation,
        1_000,
        u16::MAX,
    )
    .unwrap()
}

fn travel_days() -> (u32, u32) {
    let world = World::generate(WorldConfig::new(7, 1), RngFactory::new(SEED)).unwrap();
    let table = travel_model()
        .derive_table_with_tie_seed(&region(), &world, 717_002)
        .unwrap();
    let near = match table.resolution(NEAR_RESIDENCE).unwrap() {
        TemporaryTravelResolution::Reachable {
            outbound_travel_days,
            ..
        } => outbound_travel_days,
        TemporaryTravelResolution::Unreachable => panic!("near residence unexpectedly unreachable"),
    };
    let far = match table.resolution(FAR_RESIDENCE).unwrap() {
        TemporaryTravelResolution::Reachable {
            outbound_travel_days,
            ..
        } => outbound_travel_days,
        TemporaryTravelResolution::Unreachable => panic!("far residence unexpectedly unreachable"),
    };
    assert!(far > near, "far route must take longer than near route");
    (near, far)
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-f-handoff-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: NEAR_RESIDENCE,
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: FAR_RESIDENCE,
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(31 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(2),
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
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn experiment() -> (ExperimentConfig, u32, u32) {
    let (near_days, far_days) = travel_days();
    let stay_duration_days = far_days - near_days;
    assert!(stay_duration_days > 0);

    let schedule = TemporaryMobilitySchedule::new(
        "audit-v6-area-f-zero-duration-handoff",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        stay_duration_days,
    )
    .unwrap();
    let temporary = TemporaryMobilityConfig::new(region(), schedule, travel_model()).unwrap();

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(SEED, 1)
        .with_world(WorldConfig::new(7, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_target_household_size(1)
                .with_max_person_records(10),
        )
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary);

    (config, near_days, far_days)
}

#[test]
fn zero_duration_same_day_handoff_cannot_inflate_positive_duration_peak_visitors() {
    let (config, near_days, far_days) = experiment();
    let stay_duration_days = far_days - near_days;
    let simulation = Simulation::new(config).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().unwrap();
    recorded.validate_invariants().unwrap();

    let report =
        derive_temporary_mobility_observability(&world, &initial_population, &recorded.checkpoint)
            .unwrap();
    assert_eq!(report.summary.journeys_started, 2);
    assert_eq!(report.summary.journeys_completed, 2);

    let near = report
        .journeys
        .iter()
        .find(|journey| journey.household == HouseholdId::new(1))
        .unwrap();
    let far = report
        .journeys
        .iter()
        .find(|journey| journey.household == HouseholdId::new(2))
        .unwrap();

    assert_eq!(near.outbound_travel_days, near_days);
    assert_eq!(far.outbound_travel_days, far_days);
    assert_eq!(near.planned_visit_duration_days, stay_duration_days);
    assert_eq!(far.planned_visit_duration_days, stay_duration_days);
    assert_eq!(
        near.return_departure_day, far.arrival_day,
        "near visitor must leave at the exact boundary where the far visitor arrives"
    );
    assert_eq!(
        near.observed_visitor_person_days,
        u64::from(stay_duration_days)
    );
    assert_eq!(
        far.observed_visitor_person_days,
        u64::from(stay_duration_days)
    );
    assert_eq!(
        report.summary.visitor_person_days,
        2 * u64::from(stay_duration_days)
    );

    let destination = report
        .cells
        .iter()
        .find(|cell| cell.cell == DESTINATION)
        .unwrap();

    eprintln!(
        "handoff day={}; near=[{},{}); far=[{},{}); visitor_person_days={}; reported_peak={}; cell_peak={}",
        far.arrival_day,
        near.arrival_day,
        near.return_departure_day,
        far.arrival_day,
        far.return_departure_day,
        report.summary.visitor_person_days,
        report.summary.peak_visitors,
        destination.peak_visitors,
    );

    // The two half-open positive-duration visiting intervals touch but never overlap. At every
    // interval with nonzero duration, exactly one living visitor is present at the destination.
    let positive_duration_peak = 1;
    assert_eq!(
        report.summary.peak_visitors, positive_duration_peak,
        "peak visitors must describe simultaneous physical presence over the half-open observation intervals, not a zero-duration intermediate state created only by same-day event ordering"
    );
    assert_eq!(destination.peak_visitors, positive_duration_peak);
}
