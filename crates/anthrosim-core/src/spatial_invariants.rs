use thiserror::Error;

use crate::{
    InvariantError, LandscapeBundle, SpatialLandscapeError, SpatialLandscapeRecordedRun, World,
    rng::RngFactory, transform_landscape, validate_run_artifacts_with_world,
};

#[derive(Debug, Error)]
pub enum SpatialInvariantError {
    #[error(transparent)]
    Spatial(#[from] SpatialLandscapeError),
    #[error(transparent)]
    Core(#[from] InvariantError),
}

/// Validate one completed transformed-landscape run with both its spatial-specific contract and
/// the complete core scientific-invariant suite.
///
/// The spatial host owns proof that the normalized landscape, transformation configuration and
/// transformed-world identity are coherent. The shared core validator then checks the same
/// population, resource, migration, authoritative-event, M9-history, metric, manifest/statistics
/// and terminal-state invariants used by ordinary synthetic runs, but against the reconstructed
/// transformed authoritative world rather than a regenerated synthetic baseline.
pub fn validate_spatial_landscape_recorded_run(
    run: &SpatialLandscapeRecordedRun,
    landscape: &LandscapeBundle,
) -> Result<(), SpatialInvariantError> {
    crate::spatial_simulation::validate_spatial_landscape_recorded_run(run, landscape)?;
    let world = reconstruct_authoritative_world(run, landscape)?;
    validate_run_artifacts_with_world(run.core_manifest(), run.core_checkpoint(), &world)?;
    Ok(())
}

fn reconstruct_authoritative_world(
    run: &SpatialLandscapeRecordedRun,
    landscape: &LandscapeBundle,
) -> Result<World, SpatialLandscapeError> {
    let checkpoint = run.core_checkpoint();
    let config = &checkpoint.experiment;

    landscape.validate_evidence_context(config.evidence.as_ref())?;
    run.checkpoint
        .spatial
        .config
        .validate_evidence_links(config.evidence.as_ref())?;
    let overlay = transform_landscape(landscape, &run.checkpoint.spatial.config)?;
    let world = World::generate(
        config.world,
        RngFactory::new(
            run.checkpoint
                .spatial
                .environment
                .realization
                .environment_seed,
        ),
    )?
    .with_model_field_overlay(
        overlay.movement_cost.as_deref(),
        overlay.water_access.as_deref(),
        overlay.base_productivity.as_deref(),
    )?;
    Ok(world)
}
