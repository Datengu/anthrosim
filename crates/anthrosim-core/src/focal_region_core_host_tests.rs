use crate::{
    config::{ExperimentConfig, PopulationConfig, WorldConfig},
    focal_region::{FocalRegion, FocalRegionSource},
    ids::CellId,
    simulation::{Simulation, SimulationError},
    temporary_mobility::{
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTriggerTiming,
    },
    temporary_travel::TemporaryTravelModel,
};

fn temporary_mobility(region: FocalRegion) -> TemporaryMobilityConfig {
    TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            "core-host-focal-region-test",
            TemporaryTriggerTiming::DepartureDay,
            vec![0],
            1,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn core_config(region: FocalRegion) -> ExperimentConfig {
    ExperimentConfig::new(18_701, 1)
        .with_world(WorldConfig::new(2, 2))
        .with_population(PopulationConfig::new(4).with_target_household_size(2))
        .with_temporary_mobility(temporary_mobility(region))
}

#[test]
fn core_simulation_rejects_landscape_mask_provenance_without_bound_landscape() {
    let region = FocalRegion::new(
        "claimed-mask",
        FocalRegionSource::LandscapeMask {
            layer_id: "focal-mask".to_owned(),
            evidence_input_id: "mask-input".to_owned(),
        },
        vec![CellId::new(2), CellId::new(4)],
    )
    .unwrap();
    assert!(matches!(
        Simulation::new(core_config(region)),
        Err(SimulationError::LandscapeMaskRegionRequiresSpatialLandscape)
    ));
}

#[test]
fn core_simulation_still_accepts_explicit_synthetic_focal_regions() {
    let region = FocalRegion::new(
        "synthetic-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2), CellId::new(4)],
    )
    .unwrap();
    Simulation::new(core_config(region)).unwrap();
}
