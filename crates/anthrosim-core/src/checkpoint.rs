use serde::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig,
    events::EventLog,
    metrics::MetricSeries,
    migration::MigrationCheckpointState,
    population::Population,
    resources::ResourceSystem,
    rng::RngStreamPosition,
    time::SimTime,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RngCheckpoint {
    pub demography_mortality: RngStreamPosition,
    pub demography_fertility: RngStreamPosition,
    pub demography_parentage: RngStreamPosition,
    pub demography_newborn_sex: RngStreamPosition,
    pub resource_scarcity_mortality: RngStreamPosition,
    pub migration_choice: RngStreamPosition,
    pub migration_uncertainty: RngStreamPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationCheckpoint {
    pub schema_version: u32,
    pub model_version: String,
    pub git_commit: Option<String>,
    pub experiment: ExperimentConfig,
    pub time: SimTime,
    pub completed_years: u64,
    pub world_digest64: u64,
    pub population: Population,
    pub resources: ResourceSystem,
    pub migration: MigrationCheckpointState,
    pub rng: RngCheckpoint,
    pub events: EventLog,
    pub metrics: MetricSeries,
    pub state_digest64: u64,
}

impl SimulationCheckpoint {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

#[must_use]
pub fn state_digest64(
    day: u64,
    world_digest64: u64,
    population_digest64: u64,
    resource_digest64: u64,
    migration_digest64: u64,
) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for value in [
        day,
        world_digest64,
        population_digest64,
        resource_digest64,
        migration_digest64,
    ] {
        for byte in value.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}
