use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig, derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId, PersonId},
};

const DAYS_PER_YEAR: i64 = 365;
const CERTAIN: u32 = 1_000_000;

fn demography(fertility_enabled: bool) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = if fertility_enabled { CERTAIN } else { 0 };
    }
    config
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-f-birth-during-visit-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
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
                birth_day: -(32 * DAYS_PER_YEAR),
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

fn temporary_mobility() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v5-area-f-visit-destination",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v5-area-f-birth-window",
        TemporaryTriggerTiming::DepartureDay,
        vec![350],
        60,
    )
    .unwrap();
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn experiment(fertility_enabled: bool) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(61_001, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_target_household_size(2)
                .with_max_person_records(20),
        )
        .with_founder_population(founders())
        .with_demography(demography(fertility_enabled))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

struct ResultRow {
    visitor_person_days: u64,
    visitor_household_days: u64,
    peak_visitors: u64,
    destination_visitor_person_days: u64,
    arrival_day: u64,
    return_departure_day: u64,
    birth_days: Vec<u64>,
    journey_observed_visitor_person_days: u64,
}

fn run(fertility_enabled: bool) -> ResultRow {
    let simulation = Simulation::new(experiment(fertility_enabled)).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().unwrap();
    recorded.validate_invariants().unwrap();

    let report =
        derive_temporary_mobility_observability(&world, &initial_population, &recorded.checkpoint)
            .unwrap();
    let journey = report.journeys.first().expect("one journey");
    assert_eq!(report.summary.journeys_started, 1);
    assert_eq!(report.summary.journeys_completed, 1);

    let birth_days = recorded
        .checkpoint
        .events
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth { .. } => Some(record.day),
            _ => None,
        })
        .collect::<Vec<_>>();

    let destination = report
        .cells
        .iter()
        .find(|row| row.cell == CellId::new(2))
        .expect("destination cell row");

    ResultRow {
        visitor_person_days: report.summary.visitor_person_days,
        visitor_household_days: report.summary.visitor_household_days,
        peak_visitors: report.summary.peak_visitors,
        destination_visitor_person_days: destination.visitor_person_days,
        arrival_day: journey.arrival_day,
        return_departure_day: journey.return_departure_day,
        birth_days,
        journey_observed_visitor_person_days: journey.observed_visitor_person_days,
    }
}

#[test]
fn birth_during_active_visit_adds_only_post_birth_newborn_visitor_exposure() {
    let control = run(false);
    let birth = run(true);

    assert!(control.birth_days.is_empty());
    assert_eq!(birth.birth_days, vec![365]);
    assert_eq!(control.arrival_day, birth.arrival_day);
    assert_eq!(control.return_departure_day, birth.return_departure_day);
    assert!(birth.arrival_day < 365 && 365 < birth.return_departure_day);

    let visit_days = birth.return_departure_day - birth.arrival_day;
    let expected_control = 2 * visit_days;
    let expected_birth = 2 * (365 - birth.arrival_day) + 3 * (birth.return_departure_day - 365);

    assert_eq!(control.visitor_household_days, visit_days);
    assert_eq!(birth.visitor_household_days, visit_days);
    assert_eq!(control.visitor_person_days, expected_control);
    assert_eq!(birth.visitor_person_days, expected_birth);
    assert_eq!(birth.visitor_person_days - control.visitor_person_days, birth.return_departure_day - 365);
    assert_eq!(control.peak_visitors, 2);
    assert_eq!(birth.peak_visitors, 3);
    assert_eq!(control.destination_visitor_person_days, expected_control);
    assert_eq!(birth.destination_visitor_person_days, expected_birth);
    assert_eq!(control.journey_observed_visitor_person_days, expected_control);
    assert_eq!(birth.journey_observed_visitor_person_days, expected_birth);

    eprintln!(
        "arrival={}; birth=365; return_departure={}; control_visitor_person_days={}; birth_visitor_person_days={}; added_newborn_days={}; peaks=({}, {})",
        birth.arrival_day,
        birth.return_departure_day,
        control.visitor_person_days,
        birth.visitor_person_days,
        birth.visitor_person_days - control.visitor_person_days,
        control.peak_visitors,
        birth.peak_visitors,
    );
}
