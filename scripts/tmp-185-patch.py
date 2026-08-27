#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match in {path}: found {count}\n{old}")
    path.write_text(text.replace(old, new, 1))


spatial = ROOT / "crates/anthrosim-core/src/spatial_simulation.rs"
replace_once(
    spatial,
    """        let landscape_binding = LandscapeBinding::from_bundle(&landscape)?;\n        validate_grid_match(&config, &landscape_binding)?;\n        if let Some(evidence) = &config.evidence {\n""",
    """        let landscape_binding = LandscapeBinding::from_bundle(&landscape)?;\n        validate_grid_match(&config, &landscape_binding)?;\n        validate_movement_grid_geometry(&config, &landscape)?;\n        if let Some(evidence) = &config.evidence {\n""",
)
replace_once(
    spatial,
    """        validate_grid_match(\n            &checkpoint.core_checkpoint.experiment,\n            &checkpoint.landscape,\n        )?;\n        if let Some(evidence) = &checkpoint.core_checkpoint.experiment.evidence {\n""",
    """        validate_grid_match(\n            &checkpoint.core_checkpoint.experiment,\n            &checkpoint.landscape,\n        )?;\n        validate_movement_grid_geometry(&checkpoint.core_checkpoint.experiment, &landscape)?;\n        if let Some(evidence) = &checkpoint.core_checkpoint.experiment.evidence {\n""",
)
replace_once(
    spatial,
    """    run.manifest.landscape.validate_bundle(landscape)?;\n    validate_core_checkpoint_header(&run.checkpoint.core_checkpoint)?;\n    validate_experiment(&run.checkpoint.core_checkpoint.experiment)?;\n\n    let world = reconstruct_world(\n""",
    """    run.manifest.landscape.validate_bundle(landscape)?;\n    validate_core_checkpoint_header(&run.checkpoint.core_checkpoint)?;\n    validate_experiment(&run.checkpoint.core_checkpoint.experiment)?;\n    validate_movement_grid_geometry(&run.checkpoint.core_checkpoint.experiment, landscape)?;\n\n    let world = reconstruct_world(\n""",
)
replace_once(
    spatial,
    """fn fixed_schedule_boundary_day(year_start_day: u64, index: u16, periods: u16) -> Option<u64> {\n""",
    """fn validate_movement_grid_geometry(\n    config: &ExperimentConfig,\n    landscape: &LandscapeBundle,\n) -> Result<(), SpatialLandscapeError> {\n    if landscape.geometry.has_square_cells()\n        || (!config.migration.enabled && config.temporary_mobility.is_none())\n    {\n        return Ok(());\n    }\n    Err(SpatialLandscapeError::RectangularMovementGrid {\n        cell_size_x: landscape.geometry.cell_size_x,\n        cell_size_y: landscape.geometry.cell_size_y,\n        coordinate_unit: landscape.geometry.coordinate_unit.clone(),\n    })\n}\n\nfn fixed_schedule_boundary_day(year_start_day: u64, index: u16, periods: u16) -> Option<u64> {\n""",
)
replace_once(
    spatial,
    """    #[error(\n        \"declared founder genealogy is unspecified while the active migration model gives kin non-zero weight\"\n    )]\n    FounderKinStateUnspecified,\n    #[error(\"spatial binding schema {found} is unsupported; supported schema is {supported}\")]\n""",
    """    #[error(\n        \"declared founder genealogy is unspecified while the active migration model gives kin non-zero weight\"\n    )]\n    FounderKinStateUnspecified,\n    #[error(\n        \"grid-step M4/M9 movement requires square landscape cells; found {cell_size_x} by {cell_size_y} {coordinate_unit} cells\"\n    )]\n    RectangularMovementGrid {\n        cell_size_x: u64,\n        cell_size_y: u64,\n        coordinate_unit: String,\n    },\n    #[error(\"spatial binding schema {found} is unsupported; supported schema is {supported}\")]\n""",
)

mechanisms = ROOT / "crates/anthrosim-core/src/spatial_mechanisms.rs"
replace_once(
    mechanisms,
    """/// v2 separates the environment and stochastic founder realizations from the core process seed.\n/// Synthetic execution outside the spatial host continues to use the independent core\n/// `MODEL_SEMANTICS_ID` contract.\npub const SPATIAL_MODEL_SEMANTICS_ID: &str = \"anthrosim-spatial-transform-semantics-v2\";\n""",
    """/// v3 binds spatial execution to the unambiguous landscape geometry-v2 convention and prevents\n/// M4/M9 grid-step movement on rectangular cells, where equal cardinal steps would otherwise\n/// represent unequal physical distances. Physical resolution normalization remains a separate\n/// #203 contract. Synthetic execution outside the spatial host continues to use the independent\n/// core `MODEL_SEMANTICS_ID` contract.\npub const SPATIAL_MODEL_SEMANTICS_ID: &str = \"anthrosim-spatial-transform-semantics-v3\";\n""",
)

test_path = ROOT / "crates/anthrosim-core/tests/spatial_grid_geometry.rs"
if test_path.exists():
    raise SystemExit(f"refusing to overwrite existing {test_path}")
test_path.write_text(r'''use anthrosim_core::{
    ExperimentConfig, FocalRegion, FocalRegionSource, GridGeometry, LandscapeBundle,
    LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, MigrationConfig, NoDataPolicy,
    PopulationConfig, ResourceConfig, SpatialFieldTransform, SpatialLandscapeError,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, TransformDirection, WorldConfig, ids::CellId,
};

fn layer(id: &str, role: LandscapeLayerRole) -> LandscapeLayer {
    LandscapeLayer {
        layer_id: id.to_owned(),
        role,
        unit: "normalized_index".to_owned(),
        value_domain: Some(LandscapeValueDomain { min: 0, max: 1_000 }),
        evidence_input_id: None,
        values: vec![Some(500), Some(500)],
    }
}

fn landscape(cell_size_x: u64, cell_size_y: u64) -> LandscapeBundle {
    LandscapeBundle::new(
        2,
        1,
        GridGeometry {
            origin_x: 1_000,
            origin_y: 2_000,
            cell_size_x,
            cell_size_y,
            coordinate_unit: "metre".to_owned(),
            spatial_reference: "LOCAL_CS[grid-geometry-test]".to_owned(),
        },
        vec![
            layer("terrain", LandscapeLayerRole::TerrainTraversal),
            layer("water", LandscapeLayerRole::WaterAccessibility),
            layer("resources", LandscapeLayerRole::ResourceOpportunity),
        ],
    )
}

fn mechanisms() -> SpatialMechanismConfig {
    let domain = LandscapeValueDomain { min: 0, max: 1_000 };
    SpatialMechanismConfig::new(
        "grid-geometry-test-v1",
        vec![
            SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "normalized_index",
                domain,
                1_000,
                2_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::WaterAccess,
                "water",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
            SpatialFieldTransform::new(
                SpatialTargetField::BaseProductivity,
                "resources",
                "normalized_index",
                domain,
                0,
                1_000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            ),
        ],
    )
}

fn base_config() -> ExperimentConfig {
    ExperimentConfig::new(18_500, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(PopulationConfig::new(8).with_max_person_records(100))
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn rectangular_grid_allows_nonmovement_spatial_execution() {
    SpatialLandscapeSimulation::new(base_config(), landscape(10, 20), mechanisms())
        .expect("resource-only spatial execution does not interpret cardinal steps as distance");
}

#[test]
fn rectangular_grid_rejects_active_permanent_migration() {
    let config = base_config()
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(true));
    assert!(matches!(
        SpatialLandscapeSimulation::new(config, landscape(10, 20), mechanisms()),
        Err(SpatialLandscapeError::RectangularMovementGrid {
            cell_size_x: 10,
            cell_size_y: 20,
            ..
        })
    ));
}

#[test]
fn rectangular_grid_rejects_temporary_travel() {
    let region = FocalRegion::new(
        "grid-geometry-region",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .expect("region");
    let schedule = TemporaryMobilitySchedule::new(
        "grid-geometry-schedule",
        TemporaryTriggerTiming::DepartureDay,
        vec![0],
        1,
    )
    .expect("schedule");
    let temporary = TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility");
    let config = base_config().with_temporary_mobility(temporary);

    assert!(matches!(
        SpatialLandscapeSimulation::new(config, landscape(10, 20), mechanisms()),
        Err(SpatialLandscapeError::RectangularMovementGrid {
            cell_size_x: 10,
            cell_size_y: 20,
            ..
        })
    ));
}

#[test]
fn square_grid_preserves_active_grid_step_movement() {
    let config = base_config()
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(true));
    SpatialLandscapeSimulation::new(config, landscape(10, 10), mechanisms())
        .expect("square cells preserve existing grid-step movement semantics");
}
''')
