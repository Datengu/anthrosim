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
            spatial_reference: "LOCAL_CS[audit-v3-founder-history]".to_owned(),
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
        "audit-v3-founder-history-parity",
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
    .with_run_realization(SpatialRunRealization::new(41_001, 41_002))
}

fn forced_fertility() -> DemographyConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    demography.male_parent_min_age_years = 0;
    demography.male_parent_max_age_years_exclusive = 100;
    demography
}

fn founders(last_birth_day: i64) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v3-founder-reproductive-history",
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
    ExperimentConfig::new(41_003, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(founders(last_birth_day))
        .with_demography(forced_fertility())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

/// Audit-v3 Area A host-parity adversary. Declared founder reproductive history is part of the
/// experiment's causal initial state and therefore must constrain fertility identically in the
/// synthetic and spatial authoritative hosts.
#[test]
fn spatial_host_preserves_declared_founder_birth_spacing_history() {
    let config = experiment(-100);

    let synthetic = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let spatial = SpatialLandscapeSimulation::new(config, landscape(), mechanisms())
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(synthetic.manifest.population.births_since_start, 0);
    assert_eq!(
        spatial.core_manifest().population.births_since_start,
        synthetic.manifest.population.births_since_start,
        "spatial execution must honor the same declared pre-run lastBirthDay history as synthetic execution",
    );
}
