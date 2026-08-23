//! Deterministic core primitives for AnthroSim.
//!
//! v0.1 begins with a deliberately small headless lifecycle. Domain systems
//! are added behind this boundary and remain independent of rendering/networking.

pub mod checkpoint;
pub mod config;
pub mod demography;
pub mod events;
pub mod evidence;
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
pub mod time;
pub mod world;

pub use checkpoint::{RngCheckpoint, SimulationCheckpoint, state_digest64};
pub use config::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ResourceConfig, WorldConfig,
};
pub use demography::{
    DemographyConfigError, annual_probability_for_age, validate_demography_config,
};
pub use events::{DeathCause, EventKind, EventLog, EventProvenance, EventRecord};
pub use evidence::{
    EvidenceCatalog, EvidenceError, EvidenceRecord, EvidenceSource, EvidenceTransformation,
    EvidenceUncertainty, ExternalInputEvidence, ParameterEvidenceLink, validate_evidence_catalog,
};
pub use invariants::{
    InvariantError, InvariantReport, validate_checkpoint_invariants,
    validate_recorded_run_invariants,
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
pub use time::SimTime;
pub use world::{Cell, World, WorldError, WorldSummary, WorldValidationError};
