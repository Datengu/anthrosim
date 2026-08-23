use anthrosim_core::{
    DemographyConfig, ExperimentConfig, InvariantError, MigrationConfig, PopulationConfig,
    ResourceConfig, ResumeLineage, SimTime, Simulation, StopReason, WorldConfig,
    config::PROBABILITY_PER_MILLION,
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

#[test]
fn thousand_year_stable_soak_preserves_cross_system_invariants() {
    let config = ExperimentConfig::new(7_401, 1_000)
        .with_world(WorldConfig::new(8, 8))
        .with_population(PopulationConfig::new(64).with_max_person_records(1_000))
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources());

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    let report = run.validate_invariants().unwrap();

    assert_eq!(run.manifest.stop_reason, StopReason::DurationReached);
    assert_eq!(run.manifest.population.living_population, 64);
    assert_eq!(run.manifest.population.births_since_start, 0);
    assert_eq!(run.manifest.population.deaths_since_start, 0);
    assert_eq!(report.resource_periods, 4_000);
    assert_eq!(report.births, 0);
    assert_eq!(report.deaths, 0);
}

#[test]
fn long_dynamic_checkpoint_resume_matches_uninterrupted_execution() {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million /= 10;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million /= 3;
    }

    let config = ExperimentConfig::new(7_402, 120)
        .with_world(WorldConfig::new(16, 16))
        .with_population(PopulationConfig::new(800).with_max_person_records(100_000))
        .with_demography(demography)
        .with_resources(no_pressure_resources());

    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    uninterrupted.validate_invariants().unwrap();
    assert_eq!(
        uninterrupted.manifest.stop_reason,
        StopReason::DurationReached
    );
    assert!(uninterrupted.manifest.population.births_since_start > 0);
    assert!(uninterrupted.manifest.population.deaths_since_start > 0);

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(60)
        .unwrap();
    checkpoint.validate_invariants().unwrap();
    let source_day = checkpoint.time.days();
    let source_state_digest64 = checkpoint.state_digest64;
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();
    resumed.validate_invariants().unwrap();

    let mut resumed_manifest_without_lineage = resumed.manifest.clone();
    resumed_manifest_without_lineage.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_manifest_without_lineage, uninterrupted.manifest);

    let mut resumed_checkpoint_without_lineage = resumed.checkpoint.clone();
    resumed_checkpoint_without_lineage.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_checkpoint_without_lineage, uninterrupted.checkpoint);

    assert_eq!(resumed.manifest.resume_lineage, resumed.checkpoint.resume_lineage);
    let boundaries = &resumed.manifest.resume_lineage.boundaries;
    assert_eq!(boundaries.len(), 1);
    let boundary = &boundaries[0];
    assert_eq!(boundary.source, boundary.continuation);
    assert_eq!(boundary.boundary_day, source_day);
    assert_eq!(boundary.boundary_completed_years, 60);
    assert_eq!(boundary.source_state_digest64, source_state_digest64);
}

#[test]
fn adversarial_resource_soak_terminates_reproducibly_without_corrupting_state() {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(100)
        .with_annual_need_units_per_person(250);
    resources.max_condition_loss_per_period = 400;
    resources.max_scarcity_mortality_probability_per_million = 500_000;

    for seed in [7_410, 7_411, 7_412, 7_413] {
        let config = ExperimentConfig::new(seed, 200)
            .with_world(WorldConfig::new(8, 8))
            .with_population(PopulationConfig::new(200).with_max_person_records(50_000))
            .with_resources(resources.clone());

        let first = Simulation::new(config.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        first.validate_invariants().unwrap();
        let second = Simulation::new(config).unwrap().run_recorded().unwrap();
        second.validate_invariants().unwrap();

        assert_eq!(first.manifest, second.manifest);
        assert_eq!(first.checkpoint, second.checkpoint);
        assert!(matches!(
            first.manifest.stop_reason,
            StopReason::DurationReached
                | StopReason::PopulationExtinct
                | StopReason::PersonRecordLimitReached
        ));
    }
}

#[test]
fn extinction_and_record_limit_terminal_states_are_explicit_and_reproducible() {
    let mut mortality = no_event_demography();
    for band in &mut mortality.mortality_bands {
        band.annual_probability_per_million = PROBABILITY_PER_MILLION;
    }
    let extinct_config = ExperimentConfig::new(7_420, 50)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(50))
        .with_demography(mortality)
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let extinct = Simulation::new(extinct_config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    extinct.validate_invariants().unwrap();
    assert_eq!(extinct.manifest.stop_reason, StopReason::PopulationExtinct);
    assert_eq!(
        extinct,
        Simulation::new(extinct_config)
            .unwrap()
            .run_recorded()
            .unwrap()
    );

    let mut fertility = no_event_demography();
    for band in &mut fertility.fertility_bands {
        band.annual_probability_per_million = PROBABILITY_PER_MILLION;
    }
    fertility.minimum_birth_spacing_days = 0;
    fertility.male_parent_min_age_years = 0;
    fertility.male_parent_max_age_years_exclusive = 100;
    let limit_config = ExperimentConfig::new(7_421, 50)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(100).with_max_person_records(101))
        .with_demography(fertility)
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));
    let limited = Simulation::new(limit_config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    limited.validate_invariants().unwrap();
    assert_eq!(
        limited.manifest.stop_reason,
        StopReason::PersonRecordLimitReached
    );
    assert_eq!(limited.manifest.population.person_records, 101);
    assert_eq!(
        limited,
        Simulation::new(limit_config)
            .unwrap()
            .run_recorded()
            .unwrap()
    );
}

#[test]
fn invariant_validator_rejects_cross_artifact_accounting_tampering() {
    let config = ExperimentConfig::new(7_430, 20)
        .with_world(WorldConfig::new(8, 8))
        .with_population(PopulationConfig::new(100).with_max_person_records(5_000));
    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.validate_invariants().unwrap();

    let mut tampered = run.checkpoint.clone();
    tampered.migration.total_distance_cells =
        tampered.migration.total_distance_cells.saturating_add(1);
    assert!(tampered.validate_invariants().is_err());
}

#[test]
fn invariant_validator_requires_completed_annual_checkpoint_boundary() {
    let config = ExperimentConfig::new(7_431, 3)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(32).with_max_person_records(1_000))
        .with_demography(no_event_demography())
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    checkpoint.validate_invariants().unwrap();

    let mut malformed = checkpoint;
    malformed.time = SimTime::from_days(366);
    let error = malformed.validate_invariants().unwrap_err();
    assert!(matches!(
        error,
        InvariantError::Violation(message)
            if message == "checkpoint day is not a completed annual boundary"
    ));
}
