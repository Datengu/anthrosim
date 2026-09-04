use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, GridGeometry, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy, ParameterProvenance,
    PopulationConfig, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TransformDirection, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const DOMAIN: LandscapeValueDomain = LandscapeValueDomain { min: 0, max: 1_000 };

fn landscape(width: u32, height: u32, values: Vec<i32>) -> LandscapeBundle {
    LandscapeBundle::new(
        width,
        height,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:AV4-009".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "resources".to_owned(),
            role: LandscapeLayerRole::ResourceOpportunity,
            unit: "normalized_index".to_owned(),
            value_domain: Some(DOMAIN),
            evidence_input_id: None,
            values: values.into_iter().map(Some).collect(),
        }],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "av4-009-m4-spatial-isomorphism",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::BaseProductivity,
            "resources",
            "normalized_index",
            DOMAIN,
            0,
            1_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    )
    .with_run_realization(SpatialRunRealization::new(71_001, 81_001))
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "av4-009-founders",
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
            condition_permille: 500,
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

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(10_000)
        .with_initial_stock_units_per_productivity(10)
        .with_annual_regeneration_units_per_productivity(1)
        .with_productivity_scale_permille(1_000);
    config.periods_per_year = 1;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn migration(max_uncertainty_penalty_permille: u16) -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_candidate_radius_cells(1)
        .with_decision_periods_per_year(1);
    config.condition_pressure_threshold_permille = 1_000;
    config.resource_pressure_threshold_permille = 0;
    config.minimum_utility_improvement = 0;
    config.resource_weight = 4;
    config.water_security_weight = 0;
    config.kin_weight = 0;
    config.travel_cost_weight = 0;
    config.max_uncertainty_penalty_permille = max_uncertainty_penalty_permille;
    config.relocation_risk_base_penalty_permille = 0;
    config.relocation_risk_per_cell_permille = 0;
    config.travel_condition_cost_per_cell = 0;
    config
}

fn run(
    seed: u64,
    width: u32,
    height: u32,
    values: Vec<i32>,
    max_uncertainty_penalty_permille: u16,
) -> Vec<(CellId, CellId)> {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(width, height))
        .with_population(PopulationConfig::new(1).with_max_person_records(10))
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(migration(max_uncertainty_penalty_permille));
    SpatialLandscapeSimulation::new(config, landscape(width, height, values), mechanisms())
        .expect("spatial simulation")
        .run_recorded()
        .expect("recorded run")
        .events()
        .events
        .iter()
        .filter_map(|event| match event.event {
            EventKind::HouseholdMigration {
                origin,
                destination,
                ..
            } => Some((origin, destination)),
            _ => None,
        })
        .collect()
}

fn mirror_three(cell: CellId) -> CellId {
    match cell.0 {
        1 => CellId::new(3),
        2 => CellId::new(2),
        3 => CellId::new(1),
        other => panic!("unexpected cell {other}"),
    }
}

fn assert_reflection_sweep(width: u32, height: u32, uncertainty: u16) {
    for seed in 1..=256 {
        let canonical = run(seed, width, height, vec![900, 100, 500], uncertainty);
        let reflected = run(seed, width, height, vec![500, 100, 900], uncertainty);
        assert_eq!(canonical.len(), 1, "seed {seed}: canonical={canonical:?}");
        assert_eq!(reflected.len(), 1, "seed {seed}: reflected={reflected:?}");
        assert_eq!(canonical[0].0, CellId::new(2));
        assert_eq!(reflected[0].0, CellId::new(2));
        assert_eq!(
            reflected[0].1,
            mirror_three(canonical[0].1),
            "same physical problem failed spatial-reflection equivariance at seed {seed}, width={width}, height={height}, uncertainty={uncertainty}: canonical={canonical:?}, reflected={reflected:?}"
        );
    }
}

#[test]
fn m4_weighted_choice_is_equivariant_under_horizontal_reflection() {
    assert_reflection_sweep(3, 1, 0);
}

#[test]
fn m4_weighted_choice_is_equivariant_under_vertical_reflection() {
    assert_reflection_sweep(1, 3, 0);
}

#[test]
fn m4_candidate_uncertainty_is_equivariant_under_reflection() {
    assert_reflection_sweep(3, 1, 80);
}

#[test]
fn m4_does_not_invent_an_orientation_for_exactly_indistinguishable_destinations() {
    for seed in 1..=256 {
        let moves = run(seed, 3, 1, vec![900, 100, 900], 80);
        assert!(
            moves.is_empty(),
            "seed {seed}: exact left/right M4 symmetry must not be broken by CellId/container order: {moves:?}"
        );
    }
}
