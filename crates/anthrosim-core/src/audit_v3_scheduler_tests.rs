use std::collections::BTreeSet;

use crate::{
    config::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
        WorldConfig,
    },
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

fn collision_config(seed: u64) -> ExperimentConfig {
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

fn boundary_collision_program(config: &ExperimentConfig) -> TemporaryMobilityProgram {
    let probe = Simulation::new(config.clone()).unwrap();
    let residences: BTreeSet<_> = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| probe.population().household_location(HouseholdId::new(raw)))
        .collect();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !residences.contains(cell))
        .expect("test world must have an unoccupied temporary destination");
    let region = FocalRegion::new(
        "audit-v3-m3-m9-boundary-collision",
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
            "audit-v3-m3-m9-boundary-collision",
            TemporaryTriggerTiming::DepartureDay,
            vec![91],
            91,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap()
}

/// Audit-v3 Area A adversary: transition into visiting exactly at one M3 settlement boundary and
/// return exactly at the next one. The resource ledger is defined on half-open periods, so the
/// state transition at an end boundary must never be retroactively attributed to the elapsed
/// period. This exercises the authoritative host ordering rather than the ledger in isolation.
#[test]
fn m3_m9_boundary_collisions_preserve_half_open_resource_attribution() {
    let config = collision_config(31_001);
    let program = boundary_collision_program(&config);
    let run = Simulation::new_with_temporary_mobility(config, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    let observations = run.checkpoint.resources.period_observations();
    assert_eq!(observations.len(), 4);

    let first = &observations[0];
    assert_eq!((first.start_day, first.end_day), (0, 91));
    assert!(first.total_need > 0);
    assert_eq!(first.home_need, first.total_need);
    assert_eq!(first.visitor_need, 0);

    let second = &observations[1];
    assert_eq!((second.start_day, second.end_day), (91, 182));
    assert!(second.total_need > 0);
    assert_eq!(second.home_need, 0);
    assert_eq!(second.visitor_need, second.total_need);

    let third = &observations[2];
    assert_eq!((third.start_day, third.end_day), (182, 273));
    assert!(third.total_need > 0);
    assert_eq!(third.home_need, third.total_need);
    assert_eq!(third.visitor_need, 0);

    run.validate_invariants().unwrap();
}
