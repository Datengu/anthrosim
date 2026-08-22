use serde::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig, population::PopulationSummary, time::SimTime, world::WorldSummary,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StopReason {
    DurationReached,
    PopulationExtinct,
    PersonRecordLimitReached,
}

/// Immutable summary identifying how a run was produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunManifest {
    pub schema_version: u32,
    pub model_version: String,
    pub git_commit: Option<String>,
    pub experiment: ExperimentConfig,
    pub world: WorldSummary,
    pub population: PopulationSummary,
    pub start_time: SimTime,
    pub end_time: SimTime,
    pub stop_reason: StopReason,
}

impl RunManifest {
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;
}
