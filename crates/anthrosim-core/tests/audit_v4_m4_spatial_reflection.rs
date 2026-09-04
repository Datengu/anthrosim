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

fn landscape(mirrored: bool) -> LandscapeBundle {
    let values = if mirrored {
        vec![Some(500), Some(100), Some(900)]
    } else {
        vec![Some(900), Some(100), Some(500)]
    };
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL:AUDIT-V4-MIRROR".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "resources".to_owned(),
            role: LandscapeLayerRole::ResourceOpportunity,
            unit: "normalized_index".to_owned(),
            value_domain: Some(DOMAIN),
            evidence_input_id: None,
            values,
        }],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "audit-v4-m4-spatial-reflection",
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
        "audit-v4-m4-spatial-reflection-founders",
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

fn migration() -> MigrationConfig {
    let mut config = MigrationConfig::synthetic_validation_v1()
        .with_enabled(true)
        .with_candidate_radius_cells(1)
        .with_decision_periods_per_year(1);
    config.condition_pressure_threshold_permille = 1_000;
    config.resource_pressure_threshold_permille = 0;
    config.minimum_utility_improvement = 0;
    config.resource_weight = 1;
    config.water_security_weight = 0;
    config.kin_weight = 0;
    config.travel_cost_weight = 0;
    config.max_uncertainty_penalty_permille = 0;
    config.relocation_risk_base_penalty_permille = 0;
    config.relocation_risk_per_cell_permille = 0;
    config.travel_condition_cost_per_cell = 0;
    config
}

fn run(seed: u64, mirrored: bool) -> (CellId, CellId) {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(1).with_max_person_records(10))
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(migration());
    let recorded = SpatialLandscapeSimulation::new(config, landscape(mirrored), mechanisms())
        .expect("spatial simulation")
        .run_recorded()
        .expect("recorded run");

    let moves = recorded
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
        .collect::<Vec<_>>();
    assert_eq!(moves.len(), 1, "seed {seed} mirrored={mirrored}: {moves:?}");
    moves[0]
}

fn mirror(cell: CellId) -> CellId {
    match cell.0 {
        1 => CellId::new(3),
        2 => CellId::new(2),
        3 => CellId::new(1),
        other => panic!("unexpected cell {other}"),
    }
}

#[test]
fn m4_weighted_choice_is_equivariant_under_horizontal_spatial_reflection() {
    for seed in 1..=256 {
        let (origin, destination) = run(seed, false);
        let (mirrored_origin, mirrored_destination) = run(seed, true);
        assert_eq!(origin, CellId::new(2));
        assert_eq!(mirrored_origin, CellId::new(2));
        assert_eq!(
            mirrored_destination,
            mirror(destination),
            "same physical one-household problem failed horizontal-reflection equivariance at seed {seed}: canonical destination={destination:?}, mirrored destination={mirrored_destination:?}"
        );
    }
}
