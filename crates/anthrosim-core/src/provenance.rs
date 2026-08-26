use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::time::DAYS_PER_YEAR;

/// Compatibility identity for the authoritative scientific/model semantics used by this build.
///
/// This is intentionally independent of the package version and exact Git commit. Increment the
/// identifier whenever authoritative simulation meaning changes in a way that makes checkpoint
/// continuation scientifically incompatible. Documentation, tooling, or other source-neutral
/// changes do not require a new identity.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v8";

/// Exact software/source identity for one segment of authoritative execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRevisionIdentity {
    pub model_version: String,
    pub model_semantics_id: String,
    pub git_commit: Option<String>,
}

impl SourceRevisionIdentity {
    #[must_use]
    pub fn current() -> Self {
        Self {
            model_version: env!("CARGO_PKG_VERSION").to_owned(),
            model_semantics_id: MODEL_SEMANTICS_ID.to_owned(),
            git_commit: option_env!("ANTHROSIM_GIT_COMMIT").map(str::to_owned),
        }
    }
}

/// Append-only source lineage for checkpoint-resumed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeBoundary {
    pub source: SourceRevisionIdentity,
    pub continuation: SourceRevisionIdentity,
    pub boundary_day: u64,
    pub boundary_completed_years: u64,
    pub source_state_digest64: u64,
}

/// Append-only source lineage for checkpoint-resumed execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeLineage {
    pub schema_version: u32,
    pub boundaries: Vec<ResumeBoundary>,
}

impl Default for ResumeLineage {
    fn default() -> Self {
        Self::new()
    }
}

impl ResumeLineage {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub const fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            boundaries: Vec::new(),
        }
    }

    pub fn validate_for_artifact(
        &self,
        artifact_day: u64,
        artifact_identity: &SourceRevisionIdentity,
    ) -> Result<(), ResumeLineageError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(ResumeLineageError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if artifact_identity.model_semantics_id != MODEL_SEMANTICS_ID {
            return Err(ResumeLineageError::UnsupportedModelSemantics {
                found: artifact_identity.model_semantics_id.clone(),
                supported: MODEL_SEMANTICS_ID.to_owned(),
            });
        }
        let mut previous_boundary_day = None;
        let mut previous_continuation: Option<&SourceRevisionIdentity> = None;
        for boundary in &self.boundaries {
            if boundary.source.model_semantics_id != MODEL_SEMANTICS_ID
                || boundary.continuation.model_semantics_id != MODEL_SEMANTICS_ID
            {
                return Err(ResumeLineageError::UnsupportedModelSemantics {
                    found: if boundary.source.model_semantics_id != MODEL_SEMANTICS_ID {
                        boundary.source.model_semantics_id.clone()
                    } else {
                        boundary.continuation.model_semantics_id.clone()
                    },
                    supported: MODEL_SEMANTICS_ID.to_owned(),
                });
            }
            if boundary.boundary_day > artifact_day {
                return Err(ResumeLineageError::BoundaryAfterArtifact {
                    boundary_day: boundary.boundary_day,
                    artifact_day,
                });
            }
            if !boundary.boundary_day.is_multiple_of(DAYS_PER_YEAR)
                || boundary.boundary_completed_years != boundary.boundary_day / DAYS_PER_YEAR
            {
                return Err(ResumeLineageError::InvalidBoundaryTime {
                    boundary_day: boundary.boundary_day,
                    boundary_completed_years: boundary.boundary_completed_years,
                });
            }
            if previous_boundary_day.is_some_and(|previous| boundary.boundary_day < previous) {
                return Err(ResumeLineageError::OutOfOrderBoundary {
                    previous_day: previous_boundary_day.unwrap_or(0),
                    boundary_day: boundary.boundary_day,
                });
            }
            if let Some(previous) = previous_continuation
                && previous != &boundary.source
            {
                return Err(ResumeLineageError::BrokenSourceChain);
            }
            previous_boundary_day = Some(boundary.boundary_day);
            previous_continuation = Some(&boundary.continuation);
        }
        if let Some(last) = self.boundaries.last()
            && &last.continuation != artifact_identity
        {
            return Err(ResumeLineageError::ArtifactIdentityMismatch);
        }
        Ok(())
    }

    pub fn append(
        &mut self,
        boundary: ResumeBoundary,
        artifact_day: u64,
    ) -> Result<(), ResumeLineageError> {
        if boundary.boundary_day > artifact_day {
            return Err(ResumeLineageError::BoundaryAfterArtifact {
                boundary_day: boundary.boundary_day,
                artifact_day,
            });
        }
        if !boundary.boundary_day.is_multiple_of(DAYS_PER_YEAR)
            || boundary.boundary_completed_years != boundary.boundary_day / DAYS_PER_YEAR
        {
            return Err(ResumeLineageError::InvalidBoundaryTime {
                boundary_day: boundary.boundary_day,
                boundary_completed_years: boundary.boundary_completed_years,
            });
        }
        if boundary.source.model_semantics_id != MODEL_SEMANTICS_ID
            || boundary.continuation.model_semantics_id != MODEL_SEMANTICS_ID
        {
            return Err(ResumeLineageError::UnsupportedModelSemantics {
                found: if boundary.source.model_semantics_id != MODEL_SEMANTICS_ID {
                    boundary.source.model_semantics_id.clone()
                } else {
                    boundary.continuation.model_semantics_id.clone()
                },
                supported: MODEL_SEMANTICS_ID.to_owned(),
            });
        }
        if let Some(last) = self.boundaries.last() {
            if boundary.boundary_day < last.boundary_day {
                return Err(ResumeLineageError::OutOfOrderBoundary {
                    previous_day: last.boundary_day,
                    boundary_day: boundary.boundary_day,
                });
            }
            if last.continuation != boundary.source {
                return Err(ResumeLineageError::BrokenSourceChain);
            }
        }
        self.boundaries.push(boundary);
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResumeLineageError {
    #[error("resume lineage schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("model semantics {found} are incompatible; this build requires {supported}")]
    UnsupportedModelSemantics { found: String, supported: String },
    #[error("resume boundary day {boundary_day} lies after artifact day {artifact_day}")]
    BoundaryAfterArtifact {
        boundary_day: u64,
        artifact_day: u64,
    },
    #[error(
        "resume boundary day {boundary_day} does not match completed years {boundary_completed_years}"
    )]
    InvalidBoundaryTime {
        boundary_day: u64,
        boundary_completed_years: u64,
    },
    #[error("resume boundary day {boundary_day} precedes previous boundary day {previous_day}")]
    OutOfOrderBoundary {
        previous_day: u64,
        boundary_day: u64,
    },
    #[error("resume lineage source/continuation chain is broken")]
    BrokenSourceChain,
    #[error("artifact source identity does not match the last resume continuation")]
    ArtifactIdentityMismatch,
}
