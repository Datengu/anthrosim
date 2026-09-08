use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, PopulationConfig, ReproductiveSex,
    ResourceConfig, SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
    SpatialRunRealization, SpatialTargetField, TemporaryMobilityConfig, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn layer(id: &str, role: LandscapeLayerRole, values: [i32; 3]) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(DOMAIN),
        evidence_input_id: None,
        values: values.into_iter().map(Some).collect(),
    }
}

fn landscape(width: u32, height: u32, resource_values: [i32; 3]) -> LandscapeBundle {
    LandscapeBundle::new(
        width,
        height,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[av5-004-m9-isomorphism]".to_owned(),
        },
        vec![
            layer("terrain", LandscapeLayerRole::TerrainTraversal, [0, 0, 0]),
            layer(
                "resources",
                LandscapeLayerRole::ResourceOpportunity,
                resource_values,
            ),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "av5-004-m9-spatial-isomorphism",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                DOMAIN,
                1_000,
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
    .with_run_realization(SpatialRunRealization::new(71_001, 81_001))
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "av5-004-m9-spatial-isomorphism-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(2),
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        }],
    )
}

fn demography() -> DemographyConfig {
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
        "av5-004-m9-spatial-isomorphism-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(1), CellId::new(3)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "av5-004-m9-spatial-isomorphism-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        1,
    )
    .unwrap();
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn experiment(seed: u64, width: u32, height: u32) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(width, height))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

fn departure(seed: u64, width: u32, height: u32, resource_values: [i32; 3]) -> (CellId, u64) {
    let run = SpatialLandscapeSimulation::new(
        experiment(seed, width, height),
        landscape(width, height, resource_values),
        mechanisms(),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let departures = run
        .events()
        .events
        .iter()
        .filter_map(|record| match &record.event {
            EventKind::TemporaryJourneyDeparted {
                residence,
                destination,
                destination_tie_coupling_key,
                ..
            } => Some((*residence, *destination, *destination_tie_coupling_key)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        departures.len(),
        1,
        "seed {seed}: departures={departures:?}"
    );
    assert_eq!(departures[0].0, CellId::new(2));
    let coupling_key = departures[0]
        .2
        .expect("tied M9 execution must expose its household-local coupling key");
    (departures[0].1, coupling_key)
}

fn mirror_three(cell: CellId) -> CellId {
    match cell.0 {
        1 => CellId::new(3),
        2 => CellId::new(2),
        3 => CellId::new(1),
        other => panic!("unexpected three-cell id {other}"),
    }
}

fn assert_reflection_sweep(width: u32, height: u32) {
    let mut first = 0_u32;
    let mut third = 0_u32;
    for seed in 1..=256_u64 {
        let canonical = departure(seed, width, height, [900, 100, 500]);
        let reflected = departure(seed, width, height, [500, 100, 900]);
        assert_eq!(
            canonical.1, reflected.1,
            "pure spatial reflection altered household-local M9 coupling at seed {seed}"
        );
        assert_eq!(
            reflected.0,
            mirror_three(canonical.0),
            "same physical M9 tie failed reflection equivariance at seed {seed}, {width}x{height}: canonical={canonical:?}, reflected={reflected:?}"
        );
        match canonical.0 {
            cell if cell == CellId::new(1) => first += 1,
            cell if cell == CellId::new(3) => third += 1,
            other => panic!("seed {seed}: unexpected tied destination {other:?}"),
        }
    }
    assert!(
        first > 0 && third > 0,
        "reflection-coupled ambiguity policy must preserve both marginal alternatives: first={first}, third={third}"
    );
}

#[test]
fn m9_equal_cost_destination_is_equivariant_under_horizontal_reflection() {
    assert_reflection_sweep(3, 1);
}

#[test]
fn m9_equal_cost_destination_is_equivariant_under_vertical_reflection() {
    assert_reflection_sweep(1, 3);
}

#[test]
fn exact_spatial_symmetry_remains_exchangeable_and_replay_exact() {
    let mut first = 0_u32;
    let mut third = 0_u32;
    for seed in 1..=256_u64 {
        let run = departure(seed, 3, 1, [900, 100, 900]);
        let replay = departure(seed, 3, 1, [900, 100, 900]);
        assert_eq!(run, replay, "seed {seed}: exact replay diverged");
        match run.0 {
            cell if cell == CellId::new(1) => first += 1,
            cell if cell == CellId::new(3) => third += 1,
            other => panic!("seed {seed}: unexpected symmetric destination {other:?}"),
        }
    }
    assert!(
        first > 0 && third > 0,
        "exact automorphic alternatives must remain marginally exchangeable: first={first}, third={third}"
    );
}
