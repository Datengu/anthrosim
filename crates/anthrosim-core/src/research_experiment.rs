use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{
    ExperimentConfig, LandscapeBundle, Simulation, SourceRevisionIdentity,
    SpatialLandscapeSimulation, SpatialMechanismConfig,
};

/// Versioned research-facing sensitivity/uncertainty definition.
///
/// The authoritative scientific configuration remains `ExperimentConfig`. This type only adds a
/// deterministic Cartesian design around one exact base configuration; it does not introduce a
/// second set of model defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchExperimentDefinition {
    pub schema_version: u32,
    pub seeds: Vec<u64>,
    pub base: ResearchRunConfig,
    #[serde(default)]
    pub dimensions: Vec<ResearchDimension>,
}

impl ResearchExperimentDefinition {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn validate(&self) -> Result<(), ResearchExperimentError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(ResearchExperimentError::UnsupportedDefinitionSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.base.experiment.schema_version != ExperimentConfig::CURRENT_SCHEMA_VERSION {
            return Err(ResearchExperimentError::UnsupportedBaseExperimentSchema {
                found: self.base.experiment.schema_version,
                supported: ExperimentConfig::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.seeds.is_empty() {
            return Err(ResearchExperimentError::NoSeeds);
        }
        let mut seeds = BTreeSet::new();
        for seed in &self.seeds {
            if !seeds.insert(*seed) {
                return Err(ResearchExperimentError::DuplicateSeed(*seed));
            }
        }
        if self.base.experiment.seed != self.seeds[0] {
            return Err(ResearchExperimentError::BaseSeedMismatch {
                base_seed: self.base.experiment.seed,
                first_seed: self.seeds[0],
            });
        }

        let base = serde_json::to_value(&self.base)
            .map_err(|error| ResearchExperimentError::Serialization(error.to_string()))?;
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for dimension in &self.dimensions {
            dimension.validate_against(&base)?;
            if !ids.insert(dimension.id.clone()) {
                return Err(ResearchExperimentError::DuplicateDimensionId(
                    dimension.id.clone(),
                ));
            }
            if !paths.insert(dimension.path.clone()) {
                return Err(ResearchExperimentError::DuplicateDimensionPath(
                    dimension.path.clone(),
                ));
            }
        }
        Ok(())
    }

    /// Expand dimensions in declaration order and values in listed order.
    ///
    /// The final dimension varies fastest. This ordering is part of schema v1 and is covered by
    /// tests because point indices feed immutable experiment/retry identity.
    pub fn expand(&self) -> Result<Vec<ResearchPoint>, ResearchExperimentError> {
        self.validate()?;

        let mut partial = vec![(self.base.clone(), Vec::<ResearchCoordinate>::new())];
        for dimension in &self.dimensions {
            let mut next = Vec::with_capacity(partial.len().saturating_mul(dimension.values.len()));
            for (base, coordinates) in partial {
                for value in &dimension.values {
                    let mut run_config = base.clone();
                    apply_dimension(&mut run_config, dimension, value)?;
                    let mut point_coordinates = coordinates.clone();
                    point_coordinates.push(ResearchCoordinate {
                        id: dimension.id.clone(),
                        kind: dimension.kind,
                        path: dimension.path.clone(),
                        value: value.clone(),
                    });
                    next.push((run_config, point_coordinates));
                }
            }
            partial = next;
        }

        partial
            .into_iter()
            .enumerate()
            .map(|(index, (run_config, coordinates))| {
                let index =
                    u64::try_from(index).map_err(|_| ResearchExperimentError::TooManyPoints)?;
                Ok(ResearchPoint {
                    schema_version: ResearchPoint::CURRENT_SCHEMA_VERSION,
                    index,
                    point_id: point_identity(index, &run_config, &coordinates)?,
                    coordinates,
                    run_config,
                })
            })
            .collect()
    }

    pub fn identity(&self) -> Result<String, ResearchExperimentError> {
        self.validate()?;
        stable_identity("research-definition-v1", self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchRunConfig {
    pub experiment: ExperimentConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial: Option<ResearchSpatialConfig>,
}

impl ResearchRunConfig {
    #[must_use]
    pub fn for_seed(&self, seed: u64) -> Self {
        let mut config = self.clone();
        config.experiment.seed = seed;
        config
    }

    pub fn identity(&self) -> Result<String, ResearchExperimentError> {
        stable_identity("research-run-config-v1", self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchSpatialConfig {
    pub landscape: LandscapeBundle,
    pub mechanisms: SpatialMechanismConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchDimensionKind {
    Numeric,
    Structural,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchDimension {
    pub id: String,
    pub kind: ResearchDimensionKind,
    pub path: String,
    pub values: Vec<Value>,
}

impl ResearchDimension {
    fn validate_against(&self, base: &Value) -> Result<(), ResearchExperimentError> {
        if self.id.trim().is_empty() {
            return Err(ResearchExperimentError::EmptyDimensionId);
        }
        validate_pointer_contract(&self.path)?;
        let Some(target) = base.pointer(&self.path) else {
            return Err(ResearchExperimentError::UnknownDimensionPath(
                self.path.clone(),
            ));
        };
        if self.values.is_empty() {
            return Err(ResearchExperimentError::NoDimensionValues(self.id.clone()));
        }

        match self.kind {
            ResearchDimensionKind::Numeric if !target.is_number() => {
                return Err(ResearchExperimentError::NumericDimensionTargetsNonNumeric {
                    id: self.id.clone(),
                    path: self.path.clone(),
                });
            }
            ResearchDimensionKind::Structural if target.is_number() => {
                return Err(ResearchExperimentError::StructuralDimensionTargetsNumeric {
                    id: self.id.clone(),
                    path: self.path.clone(),
                });
            }
            _ => {}
        }

        let mut canonical_values = BTreeSet::new();
        for value in &self.values {
            match self.kind {
                ResearchDimensionKind::Numeric if !value.is_number() => {
                    return Err(
                        ResearchExperimentError::NumericDimensionHasNonNumericValue {
                            id: self.id.clone(),
                        },
                    );
                }
                ResearchDimensionKind::Structural if value.is_number() => {
                    return Err(
                        ResearchExperimentError::StructuralDimensionHasNumericValue {
                            id: self.id.clone(),
                        },
                    );
                }
                _ => {}
            }
            let canonical = canonical_json_bytes(value)?;
            if !canonical_values.insert(canonical) {
                return Err(ResearchExperimentError::DuplicateDimensionValue(
                    self.id.clone(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchCoordinate {
    pub id: String,
    pub kind: ResearchDimensionKind,
    pub path: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResearchPoint {
    pub schema_version: u32,
    pub index: u64,
    pub point_id: String,
    pub coordinates: Vec<ResearchCoordinate>,
    pub run_config: ResearchRunConfig,
}

impl ResearchPoint {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;
}

/// Exact immutable identity for one executed seed/configuration/source triple.
pub fn research_run_identity(
    point_id: &str,
    run_config: &ResearchRunConfig,
    source: &SourceRevisionIdentity,
) -> Result<String, ResearchExperimentError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        point_id: &'a str,
        run_config: &'a ResearchRunConfig,
        source: &'a SourceRevisionIdentity,
    }
    stable_identity(
        "research-run-v1",
        &Identity {
            schema_version: 1,
            point_id,
            run_config,
            source,
        },
    )
}

/// Validate a fully resolved configuration through the normal authoritative simulation host.
///
/// This constructs initialization state but advances no simulated time. The subsequent execution
/// still calls the same constructor again, so the research runner cannot bypass normal validation.
pub fn validate_resolved_research_run(
    run_config: &ResearchRunConfig,
) -> Result<(), ResearchExperimentError> {
    match &run_config.spatial {
        Some(spatial) => SpatialLandscapeSimulation::new(
            run_config.experiment.clone(),
            spatial.landscape.clone(),
            spatial.mechanisms.clone(),
        )
        .map(|_| ())
        .map_err(|error| ResearchExperimentError::InvalidResolvedConfiguration(error.to_string())),
        None => Simulation::new(run_config.experiment.clone())
            .map(|_| ())
            .map_err(|error| {
                ResearchExperimentError::InvalidResolvedConfiguration(error.to_string())
            }),
    }
}

fn apply_dimension(
    run_config: &mut ResearchRunConfig,
    dimension: &ResearchDimension,
    value: &Value,
) -> Result<(), ResearchExperimentError> {
    let mut encoded = serde_json::to_value(&*run_config)
        .map_err(|error| ResearchExperimentError::Serialization(error.to_string()))?;
    let target = encoded
        .pointer_mut(&dimension.path)
        .ok_or_else(|| ResearchExperimentError::UnknownDimensionPath(dimension.path.clone()))?;
    *target = value.clone();
    *run_config = serde_json::from_value(encoded).map_err(|error| {
        ResearchExperimentError::InvalidDimensionApplication {
            id: dimension.id.clone(),
            path: dimension.path.clone(),
            reason: error.to_string(),
        }
    })?;
    Ok(())
}

fn validate_pointer_contract(path: &str) -> Result<(), ResearchExperimentError> {
    if path.is_empty() || !path.starts_with('/') {
        return Err(ResearchExperimentError::InvalidDimensionPath(
            path.to_owned(),
        ));
    }
    let segments = pointer_segments(path)?;
    if segments
        .first()
        .is_none_or(|segment| segment.as_str() != "experiment" && segment.as_str() != "spatial")
    {
        return Err(ResearchExperimentError::InvalidDimensionRoot(
            path.to_owned(),
        ));
    }
    if segments.iter().any(|segment| segment == "schemaVersion") {
        return Err(ResearchExperimentError::ReservedDimensionPath(
            path.to_owned(),
        ));
    }
    if segments.as_slice() == ["experiment", "seed"] {
        return Err(ResearchExperimentError::SeedIsNotDimension);
    }
    Ok(())
}

fn pointer_segments(path: &str) -> Result<Vec<String>, ResearchExperimentError> {
    path.split('/')
        .skip(1)
        .map(|raw| {
            let mut output = String::new();
            let mut chars = raw.chars();
            while let Some(character) = chars.next() {
                if character != '~' {
                    output.push(character);
                    continue;
                }
                match chars.next() {
                    Some('0') => output.push('~'),
                    Some('1') => output.push('/'),
                    _ => {
                        return Err(ResearchExperimentError::InvalidDimensionPath(
                            path.to_owned(),
                        ));
                    }
                }
            }
            Ok(output)
        })
        .collect()
}

fn point_identity(
    index: u64,
    run_config: &ResearchRunConfig,
    coordinates: &[ResearchCoordinate],
) -> Result<String, ResearchExperimentError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Identity<'a> {
        schema_version: u32,
        index: u64,
        coordinates: &'a [ResearchCoordinate],
        run_config: &'a ResearchRunConfig,
    }
    stable_identity(
        "research-point-v1",
        &Identity {
            schema_version: 1,
            index,
            coordinates,
            run_config,
        },
    )
}

fn stable_identity<T: Serialize>(
    prefix: &str,
    value: &T,
) -> Result<String, ResearchExperimentError> {
    let encoded = serde_json::to_value(value)
        .map_err(|error| ResearchExperimentError::Serialization(error.to_string()))?;
    let bytes = canonical_json_bytes(&encoded)?;
    Ok(format!("{prefix}-{:016x}", fnv1a64(&bytes)))
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, ResearchExperimentError> {
    fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
            Value::Object(values) => {
                let mut keys: Vec<_> = values.keys().collect();
                keys.sort_unstable();
                let mut output = serde_json::Map::new();
                for key in keys {
                    output.insert(key.clone(), canonicalize(&values[key]));
                }
                Value::Object(output)
            }
            _ => value.clone(),
        }
    }
    serde_json::to_vec(&canonicalize(value))
        .map_err(|error| ResearchExperimentError::Serialization(error.to_string()))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResearchExperimentError {
    #[error("unsupported research-definition schema {found}; supported schema is {supported}")]
    UnsupportedDefinitionSchema { found: u32, supported: u32 },
    #[error("base ExperimentConfig schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedBaseExperimentSchema { found: u32, supported: u32 },
    #[error("research definition must declare at least one seed")]
    NoSeeds,
    #[error("research definition contains duplicate seed {0}")]
    DuplicateSeed(u64),
    #[error(
        "base ExperimentConfig seed {base_seed} must equal the first declared seed {first_seed}"
    )]
    BaseSeedMismatch { base_seed: u64, first_seed: u64 },
    #[error("research dimension id may not be empty")]
    EmptyDimensionId,
    #[error("duplicate research dimension id {0}")]
    DuplicateDimensionId(String),
    #[error("duplicate research dimension path {0}")]
    DuplicateDimensionPath(String),
    #[error("research dimension {0} must contain at least one value")]
    NoDimensionValues(String),
    #[error("research dimension {0} contains a duplicate value")]
    DuplicateDimensionValue(String),
    #[error("invalid research dimension JSON pointer {0}")]
    InvalidDimensionPath(String),
    #[error("research dimension path must be rooted at /experiment or /spatial: {0}")]
    InvalidDimensionRoot(String),
    #[error("research dimension path targets reserved schema identity: {0}")]
    ReservedDimensionPath(String),
    #[error(
        "ExperimentConfig.seed is controlled by the research definition seeds list, not a dimension"
    )]
    SeedIsNotDimension,
    #[error("research dimension path does not exist in the exact base configuration: {0}")]
    UnknownDimensionPath(String),
    #[error("numeric dimension {id} targets non-numeric path {path}")]
    NumericDimensionTargetsNonNumeric { id: String, path: String },
    #[error(
        "structural dimension {id} targets numeric path {path}; numeric parameters must use kind=numeric"
    )]
    StructuralDimensionTargetsNumeric { id: String, path: String },
    #[error("numeric dimension {id} contains a non-numeric value")]
    NumericDimensionHasNonNumericValue { id: String },
    #[error("structural dimension {id} contains a numeric value")]
    StructuralDimensionHasNumericValue { id: String },
    #[error("dimension {id} at {path} cannot be applied to authoritative configuration: {reason}")]
    InvalidDimensionApplication {
        id: String,
        path: String,
        reason: String,
    },
    #[error("resolved research configuration is invalid: {0}")]
    InvalidResolvedConfiguration(String),
    #[error("research definition expands beyond supported point indexing")]
    TooManyPoints,
    #[error("research-definition serialization failed: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DemographyConfig, GridGeometry, LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain,
        MigrationConfig, PopulationConfig, ResourceConfig, WorldConfig,
        spatial_mechanisms::{
            NoDataPolicy, SpatialFieldTransform, SpatialTargetField, TransformDirection,
        },
    };

    fn base_definition() -> ResearchExperimentDefinition {
        let mut experiment = ExperimentConfig::new(11, 2)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(20)
                    .with_target_household_size(5)
                    .with_max_person_records(100),
            )
            .with_demography(DemographyConfig::synthetic_validation_v1())
            .with_resources(ResourceConfig::synthetic_validation_v1())
            .with_migration(MigrationConfig::synthetic_validation_v1());
        experiment
            .resources
            .max_scarcity_mortality_probability_per_million = 0;
        ResearchExperimentDefinition {
            schema_version: ResearchExperimentDefinition::CURRENT_SCHEMA_VERSION,
            seeds: vec![11, 12],
            base: ResearchRunConfig {
                experiment,
                spatial: None,
            },
            dimensions: Vec::new(),
        }
    }

    fn numeric(id: &str, path: &str, values: &[u64]) -> ResearchDimension {
        ResearchDimension {
            id: id.to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: path.to_owned(),
            values: values.iter().copied().map(Value::from).collect(),
        }
    }

    #[test]
    fn representative_cross_subsystem_dimensions_expand_in_declared_order() {
        let mut definition = base_definition();
        definition.dimensions = vec![
            numeric(
                "m2_birth_spacing_days",
                "/experiment/demography/minimumBirthSpacingDays",
                &[300, 400],
            ),
            numeric(
                "m3_periods_per_year",
                "/experiment/resources/periodsPerYear",
                &[4, 12],
            ),
            numeric(
                "m3_condition_mortality_response",
                "/experiment/resources/maxConditionMortalityProbabilityPerMillion",
                &[0, 1000],
            ),
            numeric(
                "m4_travel_cost_weight",
                "/experiment/migration/travelCostWeight",
                &[1, 2],
            ),
        ];
        let points = definition.expand().unwrap();
        assert_eq!(points.len(), 16);
        assert_eq!(points[0].coordinates[0].value, Value::from(300));
        assert_eq!(points[0].coordinates[3].value, Value::from(1));
        assert_eq!(points[1].coordinates[3].value, Value::from(2));
        assert_eq!(points[2].coordinates[2].value, Value::from(1000));
        assert_eq!(points[8].coordinates[0].value, Value::from(400));
        assert_eq!(
            points[15].run_config.experiment.resources.periods_per_year,
            12
        );
        assert_eq!(
            points[15]
                .run_config
                .experiment
                .migration
                .travel_cost_weight,
            2
        );
    }

    #[test]
    fn same_definition_has_same_identity_and_scientific_change_changes_it() {
        let definition = base_definition();
        let first = definition.identity().unwrap();
        assert_eq!(first, definition.clone().identity().unwrap());

        let mut changed = definition;
        changed.base.experiment.resources.periods_per_year += 1;
        assert_ne!(first, changed.identity().unwrap());
    }

    #[test]
    fn seed_pairing_changes_only_the_authoritative_experiment_seed() {
        let point = base_definition().expand().unwrap().remove(0);
        let first = point.run_config.for_seed(11);
        let second = point.run_config.for_seed(12);
        assert_eq!(first.experiment.seed, 11);
        assert_eq!(second.experiment.seed, 12);
        let mut normalized = second;
        normalized.experiment.seed = 11;
        assert_eq!(first, normalized);
    }

    #[test]
    fn unknown_path_type_error_and_invalid_result_fail_closed() {
        let mut unknown = base_definition();
        unknown.dimensions.push(numeric(
            "typo",
            "/experiment/resources/periodsPerYeear",
            &[12],
        ));
        assert!(matches!(
            unknown.expand(),
            Err(ResearchExperimentError::UnknownDimensionPath(_))
        ));

        let mut wrong_kind = base_definition();
        wrong_kind.dimensions.push(ResearchDimension {
            id: "bad_type".to_owned(),
            kind: ResearchDimensionKind::Numeric,
            path: "/experiment/migration/enabled".to_owned(),
            values: vec![Value::from(1)],
        });
        assert!(matches!(
            wrong_kind.expand(),
            Err(ResearchExperimentError::NumericDimensionTargetsNonNumeric { .. })
        ));

        let mut invalid_width = base_definition();
        invalid_width.dimensions.push(numeric(
            "world_width",
            "/experiment/world/width",
            &[u64::from(u32::MAX) + 1],
        ));
        assert!(matches!(
            invalid_width.expand(),
            Err(ResearchExperimentError::InvalidDimensionApplication { .. })
        ));
    }

    #[test]
    fn seed_and_schema_paths_are_reserved() {
        let mut seed = base_definition();
        seed.dimensions
            .push(numeric("seed", "/experiment/seed", &[99]));
        assert_eq!(
            seed.expand(),
            Err(ResearchExperimentError::SeedIsNotDimension)
        );

        let mut schema = base_definition();
        schema.dimensions.push(numeric(
            "schema",
            "/experiment/resources/schemaVersion",
            &[4],
        ));
        assert!(matches!(
            schema.expand(),
            Err(ResearchExperimentError::ReservedDimensionPath(_))
        ));
    }

    #[test]
    fn structural_dimension_is_explicit_and_cannot_be_pooled_as_numeric() {
        let mut definition = base_definition();
        definition.dimensions.push(ResearchDimension {
            id: "migration_enabled".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: "/experiment/migration/enabled".to_owned(),
            values: vec![Value::Bool(false), Value::Bool(true)],
        });
        let points = definition.expand().unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(
            points[0].coordinates[0].kind,
            ResearchDimensionKind::Structural
        );
        assert!(!points[0].run_config.experiment.migration.enabled);
        assert!(points[1].run_config.experiment.migration.enabled);

        let mut misclassified = base_definition();
        misclassified.dimensions.push(ResearchDimension {
            id: "periods".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: "/experiment/resources/periodsPerYear".to_owned(),
            values: vec![Value::from(12)],
        });
        assert!(matches!(
            misclassified.expand(),
            Err(ResearchExperimentError::StructuralDimensionTargetsNumeric { .. })
        ));
    }

    #[test]
    fn spatial_structural_alternative_is_part_of_point_configuration() {
        let mut definition = base_definition();
        let landscape = LandscapeBundle::new(
            4,
            4,
            GridGeometry {
                origin_x: 0,
                origin_y: 400,
                cell_size_x: 100,
                cell_size_y: 100,
                coordinate_unit: "metre".to_owned(),
                spatial_reference: "EPSG:27700".to_owned(),
            },
            vec![LandscapeLayer {
                layer_id: "terrain".to_owned(),
                role: LandscapeLayerRole::TerrainTraversal,
                unit: "cost".to_owned(),
                value_domain: Some(LandscapeValueDomain { min: 0, max: 100 }),
                evidence_input_id: None,
                values: vec![Some(50); 16],
            }],
        );
        let mechanisms = SpatialMechanismConfig::new(
            "test-spatial",
            vec![SpatialFieldTransform::new(
                SpatialTargetField::MovementCost,
                "terrain",
                "cost",
                LandscapeValueDomain { min: 0, max: 100 },
                100,
                1000,
                TransformDirection::Direct,
                NoDataPolicy::Reject,
            )],
        );
        definition.base.spatial = Some(ResearchSpatialConfig {
            landscape,
            mechanisms,
        });
        definition.dimensions.push(ResearchDimension {
            id: "m8_transform_direction".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: "/spatial/mechanisms/transforms/0/direction".to_owned(),
            values: vec![
                Value::String("direct".to_owned()),
                Value::String("inverse".to_owned()),
            ],
        });
        let points = definition.expand().unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(
            points[0].coordinates[0].kind,
            ResearchDimensionKind::Structural
        );
        assert_ne!(points[0].point_id, points[1].point_id);
    }

    #[test]
    fn invalid_model_variant_is_rejected_by_normal_execution_validation() {
        let mut definition = base_definition();
        definition.dimensions.push(ResearchDimension {
            id: "migration_model".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: "/experiment/migration/modelId".to_owned(),
            values: vec![Value::String("not-a-real-model".to_owned())],
        });
        let point = definition.expand().unwrap().remove(0);
        assert!(matches!(
            validate_resolved_research_run(&point.run_config),
            Err(ResearchExperimentError::InvalidResolvedConfiguration(_))
        ));
    }
}
