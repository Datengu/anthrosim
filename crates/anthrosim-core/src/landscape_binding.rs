use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    InvariantError, InvariantReport, LandscapeBundle, LandscapeError, Population, RecordedRun,
    RunManifest, Simulation, SimulationCheckpoint, SimulationError, World,
    validate_recorded_run_invariants,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeRunManifest {
    pub schema_version: u32,
    pub landscape: LandscapeBinding,
    pub core_manifest: RunManifest,
}

impl LandscapeRunManifest {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandscapeCheckpoint {
    pub schema_version: u32,
    pub landscape: LandscapeBinding,
    pub core_checkpoint: SimulationCheckpoint,
}

impl LandscapeCheckpoint {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandscapeRecordedRun {
    pub manifest: LandscapeRunManifest,
    pub checkpoint: LandscapeCheckpoint,
}

impl LandscapeRecordedRun {
    #[must_use]
    pub const fn core_manifest(&self) -> &RunManifest {
        &self.manifest.core_manifest
    }

    #[must_use]
    pub const fn core_checkpoint(&self) -> &SimulationCheckpoint {
        &self.checkpoint.core_checkpoint
    }

    #[must_use]
    pub const fn events(&self) -> &crate::EventLog {
        &self.checkpoint.core_checkpoint.events
    }

    #[must_use]
    pub const fn metrics(&self) -> &crate::MetricSeries {
        &self.checkpoint.core_checkpoint.metrics
    }
}

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
        landscape.validate_evidence_context(config.evidence.as_ref())?;
        Ok(Self {
            simulation: Simulation::new(config)?,
            landscape,
            binding,
        })
    }

    pub fn from_checkpoint(
        checkpoint: LandscapeCheckpoint,
        landscape: LandscapeBundle,
    ) -> Result<Self, LandscapeBindingError> {
        if checkpoint.schema_version != LandscapeCheckpoint::CURRENT_SCHEMA_VERSION {
            return Err(
                LandscapeBindingError::UnsupportedLandscapeCheckpointSchema {
                    found: checkpoint.schema_version,
                    supported: LandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
                },
            );
        }
        checkpoint.landscape.validate_bundle(&landscape)?;
        validate_grid_match(
            &checkpoint.core_checkpoint.experiment,
            &checkpoint.landscape,
        )?;
        landscape
            .validate_evidence_context(checkpoint.core_checkpoint.experiment.evidence.as_ref())?;
        Ok(Self {
            simulation: Simulation::from_checkpoint(checkpoint.core_checkpoint)?,
            landscape,
            binding: checkpoint.landscape,
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

    pub fn checkpoint_at_year(
        self,
        target_year: u64,
    ) -> Result<LandscapeCheckpoint, LandscapeBindingError> {
        Ok(LandscapeCheckpoint {
            schema_version: LandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
            landscape: self.binding,
            core_checkpoint: self.simulation.checkpoint_at_year(target_year)?,
        })
    }

    pub fn run_recorded(self) -> Result<LandscapeRecordedRun, LandscapeBindingError> {
        let binding = self.binding;
        let recorded = self.simulation.run_recorded()?;
        let run = LandscapeRecordedRun {
            manifest: LandscapeRunManifest {
                schema_version: LandscapeRunManifest::CURRENT_SCHEMA_VERSION,
                landscape: binding.clone(),
                core_manifest: recorded.manifest.clone(),
            },
            checkpoint: LandscapeCheckpoint {
                schema_version: LandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
                landscape: binding,
                core_checkpoint: recorded.checkpoint,
            },
        };
        validate_landscape_recorded_run_invariants(&run)?;
        Ok(run)
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

pub fn validate_landscape_recorded_run_invariants(
    run: &LandscapeRecordedRun,
) -> Result<InvariantReport, LandscapeBindingError> {
    if run.manifest.schema_version != LandscapeRunManifest::CURRENT_SCHEMA_VERSION {
        return Err(LandscapeBindingError::UnsupportedLandscapeManifestSchema {
            found: run.manifest.schema_version,
            supported: LandscapeRunManifest::CURRENT_SCHEMA_VERSION,
        });
    }
    if run.checkpoint.schema_version != LandscapeCheckpoint::CURRENT_SCHEMA_VERSION {
        return Err(
            LandscapeBindingError::UnsupportedLandscapeCheckpointSchema {
                found: run.checkpoint.schema_version,
                supported: LandscapeCheckpoint::CURRENT_SCHEMA_VERSION,
            },
        );
    }
    if run.manifest.landscape != run.checkpoint.landscape {
        return Err(LandscapeBindingError::CrossArtifactBindingMismatch);
    }
    if run.manifest.core_manifest.experiment != run.checkpoint.core_checkpoint.experiment
        || run.manifest.core_manifest.state_digest64
            != run.checkpoint.core_checkpoint.state_digest64
    {
        return Err(LandscapeBindingError::CrossArtifactCoreMismatch);
    }
    let core = RecordedRun {
        manifest: run.manifest.core_manifest.clone(),
        checkpoint: run.checkpoint.core_checkpoint.clone(),
    };
    Ok(validate_recorded_run_invariants(&core)?)
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
    #[error(
        "landscape binding mismatch: checkpoint expects {expected}, supplied bundle is {actual}"
    )]
    BindingMismatch { expected: String, actual: String },
    #[error("landscape manifest and checkpoint bindings disagree")]
    CrossArtifactBindingMismatch,
    #[error("landscape wrapper core manifest and checkpoint do not describe the same run")]
    CrossArtifactCoreMismatch,
    #[error(
        "landscape-bound checkpoint schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedLandscapeCheckpointSchema { found: u32, supported: u32 },
    #[error(
        "landscape-bound manifest schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedLandscapeManifestSchema { found: u32, supported: u32 },
}
