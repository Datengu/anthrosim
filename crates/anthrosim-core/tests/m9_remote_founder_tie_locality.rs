use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, Population, PopulationConfig,
    PopulationInitialization, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, TransformDirection, World, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
};

const FOCAL_RESIDENCE: CellId = CellId::new(13);
const REMOTE_RESIDENCE: CellId = CellId::new(1);
const TERRAIN_DOMAIN: LandscapeValueDomain = LandscapeValueDomain {
    min: 0,
    max: 65_000,
};

fn movement_values() -> Vec<i32> {
    let mut values = vec![0_i32; 25];
    for index in [0_usize, 1, 5] {
        values[index] = 65_000;
    }
    values
}

fn landscape() -> LandscapeBundle {
    LandscapeBundle::new(
        5,
        5,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 1,
            cell_size_y: 1,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[av5-001-m9-locality]".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "terrain".to_owned(),
            role: LandscapeLayerRole::TerrainTraversal,
            unit: "normalized_index".to_owned(),
            value_domain: Some(TERRAIN_DOMAIN),
            evidence_input_id: None,
            values: movement_values().into_iter().map(Some).collect(),
        }],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    SpatialMechanismConfig::new(
        "av5-001-m9-locality",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "terrain",
            "normalized_index",
            TERRAIN_DOMAIN,
            1_000,
            65_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    )
    .with_run_realization(SpatialRunRealization::new(50_001, 60_001))
}

fn focal_person() -> FounderPerson {
    FounderPerson {
        id: PersonId::new(1),
        birth_day: -(20 * 365),
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(include_remote: bool) -> FounderPopulationDefinition {
    let mut households = vec![FounderHousehold {
        id: HouseholdId::new(1),
        location: FOCAL_RESIDENCE,
    }];
    let mut people = vec![focal_person()];
    if include_remote {
        households.push(FounderHousehold {
            id: HouseholdId::new(2),
            location: REMOTE_RESIDENCE,
        });
        people.push(FounderPerson {
            id: PersonId::new(2),
            birth_day: -(40 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(2),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        });
    }
    FounderPopulationDefinition::new(
        if include_remote {
            "av5-001-augmented"
        } else {
            "av5-001-baseline"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
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
        "av5-001-tied-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(8), CellId::new(18)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "av5-001-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        1,
    )
    .unwrap();
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::new(
            "av5-001-travel",
            ParameterProvenance::SyntheticValidation,
            3_000,
            10_000,
        )
        .unwrap(),
    )
    .unwrap()
}

fn experiment(seed: u64, include_remote: bool) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(5, 5))
        .with_population(
            PopulationConfig::new(if include_remote { 2 } else { 1 })
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(16),
        )
        .with_founder_population(founders(include_remote))
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

fn controlled_world() -> World {
    let values = movement_values();
    let movement = values
        .into_iter()
        .map(|value| {
            if value == 65_000 {
                65_000_u16
            } else {
                1_000_u16
            }
        })
        .collect::<Vec<_>>();
    World::generate(WorldConfig::new(5, 5), RngFactory::new(50_001))
        .unwrap()
        .with_model_field_overlay(Some(&movement), None, None)
        .unwrap()
}

fn serialized_focal_rank(include_remote: bool) -> u64 {
    let world = controlled_world();
    let definition = founders(include_remote);
    let population = Population::initialize_declared_founder_state_v1(
        PopulationConfig::new(if include_remote { 2 } else { 1 })
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
            .with_max_person_records(16),
        &definition,
        &world,
        &demography(),
    )
    .unwrap();
    serde_json::to_value(population).unwrap()["stochasticCouplingRanks"][0]
        .as_u64()
        .unwrap()
}

fn focal_departure(seed: u64, include_remote: bool) -> (CellId, u64) {
    let run = SpatialLandscapeSimulation::new(
        experiment(seed, include_remote),
        landscape(),
        mechanisms(),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let departure = run
        .events()
        .events
        .iter()
        .find_map(|record| match record.event {
            EventKind::TemporaryJourneyDeparted {
                household,
                residence,
                destination,
                destination_tie_coupling_key: Some(coupling_key),
                ..
            } if household == HouseholdId::new(1) => Some((residence, destination, coupling_key)),
            _ => None,
        })
        .expect("focal household must depart from the tied origin");
    assert_eq!(departure.0, FOCAL_RESIDENCE);
    (departure.1, departure.2)
}

#[test]
fn isolated_remote_founder_does_not_change_authoritative_m9_tie_realization() {
    // Preserve the discovery mechanism as a limiting control: the population-wide ranks still
    // renumber because that identity remains authoritative for other stochastic mechanisms.
    assert_eq!(serialized_focal_rank(false), 1);
    assert_eq!(serialized_focal_rank(true), 2);

    for seed in 0..=1_023_u64 {
        let baseline = focal_departure(seed, false);
        let augmented = focal_departure(seed, true);
        assert_eq!(
            baseline, augmented,
            "unreachable unrelated founder perturbed focal M9 tie realization at process seed {seed}: baseline={baseline:?}, augmented={augmented:?}"
        );
    }
}
