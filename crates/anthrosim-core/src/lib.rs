//! Deterministic core primitives for AnthroSim.
//!
//! v0.1 begins with a deliberately small headless lifecycle. Domain systems
//! are added behind this boundary and remain independent of rendering/networking.

pub mod checkpoint;
pub mod config;
pub mod demography;
pub mod events;
pub mod evidence;
pub mod focal_region;
pub mod ids;
pub mod invariants;
pub mod landscape;
pub mod landscape_binding;
pub mod manifest;
pub mod metrics;
pub mod migration;
pub mod population;
pub mod provenance;
pub mod resources;
pub mod rng;
pub mod simulation;
pub mod spatial_mechanisms;
pub mod spatial_observability;
pub mod spatial_simulation;
pub mod temporary_history;
// M9.3's internal trigger/event helpers intentionally carry the complete causal context at one
// boundary. Keep the argument-count exception local to this module rather than weakening Clippy
// across the workspace; the public API remains compact and typed.
#[allow(clippy::too_many_arguments)]
pub mod temporary_mobility;
pub mod temporary_observability;
pub mod temporary_resource;
pub mod temporary_travel;
pub mod time;
pub mod world;

#[cfg(test)]
mod m9_integration_tests;

pub use checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64};
pub use config::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ResourceConfig, WorldConfig,
};
pub use demography::{
    DemographyConfigError, annual_probability_for_age, validate_demography_config,
};
pub use events::{
    DeathCause, EventKind, EventLog, EventProvenance, EventRecord, TemporaryJourneyIneligibility,
};
pub use evidence::{
    EvidenceCatalog, EvidenceError, EvidenceRecord, EvidenceSource, EvidenceTransformation,
    EvidenceUncertainty, ExternalInputEvidence, ParameterEvidenceLink, validate_evidence_catalog,
};
pub use focal_region::{FocalRegion, FocalRegionBindingError, FocalRegionError, FocalRegionSource};
pub use invariants::{
    InvariantError, InvariantReport, validate_checkpoint_invariants,
    validate_checkpoint_invariants_with_world, validate_recorded_run_invariants,
    validate_run_artifacts_with_world,
};
pub use landscape::{
    GridGeometry, LandscapeBundle, LandscapeError, LandscapeLayer, LandscapeLayerRole,
    LandscapeValueDomain,
};
pub use landscape_binding::{
    LandscapeBinding, LandscapeBindingError, LandscapeCheckpoint, LandscapeRecordedRun,
    LandscapeRunManifest, LandscapeSimulation, validate_landscape_recorded_run_invariants,
};
pub use manifest::{ArtifactSchemas, RunManifest, RunStatistics, StopReason};
pub use metrics::{
    MetricProvenance, MetricSeries, MetricSnapshot, MigrationMetrics, PopulationMetrics,
    ResourceMetrics,
};
pub use migration::{
    MigrationCheckpointState, MigrationConfigError, MigrationDecisionTrace, MigrationError,
    MigrationSummary, MigrationSystem, MigrationUtilityBreakdown, bounded_candidate_cells,
    candidate_count_upper_bound, migration_pressure_permille, validate_migration_config,
};
pub use population::{
    CellOccupancy, PersonSnapshot, Population, PopulationError, PopulationSummary,
    PopulationValidationError, ReproductiveSex,
};
pub use provenance::{
    MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, ResumeLineageError, SourceRevisionIdentity,
};
pub use resources::{
    ResourceConfigError, ResourceError, ResourceSummary, ResourceSystem, validate_resource_config,
};
pub use rng::RngStreamPosition;
pub use simulation::{RecordedRun, Simulation, SimulationError};
pub use spatial_mechanisms::{
    NoDataPolicy, SPATIAL_MODEL_SEMANTICS_ID, SpatialFieldTransform, SpatialMechanismConfig,
    SpatialMechanismError, SpatialMechanismOverlay, SpatialTargetField, TransformDirection,
    transform_landscape,
};
pub use spatial_observability::{
    SpatialCellObservability, SpatialDerivedCell, SpatialLayerDescriptor,
    SpatialMigrationDistanceBin, SpatialMigrationFlow, SpatialModelFacingCell,
    SpatialObservabilityError, SpatialObservabilityReport, SpatialObservabilitySource,
    SpatialObservabilitySummary, derive_spatial_observability,
};
pub use spatial_simulation::{
    SpatialLandscapeCheckpoint, SpatialLandscapeError, SpatialLandscapeRecordedRun,
    SpatialLandscapeRunManifest, SpatialLandscapeSimulation, SpatialMechanismBinding,
    validate_spatial_landscape_recorded_run,
};
pub use temporary_history::{TemporaryMobilityHistoryError, validate_temporary_mobility_history};
pub use temporary_mobility::{
    ActiveTemporaryJourney, HouseholdPresence, TemporaryJourneySkip, TemporaryMobilityConfig,
    TemporaryMobilityConfigError, TemporaryMobilityDayOutcome, TemporaryMobilityError,
    TemporaryMobilityExecutionError, TemporaryMobilityProgram, TemporaryMobilityProgramError,
    TemporaryMobilitySchedule, TemporaryMobilityState, TemporaryMobilityValidationError,
    TemporaryTravelResolution, TemporaryTravelTable, TemporaryTriggerTiming,
};
pub use temporary_observability::{
    TemporaryJourneyObservability, TemporaryJourneyObservedStatus,
    TemporaryMobilityCellObservability, TemporaryMobilityObservabilityError,
    TemporaryMobilityObservabilityReport, TemporaryMobilityObservabilitySource,
    TemporaryMobilityObservabilitySummary, TemporaryOriginCatchment, TemporaryVisitDurationBin,
    derive_temporary_mobility_observability,
};
pub use temporary_resource::{
    TemporaryResourceAccountingError, TemporaryResourcePeriod, TemporaryResourcePresenceDays,
};
pub use temporary_travel::{
    TemporaryTravelModel, TemporaryTravelModelError, temporary_travel_edge_cost,
};
pub use time::SimTime;
pub use world::{Cell, World, WorldError, WorldSummary, WorldValidationError};
