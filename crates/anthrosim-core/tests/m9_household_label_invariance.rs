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

fn destination_for_household_label(seed: u64, household: HouseholdId) -> CellId {
    let world = uniform_world();
    let region = FocalRegion::new(
        "m9-household-label-invariance",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(8)],
    )
    .unwrap();
    let model = TemporaryTravelModel::new(
        "m9-household-label-invariance",
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
fn exact_original_style_household_relabel_sweep_is_invariant() {
    for seed in 1..=1_000 {
        assert_eq!(
            destination_for_household_label(seed, HouseholdId::new(1)),
            destination_for_household_label(seed, HouseholdId::new(2)),
            "label-neutral M9 compatibility resolution diverged at seed {seed}"
        );
    }
}

#[test]
fn scientific_coupling_keys_retain_non_degenerate_tie_diversity() {
    let world = uniform_world();
    let region = FocalRegion::new(
        "m9-scientific-key-diversity",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(8)],
    )
    .unwrap();
    let table = TemporaryTravelModel::synthetic_validation_v1()
        .derive_table_with_tie_seed(&region, &world, 91)
        .unwrap();
    let mut top = 0_u32;
    let mut bottom = 0_u32;
    for key in 0..512_u64 {
        let resolution = table
            .resolution_for_coupling_key(CellId::new(5), key, 0)
            .unwrap();
        match resolution {
            TemporaryTravelResolution::Reachable { destination, .. }
                if destination == CellId::new(2) =>
            {
                top += 1
            }
            TemporaryTravelResolution::Reachable { destination, .. }
                if destination == CellId::new(8) =>
            {
                bottom += 1
            }
            other => panic!("unexpected tied resolution {other:?}"),
        }
    }
    assert!(top > 0 && bottom > 0, "top={top}, bottom={bottom}");
    assert!(top.abs_diff(bottom) < 128, "top={top}, bottom={bottom}");
}
