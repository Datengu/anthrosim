use anthrosim_core::ids::CellId;
use anthrosim_core::{
    DemographyConfig, EventKind, EvidenceCatalog, ExperimentConfig, ExternalInputEvidence,
    FocalRegion, FocalRegionSource, MigrationConfig, PopulationConfig, ResourceConfig, Simulation,
    SimulationError, TemporaryMobilityConfig, TemporaryMobilityConfigError,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
};

fn no_events_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn definition() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "generic-experiment-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4)],
    )
    .expect("region");
    let schedule = TemporaryMobilitySchedule::new(
        "generic-experiment-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        5,
    )
    .expect("schedule");
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility definition")
}

fn config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(24).with_target_household_size(4))
        .with_demography(no_events_demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(definition())
}

#[test]
fn ordinary_experiment_derives_and_preserves_world_specific_temporary_program() {
    let configured = config(96_101);
    let expected_definition = configured
        .temporary_mobility
        .clone()
        .expect("configured temporary mobility");
    let simulation = Simulation::new(configured.clone()).expect("configured simulation");
    let expected_program = expected_definition
        .derive_program(simulation.world())
        .expect("world-specific program");
    assert_eq!(
        simulation.temporary_mobility().program(),
        Some(&expected_program)
    );

    let run = simulation.run_recorded().expect("configured run");
    assert_eq!(run.manifest.experiment, configured);
    assert_eq!(run.checkpoint.experiment, configured);
    assert!(
        run.events()
            .events
            .iter()
            .any(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
    );
}

#[test]
fn configured_program_is_rederived_for_each_seed_world() {
    let definition = definition();
    for seed in [96_102, 96_103] {
        let configured = ExperimentConfig::new(seed, 0)
            .with_world(WorldConfig::new(4, 1))
            .with_population(PopulationConfig::new(8).with_target_household_size(4))
            .with_temporary_mobility(definition.clone());
        let simulation = Simulation::new(configured).expect("configured simulation");
        let expected = definition
            .derive_program(simulation.world())
            .expect("program derived from this seed's world");
        assert_eq!(simulation.temporary_mobility().program(), Some(&expected));
    }
}

#[test]
fn resume_rejects_config_definition_that_no_longer_matches_authoritative_program() {
    let mut checkpoint = Simulation::new(config(96_104))
        .expect("configured simulation")
        .checkpoint_at_year(1)
        .expect("checkpoint");
    checkpoint
        .experiment
        .temporary_mobility
        .as_mut()
        .expect("definition")
        .schedule
        .stay_duration_days += 1;
    checkpoint = checkpoint.seal_continuation_identity();

    assert!(matches!(
        Simulation::from_checkpoint(checkpoint),
        Err(SimulationError::ConfiguredTemporaryMobilityMismatch { .. })
    ));
}

fn landscape_mask_definition(input_id: &str) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "evidence-bound-experiment-region",
        FocalRegionSource::LandscapeMask {
            layer_id: "aggregation-mask".to_owned(),
            evidence_input_id: input_id.to_owned(),
        },
        vec![CellId::new(4)],
    )
    .expect("evidence-bound region");
    let schedule = TemporaryMobilitySchedule::new(
        "evidence-bound-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        5,
    )
    .expect("schedule");
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility definition")
}

#[test]
fn landscape_mask_region_requires_an_evidence_catalog() {
    let configured = ExperimentConfig::new(96_106, 0)
        .with_world(WorldConfig::new(4, 1))
        .with_temporary_mobility(landscape_mask_definition("aggregation-mask-input"));

    assert!(matches!(
        Simulation::new(configured),
        Err(SimulationError::TemporaryMobilityConfig(
            TemporaryMobilityConfigError::MissingEvidenceCatalog { input_id }
        )) if input_id == "aggregation-mask-input"
    ));
}

#[test]
fn landscape_mask_region_rejects_unknown_evidence_external_input() {
    let catalog =
        EvidenceCatalog::new(Vec::new()).with_external_inputs(vec![ExternalInputEvidence {
            input_id: "different-input".to_owned(),
            evidence_id: "unused-in-this-preflight".to_owned(),
            format: "normalized-binary-mask".to_owned(),
            spatial_reference: None,
            content_digest: None,
        }]);
    let configured = ExperimentConfig::new(96_107, 0)
        .with_world(WorldConfig::new(4, 1))
        .with_temporary_mobility(landscape_mask_definition("aggregation-mask-input"))
        .with_evidence(catalog);

    assert!(matches!(
        Simulation::new(configured),
        Err(SimulationError::TemporaryMobilityConfig(
            TemporaryMobilityConfigError::UnknownEvidenceInput { input_id }
        )) if input_id == "aggregation-mask-input"
    ));
}
