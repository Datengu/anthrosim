use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, GridGeometry, HouseholdId, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy, ParameterProvenance,
    PersonId, PopulationConfig, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TransformDirection, WorldConfig, derive_spatial_observability, ids::CellId,
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
            spatial_reference: "LOCAL_CS[founder-init-test]".to_owned(),
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

fn mechanisms(environment_seed: u64, population_seed: u64) -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "spatial-founder-initialization-test-v1",
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

fn founder_definition(
    initialization_id: &str,
    first_household_cell: u64,
    second_household_cell: u64,
) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        initialization_id,
        ParameterProvenance::EvidenceInformed,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(first_household_cell),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(second_household_cell),
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
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: -(27 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(4),
                birth_day: -(29 * 365),
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

fn experiment(seed: u64, definition: FounderPopulationDefinition) -> ExperimentConfig {
    ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(4).with_max_person_records(100))
        .with_founder_population(definition)
        .with_demography(no_event_demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn declared_residences_are_exact_and_population_seed_is_noncausal() {
    let landscape = landscape();
    let definition = founder_definition("declared-spatial-founders-v1", 1, 4);
    let first = SpatialLandscapeSimulation::new(
        experiment(9_001, definition.clone()),
        landscape.clone(),
        mechanisms(7_001, 8_001),
    )
    .expect("first declared-founder spatial simulation");
    let second = SpatialLandscapeSimulation::new(
        experiment(9_001, definition),
        landscape.clone(),
        mechanisms(7_001, 8_999),
    )
    .expect("second declared-founder spatial simulation");

    assert_eq!(first.world().digest64(), second.world().digest64());
    assert_eq!(
        first.population().digest64(),
        second.population().digest64()
    );
    assert_ne!(
        first
            .spatial_binding()
            .environment
            .realization
            .population_seed,
        second
            .spatial_binding()
            .environment
            .realization
            .population_seed,
        "the recorded stochastic population seed may vary, but declared founder state must not consume it",
    );

    let world = first.world().clone();
    let initial_population = first.population().clone();
    let run = first.run_recorded().expect("declared-founder recorded run");
    let report = derive_spatial_observability(
        &landscape,
        &world,
        &initial_population,
        run.core_checkpoint(),
        Some(&run.checkpoint.spatial),
    )
    .expect("spatial observability");

    assert_eq!(report.cells[0].derived.initial_living_population, 2);
    assert_eq!(report.cells[3].derived.initial_living_population, 2);
    assert_eq!(report.cells[1].derived.initial_living_population, 0);
    assert_eq!(report.cells[2].derived.initial_living_population, 0);
}

#[test]
fn declared_residence_outside_spatial_world_is_rejected() {
    let definition = founder_definition("invalid-declared-spatial-founders-v1", 1, 5);
    let error = SpatialLandscapeSimulation::new(
        experiment(9_010, definition),
        landscape(),
        mechanisms(7_010, 8_010),
    )
    .expect_err("cell 5 is outside a 2x2 world");

    assert!(
        error.to_string().contains("location") || error.to_string().contains("cell"),
        "unexpected error: {error}",
    );
}

#[test]
fn controlled_initial_layouts_do_not_converge_without_a_relaxation_mechanism() {
    let landscape = landscape();
    let mechanisms = mechanisms(7_100, 8_100);

    let concentrated = SpatialLandscapeSimulation::new(
        experiment(9_100, founder_definition("concentrated-founders-v1", 1, 1)),
        landscape.clone(),
        mechanisms.clone(),
    )
    .expect("concentrated simulation");
    let split = SpatialLandscapeSimulation::new(
        experiment(9_100, founder_definition("split-founders-v1", 1, 4)),
        landscape.clone(),
        mechanisms,
    )
    .expect("split simulation");

    let concentrated_world = concentrated.world().clone();
    let concentrated_initial = concentrated.population().clone();
    let concentrated_run = concentrated
        .run_recorded()
        .expect("concentrated recorded run");
    let concentrated_report = derive_spatial_observability(
        &landscape,
        &concentrated_world,
        &concentrated_initial,
        concentrated_run.core_checkpoint(),
        Some(&concentrated_run.checkpoint.spatial),
    )
    .expect("concentrated observability");

    let split_world = split.world().clone();
    let split_initial = split.population().clone();
    let split_run = split.run_recorded().expect("split recorded run");
    let split_report = derive_spatial_observability(
        &landscape,
        &split_world,
        &split_initial,
        split_run.core_checkpoint(),
        Some(&split_run.checkpoint.spatial),
    )
    .expect("split observability");

    assert_eq!(
        concentrated_report.cells[0]
            .derived
            .terminal_living_population,
        4,
    );
    assert_eq!(split_report.cells[0].derived.terminal_living_population, 2);
    assert_eq!(split_report.cells[3].derived.terminal_living_population, 2);
    assert_eq!(
        concentrated_report
            .summary
            .terminal_largest_cell_share_permille,
        Some(1_000),
    );
    assert_eq!(
        split_report.summary.terminal_largest_cell_share_permille,
        Some(500),
    );
    assert_ne!(
        concentrated_report
            .summary
            .terminal_population_herfindahl_per_million,
        split_report
            .summary
            .terminal_population_herfindahl_per_million,
        "with migration, births, deaths and resource pressure disabled, different founder layouts remain different after the nominal two-year pre-analysis interval; AnthroSim must not assume generic burn-in convergence",
    );
}
