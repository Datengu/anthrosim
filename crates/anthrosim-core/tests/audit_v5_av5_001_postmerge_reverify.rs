use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
    MigrationConfig, NoDataPolicy, ParameterProvenance, Population, PopulationConfig,
    PopulationInitialization, ReproductiveSex, ResourceConfig, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialRunRealization, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTravelResolution, TemporaryTriggerTiming, TransformDirection, World, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
};

const FOCAL_CELL: CellId = CellId::new(13);
const REMOTE_CELL: CellId = CellId::new(1);
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

fn controlled_world() -> World {
    let movement = movement_values()
        .into_iter()
        .map(|value| if value == 65_000 { 65_000_u16 } else { 1_000_u16 })
        .collect::<Vec<_>>();
    World::generate(WorldConfig::new(5, 5), RngFactory::new(50_001))
        .unwrap()
        .with_model_field_overlay(Some(&movement), None, None)
        .unwrap()
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
            spatial_reference: "LOCAL_CS[av5-001-postmerge-reverify]".to_owned(),
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
        "av5-001-postmerge-reverify",
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
        location: FOCAL_CELL,
    }];
    let mut people = vec![focal_person()];
    if include_remote {
        households.push(FounderHousehold {
            id: HouseholdId::new(2),
            location: REMOTE_CELL,
        });
        people.push(FounderPerson {
            id: PersonId::new(2),
            // Preserve the discovery attack: an older isolated founder sorts ahead of the
            // unchanged focal founder in the population-wide stochastic-coupling rank order.
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
            "av5-001-postmerge-augmented"
        } else {
            "av5-001-postmerge-baseline"
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

fn region() -> FocalRegion {
    FocalRegion::new(
        "av5-001-postmerge-tied-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(8), CellId::new(18)],
    )
    .unwrap()
}

fn travel_model() -> TemporaryTravelModel {
    TemporaryTravelModel::new(
        "av5-001-postmerge-travel",
        ParameterProvenance::SyntheticValidation,
        3_000,
        10_000,
    )
    .unwrap()
}

fn temporary_mobility() -> TemporaryMobilityConfig {
    let schedule = TemporaryMobilitySchedule::new(
        "av5-001-postmerge-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        1,
    )
    .unwrap();
    TemporaryMobilityConfig::new(region(), schedule, travel_model()).unwrap()
}

fn population(include_remote: bool) -> Population {
    let world = controlled_world();
    Population::initialize_declared_founder_state_v1(
        PopulationConfig::new(if include_remote { 2 } else { 1 })
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
            .with_max_person_records(16),
        &founders(include_remote),
        &world,
        &demography(),
    )
    .unwrap()
}

fn serialized_focal_rank(include_remote: bool) -> u64 {
    serde_json::to_value(population(include_remote)).unwrap()["stochasticCouplingRanks"][0]
        .as_u64()
        .unwrap()
}

fn forced_old_rank_destination(
    table: &anthrosim_core::TemporaryTravelTable,
    obsolete_rank_key: u64,
) -> CellId {
    match table
        .resolution_for_coupling_key(FOCAL_CELL, obsolete_rank_key, 0)
        .expect("focal origin must resolve")
    {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => panic!("focal origin unexpectedly unreachable"),
    }
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

fn authoritative_focal_departure(seed: u64, include_remote: bool) -> (CellId, u64) {
    let run = SpatialLandscapeSimulation::new(
        experiment(seed, include_remote),
        landscape(),
        mechanisms(),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    run.events()
        .events
        .iter()
        .find_map(|record| match record.event {
            EventKind::TemporaryJourneyDeparted {
                household,
                residence,
                destination,
                destination_tie_coupling_key: Some(coupling_key),
                ..
            } if household == HouseholdId::new(1) => {
                assert_eq!(residence, FOCAL_CELL);
                Some((destination, coupling_key))
            }
            _ => None,
        })
        .expect("unchanged focal household must emit an M9 departure")
}

#[test]
fn av5_001_isolated_founder_locality_holds_after_v34_repair() {
    let baseline = population(false);
    let augmented = population(true);
    assert_eq!(baseline.person(PersonId::new(1)), augmented.person(PersonId::new(1)));

    // Preserve the original discovery mechanism as a positive control. Population-wide ranks
    // still legitimately renumber 1 -> 2 because they remain authoritative for other mechanisms.
    let baseline_rank = serialized_focal_rank(false);
    let augmented_rank = serialized_focal_rank(true);
    assert_eq!(baseline_rank, 1);
    assert_eq!(augmented_rank, 2);

    // Preserve the original two-way tie attack itself. If the obsolete global ranks are manually
    // injected into the generic keyed resolver, the construction must remain non-degenerate and
    // produce divergences. The v34 claim is specifically that authoritative M9 no longer injects
    // those population-wide ordinal ranks as its household-local causal key.
    let world = controlled_world();
    let model = travel_model();
    let region = region();
    let mut obsolete_rank_divergences = 0_u32;
    for tie_seed in 0..=1_023_u64 {
        let table = model
            .derive_table_with_tie_seed(&region, &world, tie_seed)
            .unwrap();
        assert_eq!(table.equal_cost_destination_count(FOCAL_CELL), Some(2));
        assert_eq!(
            table.resolution(REMOTE_CELL),
            Some(TemporaryTravelResolution::Unreachable)
        );
        let a = forced_old_rank_destination(&table, baseline_rank);
        let b = forced_old_rank_destination(&table, augmented_rank);
        if a != b {
            obsolete_rank_divergences += 1;
        }
    }
    assert!(
        obsolete_rank_divergences > 0,
        "the restored discovery construction became degenerate: obsolete rank keys 1 and 2 never diverged"
    );

    // Independent post-merge oracle: under actual v34 execution, the unreachable founder must
    // not change either the authoritative emitted M9 household-local tie key or its destination.
    for seed in 0..=1_023_u64 {
        let baseline_departure = authoritative_focal_departure(seed, false);
        let augmented_departure = authoritative_focal_departure(seed, true);
        assert_eq!(
            baseline_departure, augmented_departure,
            "AV5-001 recurred at process seed {seed}: baseline={baseline_departure:?}, augmented={augmented_departure:?}"
        );
    }
}
