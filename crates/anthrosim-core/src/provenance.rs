use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::time::DAYS_PER_YEAR;

/// Compatibility identity for the authoritative scientific/model semantics used by this build.
///
/// This is intentionally independent of the package version and exact Git commit. Increment the
/// identifier whenever authoritative simulation meaning changes in a way that makes checkpoint
/// continuation scientifically incompatible. Documentation, tooling, or other source-neutral
/// changes do not require a new identity.
pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v7";

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

        let mut previous: Option<&ResumeBoundary> = None;
        for (index, boundary) in self.boundaries.iter().enumerate() {
            if !boundary.boundary_day.is_multiple_of(DAYS_PER_YEAR)
                || boundary.boundary_completed_years != boundary.boundary_day / DAYS_PER_YEAR
            {
                return Err(ResumeLineageError::InvalidBoundaryTime {
                    index,
                    day: boundary.boundary_day,
                    completed_years: boundary.boundary_completed_years,
                });
            }
            if boundary.boundary_day > artifact_day {
                return Err(ResumeLineageError::BoundaryBeyondArtifact {
                    index,
                    boundary_day: boundary.boundary_day,
                    artifact_day,
                });
            }
            if boundary.source.model_version != boundary.continuation.model_version {
                return Err(ResumeLineageError::ModelVersionDiscontinuity { index });
            }
            if boundary.source.model_semantics_id != boundary.continuation.model_semantics_id {
                return Err(ResumeLineageError::ModelSemanticsDiscontinuity { index });
            }

            if let Some(previous) = previous {
                if boundary.boundary_day < previous.boundary_day {
                    return Err(ResumeLineageError::BoundaryOrder { index });
                }
                if boundary.source != previous.continuation {
                    return Err(ResumeLineageError::SourceContinuity { index });
                }
            }
            previous = Some(boundary);
        }

        if let Some(last) = self.boundaries.last()
            && &last.continuation != artifact_identity
        {
            return Err(ResumeLineageError::FinalIdentityMismatch);
        }

        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResumeLineageError {
    #[error("resume-lineage schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error(
        "resume-lineage boundary {index} has inconsistent annual boundary day {day} and completed years {completed_years}"
    )]
    InvalidBoundaryTime {
        index: usize,
        day: u64,
        completed_years: u64,
    },
    #[error(
        "resume-lineage boundary {index} at day {boundary_day} lies after artifact day {artifact_day}"
    )]
    BoundaryBeyondArtifact {
        index: usize,
        boundary_day: u64,
        artifact_day: u64,
    },
    #[error("resume-lineage boundary {index} changes package model version across one resume")]
    ModelVersionDiscontinuity { index },
    #[error("resume-lineage boundary {index} changes model-semantics identity across one resume")]
    ModelSemanticsDiscontinuity { index },
    #[error("resume-lineage boundary {index} precedes the previous boundary")]
    BoundaryOrder { index },
    #[error(
        "resume-lineage boundary {index} source identity does not match the previous continuation"
    )]
    SourceContinuity { index },
    #[error("resume-lineage final continuation identity does not match the containing artifact")]
    FinalIdentityMismatch,
}
