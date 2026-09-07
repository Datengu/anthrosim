use std::collections::BTreeSet;

use crate::{
    config::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
        WorldConfig,
    },
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

fn stable_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(16, 16))
        .with_population(
            PopulationConfig::new(20)
                .with_target_household_size(5)
                .with_max_person_records(200),
        )
        .with_demography(stable_demography())
        .with_resources(stable_resources())
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
        .expect("test world must contain a non-residential focal cell")
}

fn return_on_second_m4_boundary_program(config: &ExperimentConfig) -> TemporaryMobilityProgram {
    let probe = Simulation::new(config.clone()).unwrap();
    let destination = unoccupied_destination(&probe);
    let region = FocalRegion::new(
        "audit-v5-area-a-return-boundary-region",
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

    // Synthetic-validation M4 has four annual decision boundaries. A day-0 departure with zero
    // travel and 182 visiting days completes exactly on the second boundary, day 182.
    TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "audit-v5-area-a-return-boundary-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![0],
            182,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap()
}

#[test]
fn audit_v5_area_a_return_m4_boundary_completed_return_is_immediately_visible() {
    let seed = 50_003;
    let base = config(seed);
    let household_count = Simulation::new(base.clone())
        .unwrap()
        .population()
        .household_count() as u64;

    let baseline = Simulation::new(base.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    assert_eq!(baseline.manifest.migration.decision_boundaries, 4);
    assert_eq!(
        baseline.manifest.migration.households_evaluated,
        household_count * 4
    );

    let program = return_on_second_m4_boundary_program(&base);
    let active = Simulation::new_with_temporary_mobility(base, program)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(active.manifest.migration.decision_boundaries, 4);

    // Day 91 occurs while every household is visiting, so no household is M4-eligible there.
    // The journey completes at day 182 before M4; day 182, 273 and 365 must therefore evaluate
    // all households. If completion were applied after the coincident M4 boundary, this would be
    // only two household-count blocks rather than three.
    assert_eq!(
        active.manifest.migration.households_evaluated,
        household_count * 3
    );

    let day_182_completions = active
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 182
                && matches!(record.event, EventKind::TemporaryJourneyCompleted { .. })
        })
        .count() as u64;
    assert_eq!(day_182_completions, household_count);

    let last_completion_sequence = active
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 182
                && matches!(record.event, EventKind::TemporaryJourneyCompleted { .. })
        })
        .map(|record| record.sequence)
        .max()
        .expect("day-182 completion events must be present");

    let first_migration_sequence = active
        .events()
        .events
        .iter()
        .filter(|record| {
            record.day == 182 && matches!(record.event, EventKind::HouseholdMigration { .. })
        })
        .map(|record| record.sequence)
        .min();
    if let Some(first_migration_sequence) = first_migration_sequence {
        assert!(last_completion_sequence < first_migration_sequence);
    }

    println!(
        "households={household_count} baseline_m4_evaluations={} active_m4_evaluations={} day_182_completions={day_182_completions} last_completion_sequence={last_completion_sequence} first_day_182_migration_sequence={first_migration_sequence:?}",
        baseline.manifest.migration.households_evaluated,
        active.manifest.migration.households_evaluated,
    );

    active.validate_invariants().unwrap();
}
