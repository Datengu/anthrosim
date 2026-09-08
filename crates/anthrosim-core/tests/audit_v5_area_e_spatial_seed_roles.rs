use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, GridGeometry, LandscapeBundle,
    LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRealizationMode,
    SpatialRunRealization, SpatialTargetField, TemporaryMobilityConfig, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    validate_spatial_landscape_recorded_run,
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
            spatial_reference: "LOCAL_CS[audit-v5-area-e-seed-roles]".to_owned(),
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
        "audit-v5-area-e-spatial-seed-roles",
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

fn temporary_mobility() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v5-area-e-focal",
        FocalRegionSource::Synthetic,
        vec![CellId::new(4)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v5-area-e-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100, 300],
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

fn synthetic_experiment(process_seed: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 12;
    resources.seasonality_scale_permille = 1_000;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(process_seed, 1)
        .with_world(WorldConfig::new(4, 1))
        .with_population(PopulationConfig::new(20).with_target_household_size(4))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

fn declared_founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-e-declared-founders",
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
        .with_temporary_mobility(temporary_mobility())
}

#[test]
fn process_seed_cannot_move_fixed_environment_or_synthetic_founders_with_active_seasonality_and_m9() {
    let source = landscape();
    let mechanisms = mechanisms(70_001, 80_001);

    let first = SpatialLandscapeSimulation::new(
        synthetic_experiment(90_001),
        source.clone(),
        mechanisms.clone(),
    )
    .unwrap();
    let second = SpatialLandscapeSimulation::new(
        synthetic_experiment(90_002),
        source,
        mechanisms,
    )
    .unwrap();

    assert_eq!(first.world().digest64(), second.world().digest64());
    assert_eq!(first.population().digest64(), second.population().digest64());
    assert_eq!(
        first
            .world()
            .cells()
            .iter()
            .map(|cell| cell.season_amplitude)
            .collect::<Vec<_>>(),
        second
            .world()
            .cells()
            .iter()
            .map(|cell| cell.season_amplitude)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        first.spatial_binding().environment.realization.mode,
        SpatialRealizationMode::ExplicitSplit
    );
    assert_eq!(
        first.spatial_binding().environment.realization.environment_seed,
        second.spatial_binding().environment.realization.environment_seed
    );
    assert_eq!(
        first.spatial_binding().environment.realization.population_seed,
        second.spatial_binding().environment.realization.population_seed
    );
    assert_ne!(
        first.spatial_binding().environment.realization.process_seed,
        second.spatial_binding().environment.realization.process_seed
    );

    let first_run = first.run_recorded().unwrap();
    let second_run = second.run_recorded().unwrap();
    validate_spatial_landscape_recorded_run(&first_run, &landscape()).unwrap();
    validate_spatial_landscape_recorded_run(&second_run, &landscape()).unwrap();

    eprintln!(
        "process split: world={:016x}; population={:016x}; process_seeds=({}, {}); event_counts=({}, {})",
        first_run.core_manifest().world.digest64,
        first_run.core_manifest().population.digest64,
        first_run.manifest.spatial.environment.realization.process_seed,
        second_run.manifest.spatial.environment.realization.process_seed,
        first_run.events().events.len(),
        second_run.events().events.len(),
    );
}

#[test]
fn population_seed_is_causally_inert_for_declared_founders_even_with_active_seasonality_and_m9() {
    let source = landscape();
    let process_seed = 90_100;
    let first = SpatialLandscapeSimulation::new(
        declared_experiment(process_seed),
        source.clone(),
        mechanisms(70_100, 80_100),
    )
    .unwrap()
    .run_recorded()
    .unwrap();
    let second = SpatialLandscapeSimulation::new(
        declared_experiment(process_seed),
        source.clone(),
        mechanisms(70_100, 80_101),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    validate_spatial_landscape_recorded_run(&first, &source).unwrap();
    validate_spatial_landscape_recorded_run(&second, &source).unwrap();

    assert_eq!(
        first.checkpoint.core_checkpoint,
        second.checkpoint.core_checkpoint,
        "declared-founder initialization must not acquire a causal dependency on populationSeed"
    );
    assert_eq!(
        first.manifest.spatial.environment.transformed_world_digest64,
        second.manifest.spatial.environment.transformed_world_digest64
    );
    assert_ne!(
        first.manifest.spatial.environment.realization.population_seed,
        second.manifest.spatial.environment.realization.population_seed
    );

    eprintln!(
        "declared founders: population_seeds=({}, {}); core_state={:016x}; events={}",
        first.manifest.spatial.environment.realization.population_seed,
        second.manifest.spatial.environment.realization.population_seed,
        first.checkpoint.core_checkpoint.state_digest64,
        first.events().events.len(),
    );
}
