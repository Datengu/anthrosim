use serde::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig, events::EventLog, manifest::StopReason, metrics::MetricSeries,
    migration::MigrationCheckpointState, population::Population, provenance::ResumeLineage,
    resources::ResourceSystem, rng::RngStreamPosition, temporary_mobility::TemporaryMobilityState,
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
    pub model_semantics_id: String,
    pub git_commit: Option<String>,
    #[serde(default)]
    pub resume_lineage: ResumeLineage,
    pub experiment: ExperimentConfig,
    pub time: SimTime,
    pub completed_years: u64,
    pub terminal_stop_reason: Option<StopReason>,
    pub world_digest64: u64,
    pub population: Population,
    pub temporary_mobility: TemporaryMobilityState,
    pub resources: ResourceSystem,
    pub migration: MigrationCheckpointState,
    pub rng: RngCheckpoint,
    pub events: EventLog,
    pub metrics: MetricSeries,
    pub state_digest64: u64,
}

impl SimulationCheckpoint {
    pub const PRE_LINEAGE_SCHEMA_VERSION: u32 = 4;
    pub const PRE_TEMPORARY_MOBILITY_SCHEMA_VERSION: u32 = 5;
    pub const PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION: u32 = 6;
    pub const PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION: u32 = 7;
    pub const CURRENT_SCHEMA_VERSION: u32 = 8;
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
        digest_u64(&mut hash, value);
    }
    hash
}

/// Extend the legacy authoritative digest with M9 temporary-mobility state whenever the mechanism
/// is enabled or has execution history. The exact disabled state deliberately retains the pre-M9
/// digest so established M1-M8 reference experiments remain directly comparable.
#[must_use]
pub fn state_digest64_with_temporary_mobility(
    day: u64,
    world_digest64: u64,
    population_digest64: u64,
    resource_digest64: u64,
    migration_digest64: u64,
    temporary_mobility: &TemporaryMobilityState,
) -> u64 {
    let legacy = state_digest64(
        day,
        world_digest64,
        population_digest64,
        resource_digest64,
        migration_digest64,
    );
    if temporary_mobility.is_disabled() {
        return legacy;
    }

    let mut hash = FNV_OFFSET_BASIS;
    digest_u64(&mut hash, legacy);
    digest_u64(&mut hash, temporary_mobility.digest64());
    hash
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{PopulationConfig, WorldConfig},
        population::Population,
        rng::RngFactory,
        world::World,
    };

    #[test]
    fn disabled_temporary_mobility_preserves_legacy_state_digest() {
        let factory = RngFactory::new(9);
        let world = World::generate(WorldConfig::new(2, 2), factory).unwrap();
        let population = Population::initialize(PopulationConfig::new(8), &world, factory).unwrap();
        let temporary_mobility = TemporaryMobilityState::at_residence(&population);

        let legacy = state_digest64(365, 11, 22, 33, 44);
        let extended =
            state_digest64_with_temporary_mobility(365, 11, 22, 33, 44, &temporary_mobility);
        assert_eq!(extended, legacy);
    }
}
