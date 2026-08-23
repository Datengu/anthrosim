use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    InvariantError, InvariantReport, LandscapeBundle, LandscapeError, Population, RecordedRun,
    RunManifest, Simulation, SimulationCheckpoint, SimulationError, World,
    validate_recorded_run_invariants,
};

/// Checkpoint schema used only for M8.3 landscape-bound checkpoints.
///
/// Synthetic checkpoints remain on `SimulationCheckpoint::CURRENT_SCHEMA_VERSION`, preserving
/// their existing byte-level representation. A landscape-bound checkpoint uses a distinct
/// schema so calling `Simulation::from_checkpoint` directly cannot silently discard the bound
/// landscape. Resume must go through `LandscapeSimulation::from_checkpoint` with the same bundle.
pub const LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeBinding {
    pub schema_version: u32,
    pub landscape_identity: String,
    pub landscape_digest64: u64,
    pub width: u32,
    pub height: u32,
    pub spatial_reference: String,
    pub coordinate_unit: String,
}

impl LandscapeBinding {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn from_bundle(bundle: &LandscapeBundle) -> Result<Self, LandscapeBindingError> {
        bundle.validate()?;
        Ok(Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            landscape_identity: bundle.identity(),
            landscape_digest64: bundle.digest64(),
            width: bundle.width,
            height: bundle.height,
            spatial_reference: bundle.geometry.spatial_reference.clone(),
            coordinate_unit: bundle.geometry.coordinate_unit.clone(),
        })
    }

    pub fn validate_bundle(&self, bundle: &LandscapeBundle) -> Result<(), LandscapeBindingError> {
        bundle.validate()?;
        let actual = Self::from_bundle(bundle)?;
        if actual != *self {
            return Err(LandscapeBindingError::BindingMismatch {
                expected: self.landscape_identity.clone(),
                actual: actual.landscape_identity,
            });
        }
        Ok(())
    }
}

/// M8.3 execution boundary for an immutable normalized landscape.
///
/// The landscape is authoritative input and provenance at this stage, but its layer values are
/// deliberately behaviorally inert. M8.4 owns the explicit transformations that connect terrain,
/// water, and resource-opportunity layers to existing mechanisms.
#[derive(Debug)]
pub struct LandscapeSimulation {
    simulation: Simulation,
    landscape: LandscapeBundle,
    binding: LandscapeBinding,
}

impl LandscapeSimulation {
    pub fn new(
        config: crate::ExperimentConfig,
        landscape: LandscapeBundle,
    ) -> Result<Self, LandscapeBindingError> {
        let binding = LandscapeBinding::from_bundle(&landscape)?;
        validate_grid_match(&config, &binding)?;
        if let Some(evidence) = &config.evidence {
            evidence.validate()?;
            landscape.validate_evidence_links(evidence)?;
        }
        let simulation = Simulation::new(config)?;
        Ok(Self {
            simulation,
            landscape,
            binding,
        })
    }

    pub fn from_checkpoint(
        checkpoint: SimulationCheckpoint,
        landscape: LandscapeBundle,
    ) -> Result<Self, LandscapeBindingError> {
        if checkpoint.schema_version != LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION {
            return Err(LandscapeBindingError::UnsupportedLandscapeCheckpointSchema {
                found: checkpoint.schema_version,
                supported: LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION,
            });
        }
        let stored_binding = checkpoint
            .landscape
            .clone()
            .ok_or(LandscapeBindingError::MissingCheckpointBinding)?;
        stored_binding.validate_bundle(&landscape)?;
        validate_grid_match(&checkpoint.experiment, &stored_binding)?;
        if let Some(evidence) = &checkpoint.experiment.evidence {
            evidence.validate()?;
            landscape.validate_evidence_links(evidence)?;
        }

        let mut internal = checkpoint;
        internal.schema_version = SimulationCheckpoint::CURRENT_SCHEMA_VERSION;
        internal.landscape = None;
        let simulation = Simulation::from_checkpoint(internal)?;
        Ok(Self {
            simulation,
            landscape,
            binding: stored_binding,
        })
    }

    #[must_use]
    pub const fn world(&self) -> &World {
        self.simulation.world()
    }

    #[must_use]
    pub const fn population(&self) -> &Population {
        self.simulation.population()
    }

    #[must_use]
    pub const fn landscape(&self) -> &LandscapeBundle {
        &self.landscape
    }

    #[must_use]
    pub const fn binding(&self) -> &LandscapeBinding {
        &self.binding
    }

    pub fn checkpoint_at_year(self, target_year: u64) -> Result<SimulationCheckpoint, LandscapeBindingError> {
        let binding = self.binding;
        let mut checkpoint = self.simulation.checkpoint_at_year(target_year)?;
        bind_checkpoint(&mut checkpoint, &binding);
        Ok(checkpoint)
    }

    pub fn run_recorded(self) -> Result<RecordedRun, LandscapeBindingError> {
        let binding = self.binding;
        let mut recorded = self.simulation.run_recorded()?;
        bind_recorded_run(&mut recorded, &binding);
        validate_landscape_recorded_run_invariants(&recorded)?;
        Ok(recorded)
    }
}

fn validate_grid_match(
    config: &crate::ExperimentConfig,
    binding: &LandscapeBinding,
) -> Result<(), LandscapeBindingError> {
    if config.world.width != binding.width || config.world.height != binding.height {
        return Err(LandscapeBindingError::GridMismatch {
            world_width: config.world.width,
            world_height: config.world.height,
            landscape_width: binding.width,
            landscape_height: binding.height,
        });
    }
    Ok(())
}

fn bind_checkpoint(checkpoint: &mut SimulationCheckpoint, binding: &LandscapeBinding) {
    checkpoint.schema_version = LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION;
    checkpoint.landscape = Some(binding.clone());
}

fn bind_manifest(manifest: &mut RunManifest, binding: &LandscapeBinding) {
    manifest.landscape = Some(binding.clone());
    manifest.artifact_schemas.checkpoint = LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION;
}

fn bind_recorded_run(run: &mut RecordedRun, binding: &LandscapeBinding) {
    bind_manifest(&mut run.manifest, binding);
    bind_checkpoint(&mut run.checkpoint, binding);
}

pub fn validate_landscape_recorded_run_invariants(
    run: &RecordedRun,
) -> Result<InvariantReport, LandscapeBindingError> {
    let manifest_binding = run
        .manifest
        .landscape
        .as_ref()
        .ok_or(LandscapeBindingError::MissingManifestBinding)?;
    let checkpoint_binding = run
        .checkpoint
        .landscape
        .as_ref()
        .ok_or(LandscapeBindingError::MissingCheckpointBinding)?;
    if manifest_binding != checkpoint_binding {
        return Err(LandscapeBindingError::CrossArtifactBindingMismatch);
    }
    if run.checkpoint.schema_version != LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION
        || run.manifest.artifact_schemas.checkpoint != LANDSCAPE_BOUND_CHECKPOINT_SCHEMA_VERSION
    {
        return Err(LandscapeBindingError::CrossArtifactCheckpointSchemaMismatch);
    }

    // Reuse every pre-M8 invariant without weakening the synthetic validator. The normalized clone
    // represents the underlying M1-M7 authoritative state; the checks above validate the M8.3
    // landscape-specific cross-artifact identity around it.
    let mut normalized = run.clone();
    normalized.manifest.landscape = None;
    normalized.manifest.artifact_schemas.checkpoint = SimulationCheckpoint::CURRENT_SCHEMA_VERSION;
    normalized.checkpoint.schema_version = SimulationCheckpoint::CURRENT_SCHEMA_VERSION;
    normalized.checkpoint.landscape = None;
    Ok(validate_recorded_run_invariants(&normalized)?)
}

#[derive(Debug, Error)]
pub enum LandscapeBindingError {
    #[error(transparent)]
    Landscape(#[from] LandscapeError),
    #[error(transparent)]
    Evidence(#[from] crate::EvidenceError),
    #[error(transparent)]
    Simulation(#[from] SimulationError),
    #[error(transparent)]
    Invariant(#[from] InvariantError),
    #[error(
        "simulation grid {world_width}x{world_height} does not match bound landscape {landscape_width}x{landscape_height}"
    )]
    GridMismatch {
        world_width: u32,
        world_height: u32,
        landscape_width: u32,
        landscape_height: u32,
    },
    #[error("landscape binding mismatch: checkpoint expects {expected}, supplied bundle is {actual}")]
    BindingMismatch { expected: String, actual: String },
    #[error("landscape-bound checkpoint is missing its landscape binding")]
    MissingCheckpointBinding,
    #[error("landscape-bound manifest is missing its landscape binding")]
    MissingManifestBinding,
    #[error("manifest and checkpoint landscape bindings disagree")]
    CrossArtifactBindingMismatch,
    #[error("manifest and checkpoint do not declare the landscape-bound checkpoint schema consistently")]
    CrossArtifactCheckpointSchemaMismatch,
    #[error("landscape-bound checkpoint schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedLandscapeCheckpointSchema { found: u32, supported: u32 },
}
