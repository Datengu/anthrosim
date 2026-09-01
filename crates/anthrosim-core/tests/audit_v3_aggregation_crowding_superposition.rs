use std::collections::BTreeSet;

use anthrosim_core::ids::{CellId, HouseholdId};
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    ParameterProvenance, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
    derive_temporary_mobility_observability,
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

fn base_config(population: u32, seed: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 365;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(
            PopulationConfig::new(population)
                .with_target_household_size(1)
                .with_max_person_records(32),
        )
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn configured(population: u32, seed: u64, stay_days: u32) -> ExperimentConfig {
    let base = base_config(population, seed);
    let probe = Simulation::new(base.clone()).expect("probe simulation");
    let occupied = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| probe.population().household_location(HouseholdId::new(raw)))
        .collect::<BTreeSet<_>>();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !occupied.contains(cell))
        .expect("fixture needs an unoccupied focal destination");

    let region = FocalRegion::new(
        format!("audit-v3-area-f-region-{population}"),
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .expect("focal region");
    let travel = TemporaryTravelModel::new(
        format!("audit-v3-area-f-fast-travel-{population}"),
        ParameterProvenance::SyntheticValidation,
        1_000_000,
        u16::MAX,
    )
    .expect("travel model");
    let temporary = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            format!("audit-v3-area-f-schedule-{population}"),
            TemporaryTriggerTiming::DepartureDay,
            vec![20],
            stay_days,
        )
        .expect("schedule"),
        travel,
    )
    .expect("temporary mobility config");

    base.with_temporary_mobility(temporary)
}

#[derive(Debug)]
struct Outcome {
    visitor_person_days: u64,
    peak_visitors: u64,
    visitor_need: u64,
    journeys_started: u64,
    journeys_completed: u64,
}

fn run(population: u32) -> Outcome {
    let stay_days = 7;
    let config = configured(population, 0xA3F0_0000 + u64::from(population), stay_days);
    let simulation = Simulation::new(config).expect("configured simulation");
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let recorded = simulation.run_recorded().expect("recorded run");
    recorded.checkpoint.validate_invariants().expect("checkpoint invariants");

    let report = derive_temporary_mobility_observability(
        &world,
        &initial_population,
        &recorded.checkpoint,
    )
    .expect("temporary observability");
    let period = recorded
        .checkpoint
        .resources
        .period_observations()
        .last()
        .expect("single annual resource period observation");

    Outcome {
        visitor_person_days: report.summary.visitor_person_days,
        peak_visitors: report.summary.peak_visitors,
        visitor_need: period.visitor_need,
        journeys_started: report.summary.journeys_started,
        journeys_completed: report.summary.journeys_completed,
    }
}

#[test]
fn simultaneous_aggregation_superposes_presence_and_resource_pressure_exactly() {
    let one = run(1);
    let two = run(2);

    assert_eq!(one.journeys_started, 1);
    assert_eq!(one.journeys_completed, 1);
    assert_eq!(one.visitor_person_days, 7);
    assert_eq!(one.peak_visitors, 1);
    assert_eq!(one.visitor_need, 7);

    assert_eq!(two.journeys_started, 2);
    assert_eq!(two.journeys_completed, 2);
    assert_eq!(two.visitor_person_days, 14);
    assert_eq!(two.peak_visitors, 2);
    assert_eq!(two.visitor_need, 14);

    assert_eq!(two.visitor_person_days, one.visitor_person_days * 2);
    assert_eq!(two.peak_visitors, one.peak_visitors * 2);
    assert_eq!(two.visitor_need, one.visitor_need * 2);
}
