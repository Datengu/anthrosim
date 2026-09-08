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

fn landscape(resource_values: [i32; 3]) -> LandscapeBundle {
    LandscapeBundle::new(
        3,
        1,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[audit-v5-area-e-m9-reflection]".to_owned(),
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
        "audit-v5-area-e-m9-spatial-reflection",
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
        "audit-v5-area-e-m9-reflection-founders",
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
        "audit-v5-area-e-m9-reflection-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(1), CellId::new(3)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v5-area-e-m9-reflection-schedule",
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

fn experiment(seed: u64) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(3, 1))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

fn departure(seed: u64, resource_values: [i32; 3]) -> (CellId, u64) {
    let run =
        SpatialLandscapeSimulation::new(experiment(seed), landscape(resource_values), mechanisms())
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
        .expect("v33 M9 equal-cost execution must expose its coupling key");
    (departures[0].1, coupling_key)
}

fn mirror(cell: CellId) -> CellId {
    match cell.0 {
        1 => CellId::new(3),
        2 => CellId::new(2),
        3 => CellId::new(1),
        other => panic!("unexpected 3x1 cell {other}"),
    }
}

#[test]
fn m9_equal_cost_destination_is_equivariant_under_horizontal_reflection() {
    let mut mismatches = Vec::new();
    for seed in 1..=256_u64 {
        let canonical = departure(seed, [900, 100, 500]);
        let reflected = departure(seed, [500, 100, 900]);

        assert_eq!(
            canonical.1, reflected.1,
            "pure landscape reflection must not alter the focal household coupling key at seed {seed}"
        );
        if reflected.0 != mirror(canonical.0) {
            mismatches.push((seed, canonical.0, reflected.0, canonical.1));
        }
    }

    eprintln!(
        "M9 reflection mismatches={}/256; first={:?}",
        mismatches.len(),
        mismatches.iter().take(12).collect::<Vec<_>>()
    );

    assert!(
        mismatches.is_empty(),
        "same physical one-household M9 equal-cost problem is not horizontal-reflection equivariant; mismatches={}/256; first={:?}",
        mismatches.len(),
        mismatches.iter().take(12).collect::<Vec<_>>()
    );
}
