use anthrosim_core::{
    DemographyConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, ParameterProvenance, Population, PopulationConfig,
    PopulationInitialization, ReproductiveSex, TemporaryTravelModel, TemporaryTravelResolution,
    World, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
};

const FOCAL_CELL: CellId = CellId::new(13);
const REMOTE_CELL: CellId = CellId::new(1);

fn controlled_world() -> World {
    let mut movement_cost = [1_000_u16; 25];
    // Isolate cell 1 behind an impassable corner barrier. The focal household at cell 13 and its
    // two tied destinations (8 and 18) remain in the uniform traversable component.
    for index in [0_usize, 1, 5] {
        movement_cost[index] = 65_000;
    }
    World::generate(WorldConfig::new(5, 5), RngFactory::new(50_001))
        .unwrap()
        .with_model_field_overlay(Some(&movement_cost), None, None)
        .unwrap()
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

fn founder_definition(include_remote_isolated_founder: bool) -> FounderPopulationDefinition {
    let mut households = vec![FounderHousehold {
        id: HouseholdId::new(1),
        location: FOCAL_CELL,
    }];
    let mut people = vec![focal_person()];
    if include_remote_isolated_founder {
        households.push(FounderHousehold {
            id: HouseholdId::new(2),
            location: REMOTE_CELL,
        });
        people.push(FounderPerson {
            id: PersonId::new(2),
            // Older than the unchanged focal founder so canonical scientific ordering places this
            // isolated record before the focal record even though the focal PersonId is unchanged.
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
        if include_remote_isolated_founder {
            "audit-v5-area-a-augmented"
        } else {
            "audit-v5-area-a-baseline"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
    )
}

fn population(include_remote_isolated_founder: bool, world: &World) -> Population {
    let count = if include_remote_isolated_founder { 2 } else { 1 };
    Population::initialize_declared_founder_state_v1(
        PopulationConfig::new(count)
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
            .with_max_person_records(16),
        &founder_definition(include_remote_isolated_founder),
        world,
        &DemographyConfig::synthetic_validation_v1(),
    )
    .unwrap()
}

fn serialized_coupling_rank(population: &Population, person: PersonId) -> u64 {
    let value = serde_json::to_value(population).unwrap();
    value["stochasticCouplingRanks"]
        .as_array()
        .unwrap()[usize::try_from(person.0 - 1).unwrap()]
        .as_u64()
        .unwrap()
}

fn tied_destination(table: &anthrosim_core::TemporaryTravelTable, coupling_key: u64) -> CellId {
    match table
        .resolution_for_coupling_key(FOCAL_CELL, coupling_key, 0)
        .expect("focal origin must resolve")
    {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => panic!("focal origin unexpectedly unreachable"),
    }
}

#[test]
fn isolated_remote_founder_must_not_change_focal_equal_cost_destination() {
    let world = controlled_world();
    let baseline = population(false, &world);
    let augmented = population(true, &world);

    // The focal scientific record is byte-for-byte unchanged in the founder definition: same
    // PersonId, household, age, sex, residence, condition and genealogy. Only an isolated second
    // household/person has been appended.
    assert_eq!(baseline.person(PersonId::new(1)), augmented.person(PersonId::new(1)));

    let baseline_key = serialized_coupling_rank(&baseline, PersonId::new(1));
    let augmented_key = serialized_coupling_rank(&augmented, PersonId::new(1));
    assert_eq!(baseline_key, 1);
    assert_eq!(augmented_key, 2);

    let region = FocalRegion::new(
        "audit-v5-area-a-tied-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(8), CellId::new(18)],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "audit-v5-area-a-tie-model",
        ParameterProvenance::SyntheticValidation,
        3_000,
        10_000,
    )
    .unwrap();

    let mut divergences = Vec::new();
    for tie_seed in 0..=1_023_u64 {
        let table = model
            .derive_table_with_tie_seed(&region, &world, tie_seed)
            .unwrap();
        assert_eq!(table.equal_cost_destination_count(FOCAL_CELL), Some(2));
        assert_eq!(table.resolution(REMOTE_CELL), Some(TemporaryTravelResolution::Unreachable));

        let baseline_destination = tied_destination(&table, baseline_key);
        let augmented_destination = tied_destination(&table, augmented_key);
        if baseline_destination != augmented_destination {
            divergences.push((tie_seed, baseline_destination, augmented_destination));
        }
    }

    assert!(
        divergences.is_empty(),
        "adding one unreachable, otherwise non-interacting founder renumbered the unchanged focal household coupling key {baseline_key}->{augmented_key} and changed its M9 equal-cost destination for {}/1024 tie seeds; first divergences: {:?}",
        divergences.len(),
        &divergences[..divergences.len().min(8)]
    );
}
