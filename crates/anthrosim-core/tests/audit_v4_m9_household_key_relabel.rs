use anthrosim_core::{
    FocalRegion, FocalRegionSource, ParameterProvenance, TemporaryTravelModel,
    TemporaryTravelResolution, World, WorldConfig,
    ids::{CellId, HouseholdId},
    rng::RngFactory,
};

fn uniform_world() -> World {
    World::generate(WorldConfig::new(3, 3), RngFactory::new(9_401))
        .unwrap()
        .with_model_field_overlay(Some(&[1_000; 9]), None, None)
        .unwrap()
}

fn destination_for(seed: u64, household: HouseholdId) -> CellId {
    let world = uniform_world();
    let region = FocalRegion::new(
        "m9-household-key-relabel",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(8)],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "m9-household-key-relabel",
        ParameterProvenance::SyntheticValidation,
        3_000,
        u16::MAX,
    )
    .unwrap();
    let table = model
        .derive_table_with_tie_seed(&region, &world, seed)
        .unwrap();

    assert_eq!(table.equal_cost_destination_count(CellId::new(5)), Some(2));
    match table
        .resolution_for(CellId::new(5), household, 0)
        .expect("center origin must be reachable")
    {
        TemporaryTravelResolution::Reachable { destination, .. } => destination,
        TemporaryTravelResolution::Unreachable => panic!("center origin unexpectedly unreachable"),
    }
}

#[test]
fn equal_cost_m9_destination_is_invariant_to_pure_household_relabelling() {
    let mut informative_seeds = 0_u32;
    for seed in 1..=1_000 {
        let physical_household_label_a = HouseholdId::new(1);
        let physical_household_label_b = HouseholdId::new(2);
        let a = destination_for(seed, physical_household_label_a);
        let b = destination_for(seed, physical_household_label_b);
        if a != b {
            informative_seeds += 1;
        }
        assert_eq!(
            a, b,
            "same physical household at the same origin selected different equal-cost M9 destinations after only its arbitrary HouseholdId changed at seed {seed}: A={a:?}, B={b:?}"
        );
    }
    assert!(informative_seeds > 0, "tie adversary did not exercise label-sensitive destination selection");
}
