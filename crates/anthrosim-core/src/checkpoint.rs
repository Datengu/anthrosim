use serde::{Deserialize, Serialize};

use crate::{
    config::{ExperimentConfig, ParameterProvenance},
    events::EventLog,
    founder_initialization::{FounderGenealogyStatus, FounderPopulationDefinition},
    manifest::StopReason,
    metrics::MetricSeries,
    migration::MigrationCheckpointState,
    population::{Population, ReproductiveSex},
    provenance::ResumeLineage,
    resources::ResourceSystem,
    rng::RngStreamPosition,
    temporary_mobility::TemporaryMobilityState,
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
    pub const PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION: u32 = 8;
    pub const CURRENT_SCHEMA_VERSION: u32 = 9;
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

/// Extend authoritative state identity with declared founder history only when that history exists.
///
/// `FounderPopulationDefinition` contains pre-run reproductive timing that remains causally active
/// after day 0 until a model-period birth supersedes it. It therefore has to participate in the
/// checkpoint state digest even though it is preserved in immutable experiment configuration rather
/// than forged into model-period Population event state. `None` returns the exact legacy/M9 digest,
/// preserving every existing synthetic reference digest.
#[must_use]
pub(crate) fn state_digest64_with_founder_population(
    day: u64,
    world_digest64: u64,
    population_digest64: u64,
    resource_digest64: u64,
    migration_digest64: u64,
    temporary_mobility: &TemporaryMobilityState,
    founder_population: Option<&FounderPopulationDefinition>,
) -> u64 {
    let legacy = state_digest64_with_temporary_mobility(
        day,
        world_digest64,
        population_digest64,
        resource_digest64,
        migration_digest64,
        temporary_mobility,
    );
    let Some(founder_population) = founder_population else {
        return legacy;
    };

    let mut hash = FNV_OFFSET_BASIS;
    digest_u64(&mut hash, legacy);
    digest_u64(&mut hash, founder_population_digest64(founder_population));
    hash
}

fn founder_population_digest64(definition: &FounderPopulationDefinition) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    digest_u64(&mut hash, u64::from(definition.schema_version));
    digest_bytes(&mut hash, definition.initialization_id.as_bytes());
    digest_u64(
        &mut hash,
        match definition.provenance {
            ParameterProvenance::EmpiricalDirect => 0,
            ParameterProvenance::EmpiricalDerived => 1,
            ParameterProvenance::EvidenceInformed => 2,
            ParameterProvenance::SyntheticValidation => 3,
            ParameterProvenance::Unresolved => 4,
        },
    );
    digest_u64(
        &mut hash,
        match definition.genealogy_status {
            FounderGenealogyStatus::Unspecified => 0,
            FounderGenealogyStatus::CompleteLivingDirectParents => 1,
        },
    );
    digest_u64(
        &mut hash,
        u64::try_from(definition.households.len()).unwrap_or(u64::MAX),
    );
    for household in &definition.households {
        digest_u64(&mut hash, household.id.0);
        digest_u64(&mut hash, household.location.0);
    }
    digest_u64(
        &mut hash,
        u64::try_from(definition.people.len()).unwrap_or(u64::MAX),
    );
    for person in &definition.people {
        digest_u64(&mut hash, person.id.0);
        digest_i64(&mut hash, person.birth_day);
        digest_u64(
            &mut hash,
            match person.reproductive_sex {
                ReproductiveSex::Female => 0,
                ReproductiveSex::Male => 1,
            },
        );
        digest_u64(&mut hash, person.household.0);
        digest_optional_id(&mut hash, person.female_parent.map(|id| id.0));
        digest_optional_id(&mut hash, person.male_parent.map(|id| id.0));
        digest_optional_i64(&mut hash, person.last_birth_day);
        digest_u64(&mut hash, u64::from(person.condition_permille));
    }
    hash
}

fn digest_optional_id(hash: &mut u64, value: Option<u64>) {
    match value {
        Some(value) => {
            digest_u64(hash, 1);
            digest_u64(hash, value);
        }
        None => digest_u64(hash, 0),
    }
}

fn digest_optional_i64(hash: &mut u64, value: Option<i64>) {
    match value {
        Some(value) => {
            digest_u64(hash, 1);
            digest_i64(hash, value);
        }
        None => digest_u64(hash, 0),
    }
}

fn digest_u64(hash: &mut u64, value: u64) {
    digest_bytes(hash, &value.to_le_bytes());
}

fn digest_i64(hash: &mut u64, value: i64) {
    digest_bytes(hash, &value.to_le_bytes());
}

fn digest_bytes(hash: &mut u64, bytes: &[u8]) {
    digest_u64_length(hash, bytes.len());
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn digest_u64_length(hash: &mut u64, length: usize) {
    let length = u64::try_from(length).unwrap_or(u64::MAX);
    for byte in length.to_le_bytes() {
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
        assert_eq!(
            state_digest64_with_founder_population(
                365,
                11,
                22,
                33,
                44,
                &temporary_mobility,
                None,
            ),
            legacy
        );
    }
}
