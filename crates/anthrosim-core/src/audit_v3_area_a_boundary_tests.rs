use std::collections::BTreeSet;

use crate::{
    config::{DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, WorldConfig},
    events::EventKind,
    focal_region::{FocalRegion, FocalRegionSource},
    ids::{CellId, HouseholdId},
    simulation::Simulation,
    temporary_mobility::{
        TemporaryMobilityProgram, TemporaryMobilitySchedule, TemporaryTravelResolution,
        TemporaryTravelTable, TemporaryTriggerTiming,
    },
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

fn config(seed: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 4;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(16, 16))
        .with_population(
            PopulationConfig::new(20)
                .with_target_household_size(5)
                .with_max_person_records(200),
        )
        .with_demography(stable_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn program(
    config: &ExperimentConfig,
    trigger_days: Vec<u64>,
    stay_duration_days: u32,
) -> TemporaryMobilityProgram {
    let probe = Simulation::new(config.clone()).unwrap();
    let residences: BTreeSet<_> = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| {
            probe
                .population()
                .household_location(HouseholdId::new(raw))
        })
        .collect();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !residences.contains(cell))
        .expect("test world must contain an unoccupied destination");
    let region = FocalRegion::new(
        "audit-v3-area-a-boundary-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let resolutions = (1..=probe.world().cell_count() as u64)
        .map(|raw| {
            let origin = CellId::new(raw);
            if region.contains(origin) {
                TemporaryTravelResolution::Unreachable
            } else {
                TemporaryTravelResolution::Reachable {
                    destination,
                    outbound_travel_days: 0,
                    return_travel_days: 0,
                }
            }
        })
        .collect();
    let travel = TemporaryTravelTable::new(resolutions, &region, probe.world()).unwrap();

    TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "audit-v3-area-a-boundary-schedule",
            TemporaryTriggerTiming::DepartureDay,
            trigger_days,
            stay_duration_days,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap()
}

/// Fresh audit-v3 Area A adversary: a day-zero temporary departure must be processed before the
/// first later fixed M3 boundary. The first half-open resource period therefore belongs entirely
/// to visitor presence rather than silently attributing day-zero-to-boundary demand to residence.
#[test]
fn day_zero_m9_departure_precedes_first_resource_settlement() {
    let config = config(31_101);
    let program = program(&config, vec![0], 200);
    let run = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    let first = &run.checkpoint.resources.period_observations()[0];
    assert_eq!((first.start_day, first.end_day), (0, 91));
    assert!(first.total_need > 0);
    assert_eq!(first.home_need, 0);
    assert_eq!(first.visitor_need, first.total_need);

    let household_count = run.checkpoint.population.household_count();
    let day_zero_departures = run
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 0
                && matches!(record.event, EventKind::TemporaryJourneyDeparted { .. })
        })
        .count();
    assert_eq!(day_zero_departures, household_count);
    run.validate_invariants().unwrap();
}

/// Fresh audit-v3 Area A adversary: when an old zero-return-transit journey reaches its return
/// boundary on the exact day of a new trigger, all returns/completions must be resolved before any
/// new outward departure. This prevents active-journey status or household loop order from giving
/// one trigger hidden priority over another.
#[test]
fn same_day_return_completion_precedes_every_new_departure() {
    let config = config(31_102);
    let household_count = Simulation::new(config.clone())
        .unwrap()
        .population()
        .household_count();
    let program = program(&config, vec![10, 20], 10);
    let run = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    let day_twenty_completions: Vec<_> = run
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 20
                && matches!(record.event, EventKind::TemporaryJourneyCompleted { .. })
        })
        .collect();
    let second_trigger_departures: Vec<_> = run
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 20
                && matches!(
                    record.event,
                    EventKind::TemporaryJourneyDeparted {
                        trigger_index: 1,
                        ..
                    }
                )
        })
        .collect();

    assert_eq!(day_twenty_completions.len(), household_count);
    assert_eq!(second_trigger_departures.len(), household_count);

    let last_completion = day_twenty_completions
        .iter()
        .map(|record| record.sequence)
        .max()
        .unwrap();
    let first_new_departure = second_trigger_departures
        .iter()
        .map(|record| record.sequence)
        .min()
        .unwrap();
    assert!(last_completion < first_new_departure);

    for raw in 1..=household_count as u64 {
        assert_eq!(
            run.checkpoint
                .temporary_mobility
                .is_at_residence(HouseholdId::new(raw)),
            Some(true)
        );
    }
    run.validate_invariants().unwrap();
}
