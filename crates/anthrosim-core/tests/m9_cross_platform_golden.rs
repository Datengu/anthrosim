use std::{collections::BTreeSet, env, fs, path::PathBuf};

use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    PopulationConfig, ResourceConfig, RunManifest, Simulation, SimulationCheckpoint,
    TemporaryMobilityConfig, TemporaryMobilityObservabilityReport, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
    derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct M9CrossPlatformGoldenRun {
    manifest: RunManifest,
    checkpoint: SimulationCheckpoint,
    temporary_observability: TemporaryMobilityObservabilityReport,
}

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

fn base_config() -> ExperimentConfig {
    ExperimentConfig::new(0xA19_2026, 2)
        .with_world(WorldConfig::new(4, 4))
        .with_population(
            PopulationConfig::new(40)
                .with_target_household_size(4)
                .with_max_person_records(400),
        )
        .with_demography(no_event_demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn configured_fixture() -> ExperimentConfig {
    let base = base_config();
    let probe = Simulation::new(base.clone()).expect("probe simulation");
    let occupied = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| probe.population().household_location(HouseholdId::new(raw)))
        .collect::<BTreeSet<_>>();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !occupied.contains(cell))
        .expect("fixture needs one unoccupied focal cell");
    let temporary = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "m9-cross-platform-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .expect("focal region"),
        TemporaryMobilitySchedule::new(
            "m9-cross-platform-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![300],
            180,
        )
        .expect("temporary schedule"),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility config");

    base.with_temporary_mobility(temporary)
}

fn golden_fixture() -> M9CrossPlatformGoldenRun {
    let config = configured_fixture();
    let simulation = Simulation::new(config.clone()).expect("configured M9 simulation");
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let mut uninterrupted = simulation.run_recorded().expect("uninterrupted run");

    let paused = Simulation::new(config)
        .expect("checkpoint source")
        .checkpoint_at_year(1)
        .expect("active-journey checkpoint");
    assert!(
        (1..=paused.population.household_count() as u64).any(|raw| paused
            .temporary_mobility
            .active_journey(HouseholdId::new(raw))
            .is_some()),
        "annual checkpoint must contain active temporary journeys"
    );
    let resumed = Simulation::from_checkpoint(paused)
        .expect("resume active journeys")
        .run_recorded()
        .expect("resumed run");
    let expected = &uninterrupted.checkpoint;
    let actual = &resumed.checkpoint;
    assert_eq!(actual.state_digest64, expected.state_digest64);
    assert_eq!(actual.population, expected.population);
    assert_eq!(actual.temporary_mobility, expected.temporary_mobility);
    assert_eq!(actual.resources, expected.resources);
    assert_eq!(actual.migration, expected.migration);
    assert_eq!(actual.rng, expected.rng);
    assert_eq!(actual.events, expected.events);
    assert_eq!(actual.metrics, expected.metrics);

    assert!(
        uninterrupted
            .events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    );
    assert!(
        uninterrupted
            .events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryJourneyArrived { .. }))
    );
    assert!(
        uninterrupted
            .events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryReturnDeparted { .. }))
    );
    assert!(
        uninterrupted
            .events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryJourneyCompleted { .. }))
    );

    let mut temporary_observability = derive_temporary_mobility_observability(
        &world,
        &initial_population,
        &uninterrupted.checkpoint,
    )
    .expect("temporary observability");
    assert!(temporary_observability.summary.journeys_started > 0);
    assert!(temporary_observability.summary.journeys_completed > 0);
    assert!(temporary_observability.summary.visitor_person_days > 0);

    // Exact source revision is provenance, not the cross-platform scientific
    // determinism boundary. All runners execute the same commit, but clearing
    // it prevents platform comparison from depending on build injection.
    uninterrupted.manifest.git_commit = None;
    uninterrupted.checkpoint.git_commit = None;
    temporary_observability.source.git_commit = None;

    M9CrossPlatformGoldenRun {
        manifest: uninterrupted.manifest,
        checkpoint: uninterrupted.checkpoint,
        temporary_observability,
    }
}

#[test]
fn enabled_m9_fixture_is_byte_stable_resumable_and_exportable() {
    let first = serde_json::to_vec_pretty(&golden_fixture()).unwrap();
    let second = serde_json::to_vec_pretty(&golden_fixture()).unwrap();
    assert_eq!(first, second);

    let Some(path) = env::var_os("ANTHROSIM_M9_CROSS_PLATFORM_GOLDEN") else {
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, first).unwrap();
}
