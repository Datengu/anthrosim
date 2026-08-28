use crate::{
    config::{PopulationConfig, WorldConfig},
    events::{EventLog, TemporaryJourneyIneligibility},
    focal_region::{FocalRegion, FocalRegionSource},
    ids::{CellId, HouseholdId},
    population::Population,
    rng::RngFactory,
    temporary_mobility::{
        TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryMobilityState,
        TemporaryTravelResolution, TemporaryTravelTable, TemporaryTriggerTiming,
    },
    world::World,
};

const TARGET_DAY: u64 = 100;
const HOUSEHOLD: HouseholdId = HouseholdId::new(1);

fn fixture(seed: u64) -> (World, Population, CellId, CellId, CellId, CellId) {
    let world = World::generate(WorldConfig::new(4, 4), RngFactory::new(seed)).unwrap();
    let population = Population::initialize(
        PopulationConfig::new(4).with_target_household_size(4),
        &world,
        RngFactory::new(seed),
    )
    .unwrap();
    let initial = population.household_location(HOUSEHOLD).unwrap();
    let mut alternatives = (1..=world.cell_count() as u64)
        .map(CellId::new)
        .filter(|&cell| cell != initial);
    let near = alternatives.next().unwrap();
    let far = alternatives.next().unwrap();
    let destination = alternatives.next().unwrap();
    (world, population, initial, near, far, destination)
}

fn program(
    world: &World,
    destination: CellId,
    overrides: &[(CellId, TemporaryTravelResolution)],
) -> TemporaryMobilityProgram {
    let region = FocalRegion::new(
        "target-arrival-reconsideration",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let mut resolutions = vec![TemporaryTravelResolution::Unreachable; world.cell_count()];
    for &(cell, resolution) in overrides {
        let index = usize::try_from(cell.0 - 1).unwrap();
        resolutions[index] = resolution;
    }
    let travel = TemporaryTravelTable::new(resolutions, &region, world).unwrap();
    TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "target-arrival-reconsideration",
            TemporaryTriggerTiming::TargetArrivalDay,
            vec![TARGET_DAY],
            3,
        )
        .unwrap(),
        travel,
        world,
    )
    .unwrap()
}

fn reachable(destination: CellId, travel_days: u32) -> TemporaryTravelResolution {
    TemporaryTravelResolution::Reachable {
        destination,
        outbound_travel_days: travel_days,
        return_travel_days: travel_days,
    }
}

fn relocate(population: &mut Population, world: &World, destination: CellId) {
    let mut destinations = vec![CellId::INVALID; population.household_count()];
    destinations[0] = destination;
    let condition_costs = vec![0; population.household_count()];
    population
        .apply_household_relocations(&destinations, &condition_costs, world)
        .unwrap();
}

#[test]
fn initially_prestart_then_moved_closer_gets_valid_future_departure() {
    let (world, mut population, initial, near, _, destination) = fixture(19701);
    let program = program(
        &world,
        destination,
        &[
            (initial, reachable(destination, 150)),
            (near, reachable(destination, 5)),
        ],
    );
    let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
    let mut events = EventLog::new();

    assert_eq!(state.next_boundary_day(0, 99, &population).unwrap(), None);
    relocate(&mut population, &world, near);
    assert_eq!(
        state
            .next_boundary_day(91, TARGET_DAY, &population)
            .unwrap(),
        Some(95)
    );
    let outcome = state
        .process_day(95, &population, &world, &mut events)
        .unwrap();
    assert_eq!(outcome.departed, 1);
    assert!(outcome.skipped.is_empty());
}

#[test]
fn genuinely_prestart_journey_is_rejected_at_target_boundary() {
    let (world, population, initial, _, _, destination) = fixture(19702);
    let program = program(
        &world,
        destination,
        &[(initial, reachable(destination, 150))],
    );
    let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
    let mut events = EventLog::new();

    assert_eq!(
        state.next_boundary_day(0, TARGET_DAY, &population).unwrap(),
        Some(TARGET_DAY)
    );
    let outcome = state
        .process_day(TARGET_DAY, &population, &world, &mut events)
        .unwrap();
    assert_eq!(outcome.departed, 0);
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(
        outcome.skipped[0].reason,
        TemporaryJourneyIneligibility::DepartureBeforeSimulationStart
    );
}

#[test]
fn feasible_then_moved_farther_reports_missed_window() {
    let (world, mut population, initial, _, far, destination) = fixture(19703);
    let program = program(
        &world,
        destination,
        &[
            (initial, reachable(destination, 5)),
            (far, reachable(destination, 20)),
        ],
    );
    let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
    let mut events = EventLog::new();

    assert_eq!(
        state.next_boundary_day(0, 100, &population).unwrap(),
        Some(95)
    );
    relocate(&mut population, &world, far);
    assert_eq!(
        state
            .next_boundary_day(91, TARGET_DAY, &population)
            .unwrap(),
        Some(TARGET_DAY)
    );
    let outcome = state
        .process_day(TARGET_DAY, &population, &world, &mut events)
        .unwrap();
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(
        outcome.skipped[0].reason,
        TemporaryJourneyIneligibility::DepartureWindowMissed
    );
}

#[test]
fn initially_unreachable_then_moved_reachable_gets_valid_departure() {
    let (world, mut population, initial, near, _, destination) = fixture(19704);
    let program = program(
        &world,
        destination,
        &[
            (initial, TemporaryTravelResolution::Unreachable),
            (near, reachable(destination, 5)),
        ],
    );
    let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
    let mut events = EventLog::new();

    assert_eq!(state.next_boundary_day(0, 99, &population).unwrap(), None);
    relocate(&mut population, &world, near);
    assert_eq!(
        state
            .next_boundary_day(91, TARGET_DAY, &population)
            .unwrap(),
        Some(95)
    );
    let outcome = state
        .process_day(95, &population, &world, &mut events)
        .unwrap();
    assert_eq!(outcome.departed, 1);
}

#[test]
fn serialized_pretrigger_state_preserves_reconsideration() {
    let (world, mut population, initial, near, _, destination) = fixture(19705);
    let program = program(
        &world,
        destination,
        &[
            (initial, reachable(destination, 150)),
            (near, reachable(destination, 5)),
        ],
    );
    let state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();

    assert_eq!(state.next_boundary_day(50, 90, &population).unwrap(), None);
    let encoded = serde_json::to_string(&state).unwrap();
    let mut resumed: TemporaryMobilityState = serde_json::from_str(&encoded).unwrap();
    assert_eq!(resumed, state);

    relocate(&mut population, &world, near);
    assert_eq!(
        resumed
            .next_boundary_day(91, TARGET_DAY, &population)
            .unwrap(),
        Some(95)
    );
    let mut events = EventLog::new();
    let outcome = resumed
        .process_day(95, &population, &world, &mut events)
        .unwrap();
    assert_eq!(outcome.departed, 1);
}
