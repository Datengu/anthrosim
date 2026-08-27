use anthrosim_core::{
    DemographyConfig, ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy, PopulationConfig,
    ResourceConfig, SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
    SpatialRealizationMode, SpatialRunRealization, SpatialTargetField, TransformDirection,
    WorldConfig,
};

fn landscape() -> LandscapeBundle {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    let layer = |id: &str, role: LandscapeLayerRole, values: Vec<Option<i32>>| LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(domain),
        evidence_input_id: None,
        values,
    };
    let cell_count = 64;
    LandscapeBundle::new(
        8,
        8,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[realization-test]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                (0..cell_count)
                    .map(|index| Some((index * 17 % 1_001) as i32))
                    .collect(),
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                (0..cell_count)
                    .map(|index| Some((1_000 - index * 11 % 1_001) as i32))
                    .collect(),
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                (0..cell_count)
                    .map(|index| Some((250 + index * 13 % 751) as i32))
                    .collect(),
            ),
        ],
    )
}

fn mechanisms(environment_seed: u64, population_seed: u64) -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "spatial-realization-seed-test-v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                2_500,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                domain,
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

fn demography_with_stochastic_mortality() -> DemographyConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 500_000;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    demography
}

fn experiment(process_seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(process_seed, 1)
        .with_world(WorldConfig::new(8, 8))
        .with_population(
            PopulationConfig::new(300)
                .with_target_household_size(5)
                .with_max_person_records(1_000),
        )
        .with_demography(demography_with_stochastic_mortality())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn fixed_environment_and_founders_are_invariant_to_process_seed() {
    let landscape = landscape();
    let mechanisms = mechanisms(7_001, 8_001);

    let first = SpatialLandscapeSimulation::new(experiment(9_001), landscape.clone(), mechanisms.clone())
        .expect("first spatial simulation");
    let second = SpatialLandscapeSimulation::new(experiment(9_002), landscape, mechanisms)
        .expect("second spatial simulation");

    assert_eq!(first.world().digest64(), second.world().digest64());
    assert_eq!(first.population().digest64(), second.population().digest64());
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

    let first_run = first.run_recorded().expect("first recorded run");
    let second_run = second.run_recorded().expect("second recorded run");
    assert_ne!(
        first_run.events().events,
        second_run.events().events,
        "changing only the process seed should be able to change stochastic event history"
    );
}

#[test]
fn environment_seed_can_vary_without_changing_process_or_founder_seed() {
    let landscape = landscape();
    let process_seed = 9_100;
    let first = SpatialLandscapeSimulation::new(
        experiment(process_seed),
        landscape.clone(),
        mechanisms(7_100, 8_100),
    )
    .expect("first environment realization");
    let second = SpatialLandscapeSimulation::new(
        experiment(process_seed),
        landscape,
        mechanisms(7_101, 8_100),
    )
    .expect("second environment realization");

    assert_ne!(first.world().digest64(), second.world().digest64());
    assert_eq!(first.population().digest64(), second.population().digest64());
    assert_eq!(
        first.spatial_binding().config_identity,
        second.spatial_binding().config_identity,
        "mechanism transformation identity must not be conflated with realization choice"
    );
    assert_eq!(
        first.spatial_binding().environment.realization.process_seed,
        second.spatial_binding().environment.realization.process_seed
    );
    assert_eq!(
        first.spatial_binding().environment.realization.population_seed,
        second.spatial_binding().environment.realization.population_seed
    );
}
