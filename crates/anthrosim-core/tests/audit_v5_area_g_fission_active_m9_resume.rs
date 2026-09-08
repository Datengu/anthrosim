use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    HouseholdLifecycleConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig, derive_temporary_mobility_observability,
    ids::{CellId, HouseholdId},
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

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn experiment() -> ExperimentConfig {
    let base = ExperimentConfig::new(63_001, 3)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(12).with_target_household_size(12))
        .with_demography(stable_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(5, 18),
        );

    let probe = Simulation::new(base.clone()).unwrap();
    let residence = probe.population().household_location(HouseholdId::new(1)).unwrap();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| *cell != residence)
        .unwrap();

    let temporary = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "audit-v5-area-g-fission-active-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "audit-v5-area-g-fission-active-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![465],
            400,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();

    base.with_temporary_mobility(temporary)
}

#[test]
fn checkpoint_resume_after_fission_with_active_m9_journeys_is_exactly_equivalent() {
    let config = experiment();

    let uninterrupted_simulation = Simulation::new(config.clone()).unwrap();
    let initial_world = uninterrupted_simulation.world().clone();
    let initial_population = uninterrupted_simulation.population().clone();
    let uninterrupted = uninterrupted_simulation.run_recorded().unwrap();
    uninterrupted.validate_invariants().unwrap();

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    checkpoint.validate_invariants().unwrap();

    assert_eq!(checkpoint.population.household_count(), 3);
    assert_eq!(
        checkpoint
            .events
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::HouseholdFission { .. }))
            .count(),
        2
    );

    let away = (1..=checkpoint.population.household_count() as u64)
        .filter(|raw| {
            checkpoint
                .temporary_mobility
                .is_at_residence(HouseholdId::new(*raw))
                == Some(false)
        })
        .count();
    assert_eq!(away, 3, "all post-fission households should be on active journeys at checkpoint");

    let checkpoint_report = derive_temporary_mobility_observability(
        &initial_world,
        &initial_population,
        &checkpoint,
    )
    .unwrap();
    assert_eq!(checkpoint_report.summary.journeys_started, 3);
    assert_eq!(checkpoint_report.summary.journeys_active_at_end, 3);

    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    resumed.validate_invariants().unwrap();

    // Resume lineage is intentionally different provenance. Compare every causal/history surface
    // whose identity must otherwise be exact.
    assert_eq!(resumed.checkpoint.state_digest64, uninterrupted.checkpoint.state_digest64);
    assert_eq!(resumed.checkpoint.time, uninterrupted.checkpoint.time);
    assert_eq!(resumed.checkpoint.population, uninterrupted.checkpoint.population);
    assert_eq!(resumed.checkpoint.resources, uninterrupted.checkpoint.resources);
    assert_eq!(resumed.checkpoint.migration, uninterrupted.checkpoint.migration);
    assert_eq!(
        resumed.checkpoint.temporary_mobility,
        uninterrupted.checkpoint.temporary_mobility
    );
    assert_eq!(resumed.checkpoint.events, uninterrupted.checkpoint.events);
    assert_eq!(resumed.checkpoint.metrics, uninterrupted.checkpoint.metrics);
    assert_eq!(resumed.manifest.stop_reason, uninterrupted.manifest.stop_reason);
    assert_eq!(resumed.manifest.population, uninterrupted.manifest.population);
    assert_eq!(resumed.manifest.resources, uninterrupted.manifest.resources);
    assert_eq!(resumed.manifest.migration, uninterrupted.manifest.migration);

    let uninterrupted_report = derive_temporary_mobility_observability(
        &initial_world,
        &initial_population,
        &uninterrupted.checkpoint,
    )
    .unwrap();
    let resumed_report = derive_temporary_mobility_observability(
        &initial_world,
        &initial_population,
        &resumed.checkpoint,
    )
    .unwrap();
    assert_eq!(resumed_report, uninterrupted_report);

    eprintln!(
        "checkpoint_day={}; households={}; active_journeys={}; final_state={:016x}; final_events={}; final_journeys={}",
        730,
        3,
        checkpoint_report.summary.journeys_active_at_end,
        resumed.checkpoint.state_digest64,
        resumed.checkpoint.events.events.len(),
        resumed_report.summary.journeys_started,
    );
}
