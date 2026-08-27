use serde::{Deserialize, Serialize};

use crate::{
    checkpoint::SimulationCheckpoint,
    config::ExperimentConfig,
    events::EventLog,
    metrics::MetricSeries,
    migration::{MigrationSummary, MigrationSystem},
    population::{Population, PopulationSummary},
    provenance::ResumeLineage,
    research_readiness::EvidenceClosureAssessment,
    resources::{ResourceSummary, ResourceSystem},
    time::SimTime,
    world::{World, WorldSummary},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    DurationReached,
    PopulationExtinct,
    PersonRecordLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSchemas {
    pub manifest: u32,
    pub events: u32,
    pub metrics: u32,
    pub checkpoint: u32,
    pub world: u32,
    pub population: u32,
    pub resources: u32,
    pub migration: u32,
}

impl ArtifactSchemas {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            manifest: RunManifest::CURRENT_SCHEMA_VERSION,
            events: EventLog::CURRENT_SCHEMA_VERSION,
            metrics: MetricSeries::CURRENT_SCHEMA_VERSION,
            checkpoint: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,
            world: World::CURRENT_SCHEMA_VERSION,
            population: Population::CURRENT_SCHEMA_VERSION,
            resources: ResourceSystem::CURRENT_SCHEMA_VERSION,
            migration: MigrationSystem::CURRENT_SCHEMA_VERSION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStatistics {
    pub simulated_days: u64,
    pub authoritative_event_count: u64,
    pub metric_snapshot_count: u64,
    pub resource_periods_processed: u64,
    pub migration_decision_boundaries: u64,
}

/// Immutable summary identifying how a run was produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunManifest {
    pub schema_version: u32,
    pub model_version: String,
    pub model_semantics_id: String,
    pub git_commit: Option<String>,
    #[serde(default)]
    pub resume_lineage: ResumeLineage,
    pub experiment: ExperimentConfig,
    /// Preserved research-readiness assessment for the exact immutable experiment.
    ///
    /// This is separate from ordinary execution validity: a successful run may be synthetic or
    /// explicitly not evidence-closed. Recorded-run invariant validation recomputes this value from
    /// `experiment` so downstream tooling cannot safely relabel readiness without detection.
    pub evidence_closure: EvidenceClosureAssessment,
    pub artifact_schemas: ArtifactSchemas,
    pub world: WorldSummary,
    pub population: PopulationSummary,
    pub resources: ResourceSummary,
    pub migration: MigrationSummary,
    pub state_digest64: u64,
    pub statistics: RunStatistics,
    pub start_time: SimTime,
    pub end_time: SimTime,
    pub stop_reason: StopReason,
}

impl RunManifest {
    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 11;
    pub const PRE_UNDEFINED_EMPTY_MEANS_SCHEMA_VERSION: u32 = 12;
    pub const PRE_EVIDENCE_CLOSURE_SCHEMA_VERSION: u32 = 13;
    pub const PRE_COMPOSED_EVIDENCE_CLOSURE_SCHEMA_VERSION: u32 = 14;
    pub const CURRENT_SCHEMA_VERSION: u32 = 15;
}
