use serde::{Deserialize, Serialize};

use crate::{
    config::ExperimentConfig, events::EventLog, manifest::StopReason, metrics::MetricSeries,
    migration::MigrationCheckpointState, population::Population, provenance::ResumeLineage,
    resources::ResourceSystem, rng::RngStreamPosition, temporary_mobility::TemporaryMobilityState,
    time::SimTime,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const CONTINUATION_IDENTITY_DOMAIN: &[u8] = b"anthrosim-continuation-identity-v1\0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RngCheckpoint {
    pub demography_mortality: RngStreamPosition,
    pub demography_fertility: RngStreamPosition,
    pub demography_parentage: RngStreamPosition,
    pub demography_newborn_sex: RngStreamPosition,
    /// Historical Rust identifier retained so the RNG sequence itself remains unchanged. The v10
    /// serialized field names the scientific mechanism rather than the former scarcity label.
    #[serde(rename = "resourceConditionMortality")]
    pub resource_scarcity_mortality: RngStreamPosition,
    pub migration_choice: RngStreamPosition,
    pub migration_uncertainty: RngStreamPosition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
    /// Complete deterministic continuation/output identity. This is deliberately separate from
    /// `stateDigest64`, which remains the compact scientific present-state identity.
    pub continuation_digest64: u64,
    pub state_digest64: u64,
}

impl SimulationCheckpoint {
    pub const PRE_LINEAGE_SCHEMA_VERSION: u32 = 4;
    pub const PRE_TEMPORARY_MOBILITY_SCHEMA_VERSION: u32 = 5;
    pub const PRE_JOURNEY_LIFECYCLE_SCHEMA_VERSION: u32 = 6;
    pub const PRE_TRAVEL_SEMANTICS_SCHEMA_VERSION: u32 = 7;
    pub const PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION: u32 = 8;
    pub const PRE_CONDITION_MORTALITY_SCHEMA_VERSION: u32 = 9;
    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 10;
    pub const CURRENT_SCHEMA_VERSION: u32 = 11;

    /// Seal a newly constructed checkpoint with its complete continuation identity.
    #[must_use]
    pub fn seal_continuation_identity(mut self) -> Self {
        self.continuation_digest64 = continuation_digest64(&self);
        self
    }

    /// Recompute and compare the complete continuation identity without altering the checkpoint.
    #[must_use]
    pub fn continuation_identity_is_valid(&self) -> bool {
        self.continuation_digest64 == continuation_digest64(self)
    }
}

/// Complete deterministic continuation/output material for a checkpoint.
///
/// This deliberately does not replace `stateDigest64`: the legacy state digest remains the compact
/// scientific present-state identity. The continuation digest is a separate integrity/provenance
/// identity that binds every serialized field capable of changing deterministic continuation or
/// exact resumed authoritative output, including RNG positions, configuration, retained histories,
/// complete migration checkpoint state, and resume lineage.
///
/// `continuationDigest64` itself is intentionally excluded to avoid a self-hash cycle.
/// The digest is deterministic but is not a cryptographic authentication mechanism.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContinuationIdentityMaterial<'a> {
    schema_version: u32,
    model_version: &'a str,
    model_semantics_id: &'a str,
    git_commit: &'a Option<String>,
    resume_lineage: &'a ResumeLineage,
    experiment: &'a ExperimentConfig,
    time: SimTime,
    completed_years: u64,
    terminal_stop_reason: &'a Option<StopReason>,
    world_digest64: u64,
    population: &'a Population,
    temporary_mobility: &'a TemporaryMobilityState,
    resources: &'a ResourceSystem,
    migration: &'a MigrationCheckpointState,
    rng: &'a RngCheckpoint,
    events: &'a EventLog,
    metrics: &'a MetricSeries,
    state_digest64: u64,
}

/// Compute the complete deterministic continuation identity for a checkpoint.
///
/// JSON is used only as a deterministic byte encoding of the explicitly ordered struct above; the
/// domain separator prevents accidental equivalence with other FNV-based AnthroSim digests. The
/// checkpoint material uses ordered structs/vectors rather than unordered maps, so the encoding is
/// stable across supported platforms.
#[must_use]
pub fn continuation_digest64(checkpoint: &SimulationCheckpoint) -> u64 {
    let material = ContinuationIdentityMaterial {
        schema_version: checkpoint.schema_version,
        model_version: &checkpoint.model_version,
        model_semantics_id: &checkpoint.model_semantics_id,
        git_commit: &checkpoint.git_commit,
        resume_lineage: &checkpoint.resume_lineage,
        experiment: &checkpoint.experiment,
        time: checkpoint.time,
        completed_years: checkpoint.completed_years,
        terminal_stop_reason: &checkpoint.terminal_stop_reason,
        world_digest64: checkpoint.world_digest64,
        population: &checkpoint.population,
        temporary_mobility: &checkpoint.temporary_mobility,
        resources: &checkpoint.resources,
        migration: &checkpoint.migration,
        rng: &checkpoint.rng,
        events: &checkpoint.events,
        metrics: &checkpoint.metrics,
        state_digest64: checkpoint.state_digest64,
    };
    let encoded = serde_json::to_vec(&material)
        .expect("supported continuation-identity material must serialize deterministically");
    let mut hash = FNV_OFFSET_BASIS;
    digest_bytes(&mut hash, CONTINUATION_IDENTITY_DOMAIN);
    digest_bytes(&mut hash, &encoded);
    hash
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
    digest_bytes(hash, &value.to_le_bytes());
}

fn digest_bytes(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
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

    fn zero_position() -> serde_json::Value {
        serde_json::json!({"low": 0, "high": 0})
    }

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

    #[test]
    fn generated_checkpoint_is_sealed_with_stable_continuation_identity() {
        let checkpoint = crate::Simulation::new(ExperimentConfig::new(70, 2))
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        assert!(checkpoint.continuation_identity_is_valid());
        assert_eq!(
            checkpoint.continuation_digest64,
            continuation_digest64(&checkpoint.clone())
        );
    }

    #[test]
    fn continuation_digest_binds_rng_config_and_complete_migration_state() {
        let checkpoint = crate::Simulation::new(ExperimentConfig::new(71, 2))
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let baseline = checkpoint.continuation_digest64;

        let mut rng_changed = checkpoint.clone();
        rng_changed.rng.migration_choice.low ^= 1;
        assert_ne!(baseline, continuation_digest64(&rng_changed));
        assert!(!rng_changed.continuation_identity_is_valid());

        let mut config_changed = checkpoint.clone();
        config_changed.experiment.duration_years += 1;
        assert_ne!(baseline, continuation_digest64(&config_changed));
        assert!(!config_changed.continuation_identity_is_valid());

        let mut migration_changed = checkpoint.clone();
        migration_changed.migration.northward_steps ^= 1;
        assert_ne!(baseline, continuation_digest64(&migration_changed));
        assert!(!migration_changed.continuation_identity_is_valid());

        let mut explanatory_total_changed = checkpoint.clone();
        explanatory_total_changed
            .migration
            .origin_resource_score_total ^= 1;
        assert_ne!(baseline, continuation_digest64(&explanatory_total_changed));
        assert!(!explanatory_total_changed.continuation_identity_is_valid());
    }

    #[test]
    fn continuation_digest_binds_retained_output_and_lineage_material() {
        let checkpoint = crate::Simulation::new(ExperimentConfig::new(72, 2))
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let baseline = checkpoint.continuation_digest64;

        let mut metrics_changed = checkpoint.clone();
        metrics_changed.metrics.snapshots[0].state_digest64 ^= 1;
        assert_ne!(baseline, continuation_digest64(&metrics_changed));

        let mut events_changed = checkpoint.clone();
        events_changed.events.schema_version ^= 1;
        assert_ne!(baseline, continuation_digest64(&events_changed));

        let mut lineage_changed = checkpoint.clone();
        lineage_changed.resume_lineage.schema_version ^= 1;
        assert_ne!(baseline, continuation_digest64(&lineage_changed));

        let mut terminal_changed = checkpoint.clone();
        terminal_changed.terminal_stop_reason = Some(StopReason::DurationReached);
        assert_ne!(baseline, continuation_digest64(&terminal_changed));
    }

    #[test]
    fn stored_continuation_digest_is_not_self_referential() {
        let checkpoint = crate::Simulation::new(ExperimentConfig::new(73, 1))
            .unwrap()
            .checkpoint_at_year(0)
            .unwrap();
        let baseline = continuation_digest64(&checkpoint);
        let mut stored_digest_changed = checkpoint;
        stored_digest_changed.continuation_digest64 ^= 1;
        assert_eq!(baseline, continuation_digest64(&stored_digest_changed));
        assert!(!stored_digest_changed.continuation_identity_is_valid());
    }

    #[test]
    fn rng_checkpoint_rejects_unknown_fields() {
        let value = serde_json::json!({
            "demographyMortality": zero_position(),
            "demographyFertility": zero_position(),
            "demographyParentage": zero_position(),
            "demographyNewbornSex": zero_position(),
            "resourceConditionMortality": zero_position(),
            "migrationChoice": zero_position(),
            "migrationUncertainty": zero_position(),
            "migrationUncertanty": zero_position()
        });

        let error = serde_json::from_value::<RngCheckpoint>(value).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
        assert!(error.to_string().contains("migrationUncertanty"));
    }

    #[test]
    fn simulation_checkpoint_rejects_unknown_top_level_field_before_normalization() {
        let checkpoint = crate::Simulation::new(ExperimentConfig::new(17, 1))
            .unwrap()
            .checkpoint_at_year(0)
            .unwrap();
        let mut value = serde_json::to_value(checkpoint).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("stateDigset64".to_owned(), serde_json::json!(0));

        let error = serde_json::from_value::<SimulationCheckpoint>(value).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
        assert!(error.to_string().contains("stateDigset64"));
    }
}
