use std::collections::BTreeSet;

use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    ParameterProvenance, PopulationConfig, RecordedRun, ResourceConfig, Simulation,
    TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig,
    ids::{CellId, HouseholdId, TemporaryJourneyId},
};

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

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn recorded_m9_run() -> RecordedRun {
    let config = ExperimentConfig::new(9_141, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(
            PopulationConfig::new(20)
                .with_target_household_size(5)
                .with_max_person_records(200),
        )
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let probe = Simulation::new(config.clone()).unwrap();
    let residences = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| {
            probe
                .population()
                .household_location(HouseholdId::new(raw))
        })
        .collect::<BTreeSet<_>>();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !residences.contains(cell))
        .expect("test world needs an unoccupied focal cell");
    let region = FocalRegion::new(
        "m9-history-invariant-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "m9-history-invariant-travel",
        ParameterProvenance::SyntheticValidation,
        100_000,
        u16::MAX,
    )
    .unwrap();
    let travel = model.derive_table(&region, probe.world()).unwrap();
    let program = TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "m9-history-invariant-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![50],
            10,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap();

    Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap()
}

fn renumber(run: &mut RecordedRun) {
    for (index, record) in run.checkpoint.events.events.iter_mut().enumerate() {
        record.sequence = u64::try_from(index).unwrap().saturating_add(1);
    }
}

fn assert_history_rejected(run: &RecordedRun) {
    let error = run
        .checkpoint
        .validate_invariants()
        .expect_err("tampered temporary history must fail checkpoint invariants");
    assert!(
        error
            .to_string()
            .contains("temporary mobility event history is invalid"),
        "unexpected invariant error: {error}"
    );
}

#[test]
fn valid_m9_history_passes_general_checkpoint_and_run_invariants() {
    let run = recorded_m9_run();
    run.checkpoint.validate_invariants().unwrap();
    run.validate_invariants().unwrap();
}

#[test]
fn cross_wired_arrival_journey_is_rejected_without_changing_terminal_state() {
    let mut run = recorded_m9_run();
    let record = run
        .checkpoint
        .events
        .events
        .iter_mut()
        .find(|record| matches!(record.event, EventKind::TemporaryJourneyArrived { .. }))
        .expect("fixture must contain an arrival");
    let EventKind::TemporaryJourneyArrived { journey, .. } = &mut record.event else {
        unreachable!();
    };
    *journey = TemporaryJourneyId::new(journey.0.saturating_add(10_000));

    assert_history_rejected(&run);
}

#[test]
fn omitted_arrival_is_rejected_after_sequence_is_made_superficially_valid() {
    let mut run = recorded_m9_run();
    let index = run
        .checkpoint
        .events
        .events
        .iter()
        .position(|record| matches!(record.event, EventKind::TemporaryJourneyArrived { .. }))
        .expect("fixture must contain an arrival");
    run.checkpoint.events.events.remove(index);
    renumber(&mut run);

    assert_history_rejected(&run);
}

#[test]
fn duplicate_departure_is_rejected_after_sequence_is_made_superficially_valid() {
    let mut run = recorded_m9_run();
    let index = run
        .checkpoint
        .events
        .events
        .iter()
        .position(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
        .expect("fixture must contain a departure");
    let duplicate = run.checkpoint.events.events[index].clone();
    run.checkpoint.events.events.insert(index + 1, duplicate);
    renumber(&mut run);

    assert_history_rejected(&run);
}

#[test]
fn mistimed_return_transition_is_rejected_even_when_event_order_remains_monotonic() {
    let mut run = recorded_m9_run();
    let return_index = run
        .checkpoint
        .events
        .events
        .iter()
        .position(|record| matches!(record.event, EventKind::TemporaryReturnDeparted { .. }))
        .expect("fixture must contain a return departure");
    let previous_day = run.checkpoint.events.events[return_index - 1].day;
    let planned_day = run.checkpoint.events.events[return_index].day;
    assert!(previous_day < planned_day);
    run.checkpoint.events.events[return_index].day = previous_day;

    assert_history_rejected(&run);
}
