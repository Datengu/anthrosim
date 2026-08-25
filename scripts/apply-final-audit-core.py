from pathlib import Path


def replace(path: str, old: str, new: str, count: int = 1) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    text = text.replace(old, new, count)
    p.write_text(text, encoding="utf-8")


# ---------------------------------------------------------------------------
# #168: preserve the legacy present-state digest, but add a complete,
# versioned continuation/output identity and bind resume lineage to it.
# ---------------------------------------------------------------------------
checkpoint = "crates/anthrosim-core/src/checkpoint.rs"
replace(
    checkpoint,
    "    pub metrics: MetricSeries,\n    pub state_digest64: u64,\n}",
    "    pub metrics: MetricSeries,\n    pub state_digest64: u64,\n    /// Complete deterministic continuation/output identity. This is deliberately\n    /// separate from `stateDigest64`, whose historical present-state semantics are\n    /// retained for M1-M9 scientific-reference compatibility.\n    #[serde(default)]\n    pub continuation_digest64: u64,\n}",
)
replace(
    checkpoint,
    "    pub const PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION: u32 = 8;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 9;\n}",
    "    pub const PRE_DURATION_AWARE_RESOURCE_SCHEMA_VERSION: u32 = 8;\n    /// v0.3.0 checkpoint schema: predates complete continuation integrity.\n    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 9;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 10;\n\n    /// Recompute the complete continuation/output identity after constructing or\n    /// deliberately transforming a checkpoint. Ordinary callers should never need\n    /// to mutate authoritative checkpoint state.\n    pub fn refresh_continuation_digest64(&mut self) {\n        self.continuation_digest64 = compute_continuation_digest64(self);\n    }\n}\n\n#[derive(Serialize)]\n#[serde(rename_all = \"camelCase\")]\nstruct ContinuationIdentity<'a> {\n    schema_version: u32,\n    model_version: &'a str,\n    model_semantics_id: &'a str,\n    git_commit: &'a Option<String>,\n    resume_lineage: &'a ResumeLineage,\n    experiment: &'a ExperimentConfig,\n    time: &'a SimTime,\n    completed_years: u64,\n    terminal_stop_reason: &'a Option<StopReason>,\n    world_digest64: u64,\n    population: &'a Population,\n    temporary_mobility: &'a TemporaryMobilityState,\n    resources: &'a ResourceSystem,\n    migration: &'a MigrationCheckpointState,\n    rng: &'a RngCheckpoint,\n    events: &'a EventLog,\n    metrics: &'a MetricSeries,\n    state_digest64: u64,\n}\n\n/// Stable complete identity for deterministic continuation and exact resumed\n/// authoritative output. The legacy `stateDigest64` remains unchanged.\n///\n/// Serialization is safe as the digest projection contains only versioned\n/// integer/string/sequence structures with deterministic field order and no map\n/// with implementation-dependent iteration order.\n#[must_use]\npub fn compute_continuation_digest64(checkpoint: &SimulationCheckpoint) -> u64 {\n    let identity = ContinuationIdentity {\n        schema_version: checkpoint.schema_version,\n        model_version: &checkpoint.model_version,\n        model_semantics_id: &checkpoint.model_semantics_id,\n        git_commit: &checkpoint.git_commit,\n        resume_lineage: &checkpoint.resume_lineage,\n        experiment: &checkpoint.experiment,\n        time: &checkpoint.time,\n        completed_years: checkpoint.completed_years,\n        terminal_stop_reason: &checkpoint.terminal_stop_reason,\n        world_digest64: checkpoint.world_digest64,\n        population: &checkpoint.population,\n        temporary_mobility: &checkpoint.temporary_mobility,\n        resources: &checkpoint.resources,\n        migration: &checkpoint.migration,\n        rng: &checkpoint.rng,\n        events: &checkpoint.events,\n        metrics: &checkpoint.metrics,\n        state_digest64: checkpoint.state_digest64,\n    };\n    let encoded = serde_json::to_vec(&identity)\n        .expect(\"continuation identity contains only serializable checkpoint fields\");\n    let mut hash = FNV_OFFSET_BASIS;\n    for &byte in b\"anthrosim-continuation-v1\\0\".iter().chain(encoded.iter()) {\n        hash ^= u64::from(byte);\n        hash = hash.wrapping_mul(FNV_PRIME);\n    }\n    hash\n}",
)

provenance = "crates/anthrosim-core/src/provenance.rs"
replace(
    provenance,
    "    pub boundary_completed_years: u64,\n    pub source_state_digest64: u64,\n}",
    "    pub boundary_completed_years: u64,\n    /// Historical present-state digest retained for comparison/reporting.\n    pub source_state_digest64: u64,\n    /// Complete continuation identity accepted at this resume boundary.\n    #[serde(default)]\n    pub source_continuation_digest64: u64,\n}",
)
replace(
    provenance,
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 1;\n\n    #[must_use]\n    pub const fn new() -> Self {\n        Self {\n            schema_version: Self::CURRENT_SCHEMA_VERSION,\n            boundaries: Vec::new(),\n        }\n    }\n\n    pub fn validate_for_artifact(",
    "    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 1;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 2;\n\n    #[must_use]\n    pub const fn new() -> Self {\n        // Keep the empty lineage byte-for-byte compatible with established M8/M9\n        // non-resumed reference artifacts. The first new resume upgrades it to v2.\n        Self {\n            schema_version: Self::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION,\n            boundaries: Vec::new(),\n        }\n    }\n\n    pub fn push_boundary(&mut self, boundary: ResumeBoundary) {\n        self.schema_version = Self::CURRENT_SCHEMA_VERSION;\n        self.boundaries.push(boundary);\n    }\n\n    pub fn validate_for_artifact(",
)
replace(
    provenance,
    "        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {\n            return Err(ResumeLineageError::UnsupportedSchema {\n                found: self.schema_version,\n                supported: Self::CURRENT_SCHEMA_VERSION,\n            });\n        }\n\n        let mut previous: Option<&ResumeBoundary> = None;",
    "        let legacy_empty = self.schema_version == Self::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION\n            && self.boundaries.is_empty();\n        if self.schema_version != Self::CURRENT_SCHEMA_VERSION && !legacy_empty {\n            return Err(ResumeLineageError::UnsupportedSchema {\n                found: self.schema_version,\n                supported: Self::CURRENT_SCHEMA_VERSION,\n            });\n        }\n\n        let mut previous: Option<&ResumeBoundary> = None;",
)
replace(
    provenance,
    "            if boundary.boundary_day > artifact_day {\n                return Err(ResumeLineageError::BoundaryBeyondArtifact {",
    "            if boundary.source_continuation_digest64 == 0 {\n                return Err(ResumeLineageError::MissingContinuationIdentity { index });\n            }\n            if boundary.boundary_day > artifact_day {\n                return Err(ResumeLineageError::BoundaryBeyondArtifact {",
)
replace(
    provenance,
    "    #[error(\"resume-lineage boundary {index} has inconsistent annual boundary day {day} and completed years {completed_years}\")]",
    "    #[error(\"resume-lineage boundary {index} is missing its complete source continuation identity\")]\n    MissingContinuationIdentity { index: usize },\n    #[error(\"resume-lineage boundary {index} has inconsistent annual boundary day {day} and completed years {completed_years}\")]",
)

simulation = "crates/anthrosim-core/src/simulation.rs"
replace(
    simulation,
    "    checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64_with_temporary_mobility},",
    "    checkpoint::{\n        RngCheckpoint, SimulationCheckpoint, compute_continuation_digest64,\n        state_digest64_with_temporary_mobility,\n    },",
)
replace(
    simulation,
    "        if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {\n            return Err(SimulationError::UnsupportedCheckpointSchema {\n                found: checkpoint.schema_version,\n                supported: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,\n            });\n        }",
    "        if checkpoint.schema_version == SimulationCheckpoint::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION {\n            return Err(SimulationError::CheckpointMissingContinuationIdentity {\n                found: checkpoint.schema_version,\n            });\n        }\n        if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {\n            return Err(SimulationError::UnsupportedCheckpointSchema {\n                found: checkpoint.schema_version,\n                supported: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,\n            });\n        }\n        let actual_continuation_digest64 = compute_continuation_digest64(&checkpoint);\n        if checkpoint.continuation_digest64 != actual_continuation_digest64 {\n            return Err(SimulationError::CheckpointContinuationDigestMismatch {\n                expected: checkpoint.continuation_digest64,\n                actual: actual_continuation_digest64,\n            });\n        }",
)
replace(
    simulation,
    "        validate_terminal_checkpoint_state(&checkpoint)?;\n\n        let boundary_day = checkpoint.time.days();",
    "        validate_terminal_checkpoint_state(&checkpoint)?;\n        checkpoint\n            .validate_invariants()\n            .map_err(|error| SimulationError::CheckpointInvariantViolation {\n                reason: error.to_string(),\n            })?;\n\n        let boundary_day = checkpoint.time.days();",
)
replace(
    simulation,
    "        let source_state_digest64 = checkpoint.state_digest64;\n        let continuation_identity = SourceRevisionIdentity::current();\n        let mut resume_lineage = checkpoint.resume_lineage;\n        resume_lineage.boundaries.push(ResumeBoundary {\n            source: source_identity,\n            continuation: continuation_identity,\n            boundary_day,\n            boundary_completed_years,\n            source_state_digest64,\n        });",
    "        let source_state_digest64 = checkpoint.state_digest64;\n        let source_continuation_digest64 = checkpoint.continuation_digest64;\n        let continuation_identity = SourceRevisionIdentity::current();\n        let mut resume_lineage = checkpoint.resume_lineage;\n        resume_lineage.push_boundary(ResumeBoundary {\n            source: source_identity,\n            continuation: continuation_identity,\n            boundary_day,\n            boundary_completed_years,\n            source_state_digest64,\n            source_continuation_digest64,\n        });",
)
replace(
    simulation,
    "        SimulationCheckpoint {\n            schema_version: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,",
    "        let mut checkpoint = SimulationCheckpoint {\n            schema_version: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,",
)
replace(
    simulation,
    "            metrics: self.metrics,\n            state_digest64: state_digest,\n        }\n    }",
    "            metrics: self.metrics,\n            state_digest64: state_digest,\n            continuation_digest64: 0,\n        };\n        checkpoint.refresh_continuation_digest64();\n        checkpoint\n    }",
)
replace(
    simulation,
    "    #[error(\"checkpoint schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedCheckpointSchema { found: u32, supported: u32 },",
    "    #[error(\"checkpoint schema {found} predates complete continuation integrity and cannot be resumed safely\")]\n    CheckpointMissingContinuationIdentity { found: u32 },\n    #[error(\"checkpoint schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedCheckpointSchema { found: u32, supported: u32 },",
)
replace(
    simulation,
    "    #[error(\"checkpoint state digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CheckpointStateDigestMismatch { expected: u64, actual: u64 },",
    "    #[error(\"checkpoint continuation digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CheckpointContinuationDigestMismatch { expected: u64, actual: u64 },\n    #[error(\"checkpoint invariant validation failed before resume: {reason}\")]\n    CheckpointInvariantViolation { reason: String },\n    #[error(\"checkpoint state digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CheckpointStateDigestMismatch { expected: u64, actual: u64 },",
)

spatial = "crates/anthrosim-core/src/spatial_simulation.rs"
replace(
    spatial,
    "    checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64_with_temporary_mobility},",
    "    checkpoint::{\n        RngCheckpoint, SimulationCheckpoint, compute_continuation_digest64,\n        state_digest64_with_temporary_mobility,\n    },",
)
replace(
    spatial,
    "    landscape::LandscapeBundle,",
    "    invariants::validate_checkpoint_invariants_with_world,\n    landscape::LandscapeBundle,",
)
replace(
    spatial,
    "        checkpoint\n            .core_checkpoint\n            .population\n            .validate(&world)\n            .map_err(PopulationError::from)?;",
    "        validate_checkpoint_invariants_with_world(&checkpoint.core_checkpoint, &world).map_err(\n            |error| SpatialLandscapeError::CoreCheckpointInvariantViolation {\n                reason: error.to_string(),\n            },\n        )?;\n        checkpoint\n            .core_checkpoint\n            .population\n            .validate(&world)\n            .map_err(PopulationError::from)?;",
    1,
)
replace(
    spatial,
    "        let expected_state_digest = checkpoint.core_checkpoint.state_digest64;",
    "        let expected_state_digest = checkpoint.core_checkpoint.state_digest64;\n        let source_continuation_digest64 = checkpoint.core_checkpoint.continuation_digest64;",
)
replace(
    spatial,
    "        resume_lineage.boundaries.push(ResumeBoundary {\n            source: source_identity,\n            continuation: SourceRevisionIdentity::current(),\n            boundary_day: checkpoint.core_checkpoint.time.days(),\n            boundary_completed_years: checkpoint.core_checkpoint.completed_years,\n            source_state_digest64: checkpoint.core_checkpoint.state_digest64,\n        });",
    "        resume_lineage.push_boundary(ResumeBoundary {\n            source: source_identity,\n            continuation: SourceRevisionIdentity::current(),\n            boundary_day: checkpoint.core_checkpoint.time.days(),\n            boundary_completed_years: checkpoint.core_checkpoint.completed_years,\n            source_state_digest64: checkpoint.core_checkpoint.state_digest64,\n            source_continuation_digest64,\n        });",
)
replace(
    spatial,
    "        SimulationCheckpoint {\n            schema_version: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,",
    "        let mut checkpoint = SimulationCheckpoint {\n            schema_version: SimulationCheckpoint::CURRENT_SCHEMA_VERSION,",
)
replace(
    spatial,
    "            metrics: self.metrics,\n            state_digest64: state_digest,\n        }\n    }",
    "            metrics: self.metrics,\n            state_digest64: state_digest,\n            continuation_digest64: 0,\n        };\n        checkpoint.refresh_continuation_digest64();\n        checkpoint\n    }",
)
replace(
    spatial,
    "    if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {\n        return Err(SpatialLandscapeError::UnsupportedCoreCheckpointSchema {",
    "    if checkpoint.schema_version == SimulationCheckpoint::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION {\n        return Err(SpatialLandscapeError::CoreCheckpointMissingContinuationIdentity {\n            found: checkpoint.schema_version,\n        });\n    }\n    if checkpoint.schema_version != SimulationCheckpoint::CURRENT_SCHEMA_VERSION {\n        return Err(SpatialLandscapeError::UnsupportedCoreCheckpointSchema {",
)
replace(
    spatial,
    "    if checkpoint.model_version != env!(\"CARGO_PKG_VERSION\") {",
    "    let actual_continuation_digest64 = compute_continuation_digest64(checkpoint);\n    if checkpoint.continuation_digest64 != actual_continuation_digest64 {\n        return Err(SpatialLandscapeError::CoreCheckpointContinuationDigestMismatch {\n            expected: checkpoint.continuation_digest64,\n            actual: actual_continuation_digest64,\n        });\n    }\n    if checkpoint.model_version != env!(\"CARGO_PKG_VERSION\") {",
    1,
)
replace(
    spatial,
    "    #[error(\"core checkpoint schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedCoreCheckpointSchema { found: u32, supported: u32 },",
    "    #[error(\"core checkpoint schema {found} predates complete continuation integrity and cannot be resumed safely\")]\n    CoreCheckpointMissingContinuationIdentity { found: u32 },\n    #[error(\"core checkpoint schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedCoreCheckpointSchema { found: u32, supported: u32 },",
)
replace(
    spatial,
    "    #[error(\"checkpoint state digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CheckpointStateDigestMismatch { expected: u64, actual: u64 },",
    "    #[error(\"core checkpoint continuation digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CoreCheckpointContinuationDigestMismatch { expected: u64, actual: u64 },\n    #[error(\"core checkpoint invariant validation failed before transformed resume: {reason}\")]\n    CoreCheckpointInvariantViolation { reason: String },\n    #[error(\"checkpoint state digest mismatch: expected {expected}, reconstructed {actual}\")]\n    CheckpointStateDigestMismatch { expected: u64, actual: u64 },",
)

# The recorded transformed-run validator must run the same full checkpoint suite.
replace(
    spatial,
    "    run.checkpoint\n        .core_checkpoint\n        .population\n        .validate(&world)\n        .map_err(PopulationError::from)?;",
    "    validate_checkpoint_invariants_with_world(&run.checkpoint.core_checkpoint, &world).map_err(\n        |error| SpatialLandscapeError::CoreCheckpointInvariantViolation {\n            reason: error.to_string(),\n        },\n    )?;\n    run.checkpoint\n        .core_checkpoint\n        .population\n        .validate(&world)\n        .map_err(PopulationError::from)?;",
    1,
)

# Export the complete identity for audit/integration consumers without changing
# the legacy state-digest API.
lib = "crates/anthrosim-core/src/lib.rs"
replace(
    lib,
    "pub use checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64};",
    "pub use checkpoint::{\n    RngCheckpoint, SimulationCheckpoint, compute_continuation_digest64, state_digest64,\n};",
)

# ---------------------------------------------------------------------------
# #176: aggregate resource stock must never wrap/panic; restored cell state must
# also respect its configured capacity.
# ---------------------------------------------------------------------------
resources = "crates/anthrosim-core/src/resources.rs"
replace(
    resources,
    "    #[must_use]\n    pub fn total_food_stock(&self) -> u64 {\n        self.cell_food_stock.iter().copied().sum()\n    }",
    "    /// Observational aggregate for already-valid state. Invalid/restored state is\n    /// checked through `checked_total_food_stock` before authoritative use.\n    #[must_use]\n    pub fn total_food_stock(&self) -> u64 {\n        self.checked_total_food_stock().unwrap_or(u64::MAX)\n    }\n\n    fn checked_total_food_stock(&self) -> Result<u64, ResourceError> {\n        self.cell_food_stock.iter().try_fold(0_u64, |total, &stock| {\n            total\n                .checked_add(stock)\n                .ok_or(ResourceError::AccountingOverflow)\n        })\n    }",
)
replace(resources, "        let stock_before = self.total_food_stock();", "        let stock_before = self.checked_total_food_stock()?;")
replace(resources, "        let stock_after = self.total_food_stock();", "        let stock_after = self.checked_total_food_stock()?;")
replace(
    resources,
    "        if self.schema_version != Self::CURRENT_SCHEMA_VERSION\n            || self.model_id != config.model_id\n            || self.cell_food_stock.len() != world.cell_count()\n            || self.initial_world_digest64 != format!(\"{:016x}\", world.digest64())\n        {\n            return Err(ResourceError::StateShapeMismatch);\n        }\n        self.validate_accounting()",
    "        if self.schema_version != Self::CURRENT_SCHEMA_VERSION\n            || self.model_id != config.model_id\n            || self.cell_food_stock.len() != world.cell_count()\n            || self.initial_world_digest64 != format!(\"{:016x}\", world.digest64())\n        {\n            return Err(ResourceError::StateShapeMismatch);\n        }\n        for (index, (&stock, cell)) in self\n            .cell_food_stock\n            .iter()\n            .zip(world.cells().iter())\n            .enumerate()\n        {\n            let capacity = cell_capacity(cell.base_productivity, config);\n            if stock > capacity {\n                return Err(ResourceError::CellStockExceedsCapacity {\n                    cell_index: u64::try_from(index)\n                        .map_err(|_| ResourceError::AccountingOverflow)?,\n                    stock,\n                    capacity,\n                });\n            }\n        }\n        self.validate_accounting()",
)
replace(resources, "        let actual = self.total_food_stock();", "        let actual = self.checked_total_food_stock()?;", 1)
replace(
    resources,
    "    #[error(\"resource state does not match world cell count\")]\n    StateShapeMismatch,",
    "    #[error(\"resource state does not match world cell count or configured identity\")]\n    StateShapeMismatch,\n    #[error(\"resource cell {cell_index} stock {stock} exceeds configured capacity {capacity}\")]\n    CellStockExceedsCapacity {\n        cell_index: u64,\n        stock: u64,\n        capacity: u64,\n    },",
)

# ---------------------------------------------------------------------------
# #171: private schema/shape fields in deserialized World/Population must be
# validated just as constructor inputs are.
# ---------------------------------------------------------------------------
world = "crates/anthrosim-core/src/world.rs"
replace(
    world,
    "    pub fn validate(&self) -> Result<(), WorldValidationError> {\n        let expected = u64::from(self.width) * u64::from(self.height);",
    "    pub fn validate(&self) -> Result<(), WorldValidationError> {\n        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {\n            return Err(WorldValidationError::UnsupportedSchema {\n                found: self.schema_version,\n                supported: Self::CURRENT_SCHEMA_VERSION,\n            });\n        }\n        if self.width == 0 || self.height == 0 {\n            return Err(WorldValidationError::InvalidDimensions);\n        }\n        let expected = u64::from(self.width) * u64::from(self.height);",
)
replace(
    world,
    "pub enum WorldValidationError {\n    #[error(\"world cell count mismatch: expected {expected}, found {actual}\")]",
    "pub enum WorldValidationError {\n    #[error(\"world schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedSchema { found: u32, supported: u32 },\n    #[error(\"world width and height must both be greater than zero\")]\n    InvalidDimensions,\n    #[error(\"world cell count mismatch: expected {expected}, found {actual}\")]",
)

population = "crates/anthrosim-core/src/population.rs"
replace(
    population,
    "    pub fn validate(&self, world: &World) -> Result<(), PopulationValidationError> {\n        let person_count = self.person_count();",
    "    pub fn validate(&self, world: &World) -> Result<(), PopulationValidationError> {\n        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {\n            return Err(PopulationValidationError::UnsupportedSchema {\n                found: self.schema_version,\n                supported: Self::CURRENT_SCHEMA_VERSION,\n            });\n        }\n        let person_count = self.person_count();",
)
replace(
    population,
    "pub enum PopulationValidationError {\n    #[error(\"population structure-of-arrays columns have different lengths\")]",
    "pub enum PopulationValidationError {\n    #[error(\"population schema {found} is unsupported; supported schema is {supported}\")]\n    UnsupportedSchema { found: u32, supported: u32 },\n    #[error(\"population structure-of-arrays columns have different lengths\")]",
)

# ---------------------------------------------------------------------------
# #178: require entity-level event uniqueness/bijection and independently check
# the accounting content of non-terminal metric snapshots.
# ---------------------------------------------------------------------------
invariants = "crates/anthrosim-core/src/invariants.rs"
replace(invariants, "use thiserror::Error;", "use std::collections::BTreeSet;\n\nuse thiserror::Error;")
replace(
    invariants,
    "    checkpoint::{SimulationCheckpoint, state_digest64_with_temporary_mobility},",
    "    checkpoint::{\n        SimulationCheckpoint, compute_continuation_digest64,\n        state_digest64_with_temporary_mobility,\n    },",
)
replace(
    invariants,
    "    validate_checkpoint_identity(checkpoint, recorded_stop_reason)?;",
    "    validate_checkpoint_identity(checkpoint, recorded_stop_reason)?;\n    let actual_continuation_digest64 = compute_continuation_digest64(checkpoint);\n    if actual_continuation_digest64 != checkpoint.continuation_digest64 {\n        return violation(format!(\n            \"continuation digest mismatch: stored {}, reconstructed {}\",\n            checkpoint.continuation_digest64, actual_continuation_digest64\n        ));\n    }",
)
replace(
    invariants,
    "    validate_metrics(\n        &checkpoint.metrics,\n        checkpoint.time.days(),",
    "    validate_metrics(\n        &checkpoint.metrics,\n        &checkpoint.events,\n        checkpoint.time.days(),",
)
replace(
    invariants,
    "    let mut counts = EventCounts::default();\n    let mut previous_day = None;",
    "    let mut counts = EventCounts::default();\n    let mut previous_day = None;\n    let mut birth_people = BTreeSet::new();\n    let mut death_people = BTreeSet::new();\n    let mut migration_household_days = BTreeSet::new();",
)
replace(
    invariants,
    "            } => {\n                counts.births = counts.births.saturating_add(1);\n                let snapshot = population_state.person(*person).ok_or_else(|| {",
    "            } => {\n                if !birth_people.insert(*person) {\n                    return violation(\"duplicate authoritative birth event for one person\");\n                }\n                counts.births = counts.births.saturating_add(1);\n                let snapshot = population_state.person(*person).ok_or_else(|| {",
    1,
)
replace(
    invariants,
    "            } => {\n                counts.deaths = counts.deaths.saturating_add(1);\n                if matches!(cause, crate::DeathCause::ResourceScarcity) {",
    "            } => {\n                if !death_people.insert(*person) {\n                    return violation(\"duplicate authoritative death event for one person\");\n                }\n                counts.deaths = counts.deaths.saturating_add(1);\n                if matches!(cause, crate::DeathCause::ResourceScarcity) {",
    1,
)
replace(
    invariants,
    "            } => {\n                counts.migrations = counts.migrations.saturating_add(1);\n                counts.people_moved = counts.people_moved.saturating_add(u64::from(*people_moved));",
    "            } => {\n                if !migration_household_days.insert((record.day, *household)) {\n                    return violation(\"household has duplicate completed migration events at one boundary\");\n                }\n                counts.migrations = counts.migrations.saturating_add(1);\n                counts.people_moved = counts.people_moved.saturating_add(u64::from(*people_moved));",
    1,
)

old_metrics = '''fn validate_metrics(
    metrics: &MetricSeries,
    day: u64,
    state_digest: u64,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<(), InvariantError> {
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION
        || metrics.snapshots.is_empty()
    {
        return violation("metric series schema is invalid or terminal snapshot is missing");
    }
    let mut previous_day = None;
    for snapshot in &metrics.snapshots {
        if snapshot.schema_version != MetricSnapshot::CURRENT_SCHEMA_VERSION
            || snapshot.provenance != MetricProvenance::Derived
            || snapshot.day > day
            || previous_day.is_some_and(|prior| snapshot.day <= prior)
        {
            return violation("metric snapshot schema, provenance, or ordering is invalid");
        }
        previous_day = Some(snapshot.day);
    }
    let final_snapshot = metrics.snapshots.last().expect("non-empty checked above");
    if final_snapshot.day != day
        || final_snapshot.state_digest64 != state_digest
        || final_snapshot.population != PopulationMetrics::from(population)
        || final_snapshot.resources != ResourceMetrics::from(resources)
        || final_snapshot.migration != MigrationMetrics::from(migration)
    {
        return violation("terminal derived metrics do not reconcile with authoritative state");
    }
    Ok(())
}
'''
new_metrics = '''fn validate_metrics(
    metrics: &MetricSeries,
    events: &EventLog,
    day: u64,
    state_digest: u64,
    population: &PopulationSummary,
    resources: &ResourceSummary,
    migration: &MigrationSummary,
) -> Result<(), InvariantError> {
    if metrics.schema_version != MetricSeries::CURRENT_SCHEMA_VERSION
        || metrics.cadence != "annual_boundary_plus_terminal"
        || metrics.snapshots.is_empty()
    {
        return violation("metric series schema/cadence is invalid or terminal snapshot is missing");
    }
    let mut previous: Option<&MetricSnapshot> = None;
    for snapshot in &metrics.snapshots {
        if snapshot.schema_version != MetricSnapshot::CURRENT_SCHEMA_VERSION
            || snapshot.provenance != MetricProvenance::Derived
            || snapshot.day > day
            || previous.is_some_and(|prior| snapshot.day <= prior.day)
        {
            return violation("metric snapshot schema, provenance, or ordering is invalid");
        }

        let mut births = 0_u64;
        let mut deaths = 0_u64;
        let mut scarcity_deaths = 0_u64;
        let mut migrations = 0_u64;
        let mut people_moved = 0_u64;
        let mut migration_distance = 0_u64;
        for record in events.events.iter().take_while(|record| record.day <= snapshot.day) {
            match &record.event {
                EventKind::Birth { .. } => births = births.saturating_add(1),
                EventKind::Death { cause, .. } => {
                    deaths = deaths.saturating_add(1);
                    if matches!(cause, crate::DeathCause::ResourceScarcity) {
                        scarcity_deaths = scarcity_deaths.saturating_add(1);
                    }
                }
                EventKind::HouseholdMigration {
                    people_moved: moved,
                    distance_cells,
                    ..
                } => {
                    migrations = migrations.saturating_add(1);
                    people_moved = people_moved.saturating_add(u64::from(*moved));
                    migration_distance =
                        migration_distance.saturating_add(u64::from(*distance_cells));
                }
                EventKind::TemporaryJourneyNotStarted { .. }
                | EventKind::TemporaryJourneyDeparted { .. }
                | EventKind::TemporaryJourneyArrived { .. }
                | EventKind::TemporaryReturnDeparted { .. }
                | EventKind::TemporaryJourneyCompleted { .. } => {}
            }
        }

        let expected_records = u64::from(population.initial_population)
            .checked_add(births)
            .ok_or_else(|| InvariantError::Violation("metric population accounting overflowed".into()))?;
        let expected_living = expected_records
            .checked_sub(deaths)
            .ok_or_else(|| InvariantError::Violation("metric population accounting underflowed".into()))?;
        if snapshot.population.births_since_start != births
            || snapshot.population.deaths_since_start != deaths
            || snapshot.population.person_records != expected_records
            || snapshot.population.living_population != expected_living
            || snapshot.population.living_occupied_cell_count > expected_living
            || snapshot.population.living_below_half_condition > expected_living
            || snapshot.population.mean_living_condition_permille > PERMILLE_MAX
        {
            return violation("intermediate population metrics do not reconcile with event history");
        }

        let expected_stock = resources
            .initial_food_stock
            .checked_add(snapshot.resources.regenerated_food)
            .and_then(|value| value.checked_sub(snapshot.resources.harvested_food))
            .ok_or_else(|| InvariantError::Violation("metric resource accounting overflowed".into()))?;
        if snapshot.resources.final_food_stock != expected_stock
            || snapshot.resources.scarcity_deaths != scarcity_deaths
        {
            return violation("intermediate resource metrics do not reconcile with accounting/history");
        }
        if snapshot.migration.moves_completed != migrations
            || snapshot.migration.people_moved != people_moved
            || snapshot.migration.total_distance_cells != migration_distance
            || snapshot.migration.households_under_pressure > snapshot.migration.households_evaluated
            || snapshot.migration.moves_completed > snapshot.migration.households_under_pressure
        {
            return violation("intermediate migration metrics do not reconcile with event history");
        }

        if let Some(prior) = previous {
            let population_monotonic = snapshot.population.person_records >= prior.population.person_records
                && snapshot.population.births_since_start >= prior.population.births_since_start
                && snapshot.population.deaths_since_start >= prior.population.deaths_since_start;
            let resource_monotonic = snapshot.resources.periods_processed >= prior.resources.periods_processed
                && snapshot.resources.regenerated_food >= prior.resources.regenerated_food
                && snapshot.resources.harvested_food >= prior.resources.harvested_food
                && snapshot.resources.unmet_need >= prior.resources.unmet_need
                && snapshot.resources.household_periods_with_unmet_need
                    >= prior.resources.household_periods_with_unmet_need
                && snapshot.resources.scarcity_deaths >= prior.resources.scarcity_deaths;
            let migration_monotonic = snapshot.migration.decision_boundaries
                >= prior.migration.decision_boundaries
                && snapshot.migration.households_evaluated >= prior.migration.households_evaluated
                && snapshot.migration.households_under_pressure
                    >= prior.migration.households_under_pressure
                && snapshot.migration.moves_completed >= prior.migration.moves_completed
                && snapshot.migration.people_moved >= prior.migration.people_moved
                && snapshot.migration.total_distance_cells >= prior.migration.total_distance_cells;
            if !population_monotonic || !resource_monotonic || !migration_monotonic {
                return violation("intermediate cumulative metrics move backwards");
            }
        }
        previous = Some(snapshot);
    }
    let final_snapshot = metrics.snapshots.last().expect("non-empty checked above");
    if final_snapshot.day != day
        || final_snapshot.state_digest64 != state_digest
        || final_snapshot.population != PopulationMetrics::from(population)
        || final_snapshot.resources != ResourceMetrics::from(resources)
        || final_snapshot.migration != MigrationMetrics::from(migration)
    {
        return violation("terminal derived metrics do not reconcile with authoritative state");
    }
    Ok(())
}
'''
replace(invariants, old_metrics, new_metrics)

# ---------------------------------------------------------------------------
# Focused integration mutations: every RNG stream, future config, previously
# omitted migration state, transformed resume, lineage binding, schema checks,
# duplicate history, and intermediate metrics.
# ---------------------------------------------------------------------------
tests = Path("crates/anthrosim-core/tests/final_audit_integrity.rs")
tests.parent.mkdir(parents=True, exist_ok=True)
tests.write_text(r'''use anthrosim_core::{
    compute_continuation_digest64, ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer,
    LandscapeLayerRole, LandscapeValueDomain, NoDataPolicy, Population, PopulationConfig,
    RngStreamPosition, Simulation, SimulationCheckpoint, SpatialFieldTransform,
    SpatialLandscapeSimulation, SpatialMechanismConfig, SpatialTargetField, TransformDirection,
    World, WorldConfig,
};
use anthrosim_core::rng::RngFactory;

fn small_checkpoint() -> SimulationCheckpoint {
    Simulation::new(
        ExperimentConfig::new(2026, 2)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(100).with_max_person_records(10_000)),
    )
    .unwrap()
    .checkpoint_at_year(1)
    .unwrap()
}

fn bump(position: &mut RngStreamPosition) {
    position.low = position.low.wrapping_add(1);
}

#[test]
fn every_rng_stream_is_bound_before_resume() {
    let original = small_checkpoint();
    for mutate in [
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.demography_mortality),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.demography_fertility),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.demography_parentage),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.demography_newborn_sex),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.resource_scarcity_mortality),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.migration_choice),
        |c: &mut SimulationCheckpoint| bump(&mut c.rng.migration_uncertainty),
    ] {
        let mut mutated = original.clone();
        mutate(&mut mutated);
        assert_eq!(mutated.state_digest64, original.state_digest64);
        assert_ne!(
            compute_continuation_digest64(&mutated),
            original.continuation_digest64
        );
        assert!(Simulation::from_checkpoint(mutated).is_err());
    }
}

#[test]
fn future_config_and_complete_migration_state_are_bound() {
    let original = small_checkpoint();

    let mut config_mutation = original.clone();
    config_mutation.experiment.duration_years += 1;
    assert!(Simulation::from_checkpoint(config_mutation).is_err());

    let mut migration_mutation = original.clone();
    migration_mutation.migration.eastward_steps =
        migration_mutation.migration.eastward_steps.wrapping_add(1);
    assert!(Simulation::from_checkpoint(migration_mutation).is_err());
}

#[test]
fn v0_3_0_checkpoint_schema_is_explicitly_not_silently_reinterpreted() {
    let mut checkpoint = small_checkpoint();
    checkpoint.schema_version = SimulationCheckpoint::PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION;
    let error = Simulation::from_checkpoint(checkpoint).unwrap_err().to_string();
    assert!(error.contains("predates complete continuation integrity"));
}

#[test]
fn resume_lineage_binds_source_complete_identity() {
    let source = small_checkpoint();
    let source_identity = source.continuation_digest64;
    let resumed = Simulation::from_checkpoint(source)
        .unwrap()
        .run_recorded()
        .unwrap();
    let boundary = resumed.manifest.resume_lineage.boundaries.last().unwrap();
    assert_eq!(boundary.source_continuation_digest64, source_identity);
}

#[test]
fn deserialized_world_and_population_reject_private_schema_and_zero_shape() {
    let world = World::generate(WorldConfig::new(2, 2), RngFactory::new(7)).unwrap();
    let mut world_json = serde_json::to_value(&world).unwrap();
    world_json["schemaVersion"] = serde_json::json!(999);
    let invalid_world: World = serde_json::from_value(world_json).unwrap();
    assert!(invalid_world.validate().is_err());

    let mut zero_json = serde_json::to_value(&world).unwrap();
    zero_json["width"] = serde_json::json!(0);
    zero_json["height"] = serde_json::json!(0);
    zero_json["cells"] = serde_json::json!([]);
    let zero_world: World = serde_json::from_value(zero_json).unwrap();
    assert!(zero_world.validate().is_err());

    let population = Population::initialize(PopulationConfig::new(8), &world, RngFactory::new(7))
        .unwrap();
    let mut population_json = serde_json::to_value(&population).unwrap();
    population_json["schemaVersion"] = serde_json::json!(999);
    let invalid_population: Population = serde_json::from_value(population_json).unwrap();
    assert!(invalid_population.validate(&world).is_err());
}

#[test]
fn duplicate_birth_history_and_corrupt_intermediate_metrics_are_rejected_after_rehash() {
    let mut demography = anthrosim_core::DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    demography.minimum_birth_spacing_days = 0;
    demography.male_parent_min_age_years = 0;
    demography.male_parent_max_age_years_exclusive = 100;

    let checkpoint = Simulation::new(
        ExperimentConfig::new(88, 2)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100).with_max_person_records(10_000))
            .with_demography(demography),
    )
    .unwrap()
    .checkpoint_at_year(2)
    .unwrap();

    let mut json = serde_json::to_value(&checkpoint).unwrap();
    let events = json["events"]["events"].as_array_mut().unwrap();
    let birth_indices: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            (event["event"]["type"] == "birth").then_some(index)
        })
        .collect();
    assert!(birth_indices.len() >= 2);
    let first_event = events[birth_indices[0]]["event"].clone();
    events[birth_indices[1]]["event"] = first_event;
    let mut duplicate_birth: SimulationCheckpoint = serde_json::from_value(json).unwrap();
    duplicate_birth.refresh_continuation_digest64();
    assert!(duplicate_birth.validate_invariants().is_err());

    let mut bad_metric = checkpoint;
    assert!(bad_metric.metrics.snapshots.len() >= 2);
    bad_metric.metrics.snapshots[0].population.living_population = bad_metric.metrics.snapshots[0]
        .population
        .living_population
        .wrapping_add(1);
    bad_metric.refresh_continuation_digest64();
    assert!(bad_metric.validate_invariants().is_err());
}

fn spatial_fixture() -> (anthrosim_core::SpatialLandscapeCheckpoint, LandscapeBundle) {
    let landscape = LandscapeBundle::new(
        2,
        2,
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 10,
            cell_size_y: 10,
            coordinate_unit: "m".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        },
        vec![LandscapeLayer {
            layer_id: "terrain".to_owned(),
            role: LandscapeLayerRole::TerrainTraversal,
            unit: "cost".to_owned(),
            value_domain: Some(LandscapeValueDomain { min: 1_000, max: 2_000 }),
            evidence_input_id: None,
            values: vec![Some(1_000), Some(1_250), Some(1_500), Some(2_000)],
        }],
    );
    let mechanism = SpatialMechanismConfig::new(
        "audit-spatial",
        vec![SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "terrain",
            "cost",
            LandscapeValueDomain { min: 1_000, max: 2_000 },
            1_000,
            2_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        )],
    );
    let checkpoint = SpatialLandscapeSimulation::new(
        ExperimentConfig::new(99, 1)
            .with_world(WorldConfig::new(2, 2))
            .with_population(PopulationConfig::new(8).with_max_person_records(128)),
        landscape.clone(),
        mechanism,
    )
    .unwrap()
    .checkpoint_at_year(0)
    .unwrap();
    (checkpoint, landscape)
}

#[test]
fn transformed_resume_rejects_rng_tampering_before_execution() {
    let (mut checkpoint, landscape) = spatial_fixture();
    checkpoint.core_checkpoint.rng.migration_uncertainty.low = checkpoint
        .core_checkpoint
        .rng
        .migration_uncertainty
        .low
        .wrapping_add(1);
    assert!(SpatialLandscapeSimulation::from_checkpoint(checkpoint, landscape).is_err());
}
''', encoding="utf-8")
