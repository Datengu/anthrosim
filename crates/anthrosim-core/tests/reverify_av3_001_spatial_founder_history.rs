use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain, MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig,
    PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TransformDirection, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
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
    LandscapeBundle::new(
        2,
        2,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 10,
            cell_size_y: 10,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[av3-001-reverify]".to_owned(),
        },
        vec![
            layer(
                "terrain",
                LandscapeLayerRole::TerrainTraversal,
                vec![Some(0), Some(250), Some(500), Some(1_000)],
            ),
            layer(
                "water",
                LandscapeLayerRole::WaterAccessibility,
                vec![Some(1_000), Some(750), Some(500), Some(250)],
            ),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                vec![Some(400), Some(600), Some(800), Some(1_000)],
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "av3-001-reverify",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                2_000,
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
    .with_run_realization(SpatialRunRealization::new(38_701, 38_702))
}

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn founders(last_birth_day: i64) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "av3-001-reverify-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(25 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: Some(last_birth_day),
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn experiment(last_birth_day: i64) -> ExperimentConfig {
    ExperimentConfig::new(38_703, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(founders(last_birth_day))
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn births_by_host(last_birth_day: i64) -> (u64, u64) {
    let config = experiment(last_birth_day);
    let core = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .population
        .births_since_start;
    let spatial = SpatialLandscapeSimulation::new(config, landscape(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap()
        .core_manifest()
        .population
        .births_since_start;
    (core, spatial)
}

#[test]
fn founder_history_exact_effective_spacing_boundary_is_host_invariant() {
    // Requested spacing 1278 days is executable only at annual M2 boundaries,
    // so the effective threshold is 1460 days. At day 365:
    //   lastBirthDay=-1094 -> elapsed 1459 -> blocked
    //   lastBirthDay=-1095 -> elapsed 1460 -> eligible
    let just_short = births_by_host(-1_094);
    let exact = births_by_host(-1_095);

    assert_eq!(just_short, (0, 0));
    assert_eq!(exact, (1, 1));
}

#[test]
fn founder_history_survives_spatial_checkpoint_before_first_fertility_boundary() {
    let config = experiment(-1_094);
    let source = landscape();

    let uninterrupted =
        SpatialLandscapeSimulation::new(config.clone(), source.clone(), mechanisms())
            .unwrap()
            .run_recorded()
            .unwrap();

    let checkpoint = SpatialLandscapeSimulation::new(config, source.clone(), mechanisms())
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    let resumed = SpatialLandscapeSimulation::from_checkpoint(checkpoint, source)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        uninterrupted.core_manifest().population.births_since_start,
        0
    );
    assert_eq!(resumed.core_manifest().population.births_since_start, 0);
    assert_eq!(resumed.events(), uninterrupted.events());
    assert_eq!(resumed.metrics(), uninterrupted.metrics());
}
