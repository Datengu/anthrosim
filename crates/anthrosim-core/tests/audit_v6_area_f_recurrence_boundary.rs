use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig, derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId, PersonId},
};

const DAYS_PER_YEAR: i64 = 365;

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-f-recurrence-boundary-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * DAYS_PER_YEAR),
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        }],
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

fn temporary_mobility(second_trigger_day: u64) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v6-area-f-recurrence-destination",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v6-area-f-recurrence-boundary",
        TemporaryTriggerTiming::DepartureDay,
        vec![100, second_trigger_day],
        5,
    )
    .unwrap();
    let travel = TemporaryTravelModel::new(
        "audit_v6_area_f_one_day_travel_v1",
        ParameterProvenance::SyntheticValidation,
        u32::from(u16::MAX),
        u16::MAX,
    )
    .unwrap();
    TemporaryMobilityConfig::new(region, schedule, travel).unwrap()
}

fn experiment(second_trigger_day: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(716_001, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_target_household_size(1)
                .with_max_person_records(10),
        )
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility(second_trigger_day))
}

#[derive(Debug)]
struct ResultRow {
    trigger_outcomes: u64,
    journeys_started: u64,
    journeys_completed: u64,
    not_started_active_journey: u64,
    visitor_person_days: u64,
    departures: Vec<u64>,
    completions: Vec<u64>,
    active_skips: Vec<u64>,
}

fn run(second_trigger_day: u64) -> ResultRow {
    let simulation = Simulation::new(experiment(second_trigger_day)).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().unwrap();
    recorded.validate_invariants().unwrap();

    let report =
        derive_temporary_mobility_observability(&world, &initial_population, &recorded.checkpoint)
            .unwrap();

    let mut departures = Vec::new();
    let mut completions = Vec::new();
    let mut active_skips = Vec::new();
    for record in &recorded.checkpoint.events.events {
        match record.event {
            EventKind::TemporaryJourneyDeparted { .. } => departures.push(record.day),
            EventKind::TemporaryJourneyCompleted { .. } => completions.push(record.day),
            EventKind::TemporaryJourneyNotStarted { reason, .. }
                if reason
                    == anthrosim_core::events::TemporaryJourneyIneligibility::ActiveJourney =>
            {
                active_skips.push(record.day)
            }
            _ => {}
        }
    }

    ResultRow {
        trigger_outcomes: report.summary.trigger_outcomes,
        journeys_started: report.summary.journeys_started,
        journeys_completed: report.summary.journeys_completed,
        not_started_active_journey: report.summary.not_started_active_journey,
        visitor_person_days: report.summary.visitor_person_days,
        departures,
        completions,
        active_skips,
    }
}

#[test]
fn aggregation_recurrence_boundary_distinguishes_active_return_from_exact_completion_day() {
    // With one-day outbound/return travel and a five-day visit, a day-100 departure has:
    // arrival=101, return departure=106, completion=107.
    let still_active = run(106);
    let exact_completion = run(107);

    eprintln!("still_active={still_active:?}");
    eprintln!("exact_completion={exact_completion:?}");

    assert_eq!(still_active.trigger_outcomes, 2);
    assert_eq!(still_active.journeys_started, 1);
    assert_eq!(still_active.journeys_completed, 1);
    assert_eq!(still_active.not_started_active_journey, 1);
    assert_eq!(still_active.departures, vec![100]);
    assert_eq!(still_active.completions, vec![107]);
    assert_eq!(still_active.active_skips, vec![106]);
    assert_eq!(still_active.visitor_person_days, 5);

    assert_eq!(exact_completion.trigger_outcomes, 2);
    assert_eq!(exact_completion.journeys_started, 2);
    assert_eq!(exact_completion.journeys_completed, 2);
    assert_eq!(exact_completion.not_started_active_journey, 0);
    assert_eq!(exact_completion.departures, vec![100, 107]);
    assert_eq!(exact_completion.completions, vec![107, 114]);
    assert!(exact_completion.active_skips.is_empty());
    assert_eq!(exact_completion.visitor_person_days, 10);
}
