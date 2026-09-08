use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, GridGeometry, LandscapeBundle,
    LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, ResumeLineage,
    SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
    SpatialRealizationMode, SpatialRunRealization, SpatialTargetField, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection,
    WorldConfig, ids::{CellId, HouseholdId, PersonId}, validate_spatial_landscape_recorded_run,
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn layer(id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values,
    }
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        4,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[spatial-m9-population-seed-replay]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(250), Some(500), Some(750)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(750), Some(500), Some(250)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(300), Some(500), Some(700), Some(900)],
            ),
        ],
    )
}

fn mechanisms(environment_seed: u64, population_seed: u64) -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "spatial-m9-population-seed-replay-v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                DOMAIN,
                1_000,
                2_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                DOMAIN,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                DOMAIN,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
    .with_run_realization(SpatialRunRealization::new(
        environment_seed,
        population_seed,
    ))
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

fn temporary_mobility(trigger_days: Vec<u64>) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "spatial-m9-population-seed-focal",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "spatial-m9-population-seed-schedule",
        TemporaryTriggerTiming::DepartureDay,
        trigger_days,
        30,
    )
    .unwrap();
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn synthetic_experiment(process_seed: u64, duration_years: u64, trigger_days: Vec<u64>) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 12;
    resources.seasonality_scale_permille = 1_000;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(process_seed, duration_years)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(20).with_target_household_size(4))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility(trigger_days))
}

fn declared_founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "spatial-m9-population-seed-declared-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(2),
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(32 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn declared_experiment(process_seed: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 4;
    resources.seasonality_scale_permille = 1_000;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(process_seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(2).with_target_household_size(1))
        .with_founder_population(declared_founders())
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility(vec![100, 300]))
}

#[test]
fn fixed_environment_and_population_replay_across_process_seeds_with_active_m9() {
    let source = landscape();
    let mechanisms = mechanisms(70_001, 80_001);
    let mut founder_digest = None;
    let mut world_digest = None;

    for process_seed in [90_001, 90_002, 90_003] {
        let simulation = SpatialLandscapeSimulation::new(
            synthetic_experiment(process_seed, 1, vec![100, 300]),
            source.clone(),
            mechanisms.clone(),
        )
        .unwrap();

        assert_eq!(
            simulation.spatial_binding().environment.realization.mode,
            SpatialRealizationMode::ExplicitSplit
        );
        assert_eq!(
            simulation.spatial_binding().environment.realization.population_seed,
            80_001
        );
        assert_eq!(
            simulation.spatial_binding().environment.realization.process_seed,
            process_seed
        );
        if let Some(expected) = founder_digest {
            assert_eq!(simulation.population().digest64(), expected);
        } else {
            founder_digest = Some(simulation.population().digest64());
        }
        if let Some(expected) = world_digest {
            assert_eq!(simulation.world().digest64(), expected);
        } else {
            world_digest = Some(simulation.world().digest64());
        }

        let run = simulation.run_recorded().unwrap();
        validate_spatial_landscape_recorded_run(&run, &source).unwrap();
        assert_eq!(
            run.manifest.spatial.environment.realization.population_seed,
            80_001
        );
        assert_eq!(run.core_manifest().experiment.seed, process_seed);
    }
}

#[test]
fn changing_synthetic_population_seed_changes_founders_and_each_history_remains_replay_valid() {
    let source = landscape();
    let process_seed = 90_100;
    let first = SpatialLandscapeSimulation::new(
        synthetic_experiment(process_seed, 1, vec![100, 300]),
        source.clone(),
        mechanisms(70_100, 80_100),
    )
    .unwrap();
    let second = SpatialLandscapeSimulation::new(
        synthetic_experiment(process_seed, 1, vec![100, 300]),
        source.clone(),
        mechanisms(70_100, 80_101),
    )
    .unwrap();

    assert_eq!(first.world().digest64(), second.world().digest64());
    assert_ne!(
        first.population().digest64(),
        second.population().digest64(),
        "populationSeed must remain causal for synthetic founder realization"
    );

    let first = first.run_recorded().unwrap();
    let second = second.run_recorded().unwrap();
    validate_spatial_landscape_recorded_run(&first, &source).unwrap();
    validate_spatial_landscape_recorded_run(&second, &source).unwrap();
    assert_ne!(
        first.manifest.spatial.environment.realization.population_seed,
        second.manifest.spatial.environment.realization.population_seed
    );
    assert_eq!(first.core_manifest().experiment.seed, process_seed);
    assert_eq!(second.core_manifest().experiment.seed, process_seed);
}

#[test]
fn declared_founders_remain_population_seed_inert_with_active_m9() {
    let source = landscape();
    let process_seed = 90_200;
    let first = SpatialLandscapeSimulation::new(
        declared_experiment(process_seed),
        source.clone(),
        mechanisms(70_200, 80_200),
    )
    .unwrap()
    .run_recorded()
    .unwrap();
    let second = SpatialLandscapeSimulation::new(
        declared_experiment(process_seed),
        source.clone(),
        mechanisms(70_200, 80_201),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    validate_spatial_landscape_recorded_run(&first, &source).unwrap();
    validate_spatial_landscape_recorded_run(&second, &source).unwrap();
    assert_eq!(first.checkpoint.core_checkpoint, second.checkpoint.core_checkpoint);
    assert_ne!(
        first.manifest.spatial.environment.realization.population_seed,
        second.manifest.spatial.environment.realization.population_seed
    );
}

#[test]
fn explicit_split_m9_checkpoint_resume_matches_uninterrupted() {
    let source = landscape();
    let config = synthetic_experiment(90_300, 2, vec![100, 500]);
    let mechanisms = mechanisms(70_300, 80_300);

    let uninterrupted = SpatialLandscapeSimulation::new(
        config.clone(),
        source.clone(),
        mechanisms.clone(),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let checkpoint = SpatialLandscapeSimulation::new(config, source.clone(), mechanisms)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, source.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    validate_spatial_landscape_recorded_run(&uninterrupted, &source).unwrap();
    validate_spatial_landscape_recorded_run(&resumed, &source).unwrap();

    let mut resumed_without_lineage = resumed.clone();
    resumed_without_lineage.manifest.core_manifest.resume_lineage = ResumeLineage::new();
    resumed_without_lineage.checkpoint.core_checkpoint.resume_lineage = ResumeLineage::new();
    resumed_without_lineage.checkpoint.core_checkpoint = resumed_without_lineage
        .checkpoint
        .core_checkpoint
        .seal_continuation_identity();
    assert_eq!(resumed_without_lineage, uninterrupted);
}
