use std::collections::BTreeSet;

use crate::{
    checkpoint::state_digest64_with_temporary_mobility,
    config::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PROBABILITY_PER_MILLION,
        PopulationConfig, ResourceConfig, WorldConfig,
    },
    events::{EventKind, TemporaryJourneyIneligibility},
    focal_region::{FocalRegion, FocalRegionSource},
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    migration::MigrationSystem,
    provenance::ResumeLineage,
    rng::RngFactory,
    simulation::Simulation,
    temporary_mobility::{
        HouseholdPresence, TemporaryMobilityProgram, TemporaryMobilitySchedule,
        TemporaryTravelResolution, TemporaryTravelTable, TemporaryTriggerTiming,
    },
    world::World,
};

fn stable_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn forced_fertility_demography() -> DemographyConfig {
    let mut config = stable_demography();
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = PROBABILITY_PER_MILLION;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn stable_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn no_pressure_resources() -> ResourceConfig {
    let mut config = stable_resources();
    config.annual_need_units_per_person = 0;
    config
}

fn m9_config(
    seed: u64,
    duration_years: u64,
    initial_population: u32,
    demography: DemographyConfig,
) -> ExperimentConfig {
    ExperimentConfig::new(seed, duration_years)
        .with_world(WorldConfig::new(16, 16))
        .with_population(
            PopulationConfig::new(initial_population)
                .with_target_household_size(5)
                .with_max_person_records(initial_population.saturating_mul(10)),
        )
        .with_demography(demography)
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1())
}

fn unoccupied_destination(simulation: &Simulation) -> CellId {
    let residences: BTreeSet<_> = (1..=simulation.population().household_count() as u64)
        .filter_map(|raw| {
            simulation
                .population()
                .household_location(HouseholdId::new(raw))
        })
        .collect();
    (1..=simulation.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !residences.contains(cell))
        .expect("test world must contain a cell with no resident household")
}

fn temporary_program(
    config: &ExperimentConfig,
    trigger_timing: TemporaryTriggerTiming,
    trigger_days: Vec<u64>,
    stay_duration_days: u32,
    travel_days: u32,
    reachable: bool,
) -> TemporaryMobilityProgram {
    let probe = Simulation::new(config.clone()).unwrap();
    let destination = unoccupied_destination(&probe);
    let region = FocalRegion::new(
        "m9-integration-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let resolutions = (1..=probe.world().cell_count() as u64)
        .map(|raw| {
            let origin = CellId::new(raw);
            if !reachable || region.contains(origin) {
                TemporaryTravelResolution::Unreachable
            } else {
                TemporaryTravelResolution::Reachable {
                    destination,
                    outbound_travel_days: travel_days,
                    return_travel_days: travel_days,
                }
            }
        })
        .collect();
    let travel = TemporaryTravelTable::new(resolutions, &region, probe.world()).unwrap();
    TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "m9-integration-schedule",
            trigger_timing,
            trigger_days,
            stay_duration_days,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap()
}

fn active_checkpoint(seed: u64, duration_years: u64) -> crate::SimulationCheckpoint {
    let config = ExperimentConfig::new(seed, duration_years)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(20).with_target_household_size(5))
        .with_demography(stable_demography())
        .with_resources(stable_resources());
    let mut checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    let world = World::generate(
        checkpoint.experiment.world,
        RngFactory::new(checkpoint.experiment.seed),
    )
    .unwrap();
    let household = HouseholdId::new(1);
    let residence = checkpoint.population.household_location(household).unwrap();
    let destination = (1..=world.cell_count() as u64)
        .map(CellId::new)
        .find(|&cell| cell != residence)
        .unwrap();
    checkpoint
        .temporary_mobility
        .set_presence(
            household,
            HouseholdPresence::Visiting {
                journey: TemporaryJourneyId::new(1),
                destination,
            },
            &checkpoint.population,
            &world,
        )
        .unwrap();

    let migration = MigrationSystem::from_checkpoint_state(
        &checkpoint.population,
        &world,
        &checkpoint.experiment.migration,
        checkpoint.migration.clone(),
    )
    .unwrap();
    checkpoint.state_digest64 = state_digest64_with_temporary_mobility(
        checkpoint.time.days(),
        world.digest64(),
        checkpoint.population.digest64(),
        checkpoint.resources.digest64(),
        migration.digest64(),
        &checkpoint.temporary_mobility,
    );
    if let Some(snapshot) = checkpoint.metrics.snapshots.last_mut() {
        snapshot.state_digest64 = checkpoint.state_digest64;
    }
    checkpoint
}

#[test]
fn active_presence_round_trips_through_checkpoint_integrity() {
    let checkpoint = active_checkpoint(9_001, 2);
    let source_presence = checkpoint.temporary_mobility.clone();
    let source_digest = checkpoint.state_digest64;

    checkpoint.validate_invariants().unwrap();
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();

    assert_eq!(resumed.temporary_mobility, source_presence);
    assert_eq!(resumed.state_digest64, source_digest);
    resumed.validate_invariants().unwrap();
}

#[test]
fn active_temporary_household_is_excluded_from_m4_without_changing_residence() {
    let seed = 9_002;
    let baseline = Simulation::new(
        ExperimentConfig::new(seed, 1)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(20).with_target_household_size(5))
            .with_demography(stable_demography())
            .with_resources(stable_resources()),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let checkpoint = active_checkpoint(seed, 1);
    let household = HouseholdId::new(1);
    let residence = checkpoint.population.household_location(household).unwrap();
    let active = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        baseline.manifest.migration.households_evaluated
            - active.manifest.migration.households_evaluated,
        active.manifest.migration.decision_boundaries
    );
    assert_eq!(
        active.checkpoint.population.household_location(household),
        Some(residence)
    );
    assert_eq!(
        active
            .checkpoint
            .temporary_mobility
            .is_at_residence(household),
        Some(false)
    );
    active.validate_invariants().unwrap();
}

#[test]
fn real_active_journey_resume_matches_uninterrupted_execution() {
    let config = m9_config(9_003, 2, 40, stable_demography());
    let program = temporary_program(
        &config,
        TemporaryTriggerTiming::DepartureDay,
        vec![360],
        20,
        10,
        true,
    );

    let uninterrupted = Simulation::new_with_temporary_mobility(config.clone(), program.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let checkpoint = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    assert!(
        (1..=checkpoint.population.household_count() as u64).any(|raw| {
            checkpoint
                .temporary_mobility
                .is_at_residence(HouseholdId::new(raw))
                == Some(false)
        })
    );

    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    let mut resumed_checkpoint = resumed.checkpoint.clone();
    resumed_checkpoint.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_checkpoint, uninterrupted.checkpoint);
    resumed.validate_invariants().unwrap();
}

#[test]
fn unreachable_households_emit_explicit_non_start_outcomes() {
    let config = m9_config(9_004, 1, 40, stable_demography());
    let household_count = Simulation::new(config.clone())
        .unwrap()
        .population()
        .household_count();
    let program = temporary_program(
        &config,
        TemporaryTriggerTiming::DepartureDay,
        vec![50],
        5,
        3,
        false,
    );
    let run = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    let unreachable = run
        .events()
        .events
        .iter()
        .filter(|record| {
            matches!(
                record.event,
                EventKind::TemporaryJourneyNotStarted {
                    reason: TemporaryJourneyIneligibility::Unreachable,
                    ..
                }
            )
        })
        .count();
    let departed = run
        .events()
        .events
        .iter()
        .filter(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
        .count();
    assert_eq!(unreachable, household_count);
    assert_eq!(departed, 0);
    run.validate_invariants().unwrap();
}

#[test]
fn annual_same_day_order_is_resources_then_temporary_then_m4_then_m2() {
    let config = m9_config(9_005, 1, 200, forced_fertility_demography());
    let household_count = Simulation::new(config.clone())
        .unwrap()
        .population()
        .household_count() as u64;
    let program = temporary_program(
        &config,
        TemporaryTriggerTiming::DepartureDay,
        vec![365],
        400,
        0,
        true,
    );
    let run = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    // The first three resource/M4 boundaries evaluate everyone. At day 365 the resource period
    // has already completed, temporary departure/arrival activates every household, and M4 must
    // therefore evaluate nobody on that fourth boundary.
    assert_eq!(run.manifest.resources.periods_processed, 4);
    assert_eq!(run.manifest.migration.decision_boundaries, 4);
    assert_eq!(
        run.manifest.migration.households_evaluated,
        household_count.saturating_mul(3)
    );

    // Annual demography is after temporary mobility and M4. Forced fertility gives an observable
    // M2 event on this same day, so all M9 departure/arrival events must precede the first birth.
    let last_temporary_sequence = run
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 365
                && matches!(
                    record.event,
                    EventKind::TemporaryJourneyDeparted { .. }
                        | EventKind::TemporaryJourneyArrived { .. }
                )
        })
        .map(|record| record.sequence)
        .max()
        .expect("day-365 temporary events must be present");
    let first_birth_sequence = run
        .events()
        .events
        .iter()
        .filter(|record| record.day == 365 && matches!(record.event, EventKind::Birth { .. }))
        .map(|record| record.sequence)
        .min()
        .expect("forced annual fertility must produce a day-365 birth");
    assert!(last_temporary_sequence < first_birth_sequence);
    run.validate_invariants().unwrap();
}
