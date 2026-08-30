use std::collections::BTreeSet;

use serde_json::Value;

use super::{ResearchDimension, ResearchExperimentError, canonical_json_bytes};

pub(super) fn validate_distinct_executable_alternatives(
    base: &Value,
    dimension: &ResearchDimension,
) -> Result<(), ResearchExperimentError> {
    let base_projection = executable_projection(base)?;
    let mut projections = BTreeSet::new();
    for value in &dimension.values {
        let mut resolved = base.clone();
        let target = resolved
            .pointer_mut(&dimension.path)
            .ok_or_else(|| ResearchExperimentError::UnknownDimensionPath(dimension.path.clone()))?;
        *target = value.clone();
        let projection = executable_projection(&resolved)?;
        if !projections.insert(projection) {
            return Err(noncausal_structural_dimension(dimension));
        }
    }

    // A one-level structural dimension remains useful as an explicit treatment override, but it
    // must actually differ from the base executable configuration. With two or more levels,
    // pairwise-distinct projections above prove that the declared alternatives are genuinely
    // different executable structures; one level may legitimately represent the base structure.
    if projections.len() == 1 && projections.contains(&base_projection) {
        return Err(noncausal_structural_dimension(dimension));
    }
    Ok(())
}

fn executable_projection(value: &Value) -> Result<Vec<u8>, ResearchExperimentError> {
    let mut projected = value.clone();
    strip_noncausal_metadata(&mut projected, &mut Vec::new());
    canonical_json_bytes(&projected)
}

fn noncausal_structural_dimension(dimension: &ResearchDimension) -> ResearchExperimentError {
    ResearchExperimentError::StructuralDimensionDoesNotProvideDistinctExecutableAlternatives {
        id: dimension.id.clone(),
        path: dimension.path.clone(),
    }
}

fn strip_noncausal_metadata(value: &mut Value, path: &mut Vec<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                strip_noncausal_metadata(value, path);
            }
        }
        Value::Object(object) => {
            let keys = object.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                path.push(key.clone());
                if is_noncausal_metadata_path(path) {
                    object.remove(&key);
                } else if let Some(child) = object.get_mut(&key) {
                    strip_noncausal_metadata(child, path);
                }
                path.pop();
            }
        }
        _ => {}
    }
}

fn is_noncausal_metadata_path(path: &[String]) -> bool {
    let Some(last) = path.last().map(String::as_str) else {
        return false;
    };

    // These fields preserve provenance, evidence linkage or human-facing identity, but are not
    // read by transition equations. They remain in immutable research/run identity; they are
    // removed only from the causal projection used to decide whether a coordinate deserves the
    // `structural` treatment classification.
    if matches!(
        last,
        "provenance" | "evidenceId" | "evidenceInputId" | "scheduleId"
    ) {
        return true;
    }
    if path_is(path, &["experiment", "evidence"])
        || path_is(path, &["spatial", "mechanisms", "runRealization"])
        || path_is(
            path,
            &["experiment", "temporaryMobility", "region", "regionId"],
        )
        || path_is(
            path,
            &["experiment", "temporaryMobility", "region", "source"],
        )
    {
        return true;
    }

    if last != "modelId" {
        return false;
    }

    // Do not blanket-strip modelId. HouseholdLifecycleConfig.modelId is an executable selector
    // validated against the implemented lifecycle model. The identifiers below are descriptive
    // labels for parameterizations whose executable fields are represented separately.
    path_is(path, &["experiment", "resources", "modelId"])
        || path_is(path, &["experiment", "migration", "modelId"])
        || path_is(
            path,
            &[
                "experiment",
                "temporaryMobility",
                "travelModel",
                "modelId",
            ],
        )
        || path_is(path, &["spatial", "mechanisms", "modelId"])
}

fn path_is(path: &[String], expected: &[&str]) -> bool {
    path.len() == expected.len()
        && path
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual == expected)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::ResearchDimensionKind;

    fn projection(mut value: Value) -> Value {
        strip_noncausal_metadata(&mut value, &mut Vec::new());
        value
    }

    fn structural(path: &str, values: Vec<Value>) -> ResearchDimension {
        ResearchDimension {
            id: "structure".to_owned(),
            kind: ResearchDimensionKind::Structural,
            path: path.to_owned(),
            values,
        }
    }

    #[test]
    fn known_provenance_labels_do_not_change_executable_projection() {
        let first = json!({
            "experiment": {
                "demography": {"scheduleId": "first", "provenance": "source-a", "rate": 7},
                "resources": {"modelId": "label-a", "periodsPerYear": 4},
                "migration": {"modelId": "label-a", "enabled": true},
                "evidence": {"catalogId": "a"}
            },
            "spatial": {
                "mechanisms": {
                    "modelId": "label-a",
                    "runRealization": {"environmentSeed": 1},
                    "transforms": [{"direction": "direct", "evidenceId": "e-a"}]
                }
            }
        });
        let second = json!({
            "experiment": {
                "demography": {"scheduleId": "second", "provenance": "source-b", "rate": 7},
                "resources": {"modelId": "label-b", "periodsPerYear": 4},
                "migration": {"modelId": "label-b", "enabled": true},
                "evidence": {"catalogId": "b"}
            },
            "spatial": {
                "mechanisms": {
                    "modelId": "label-b",
                    "runRealization": {"environmentSeed": 99},
                    "transforms": [{"direction": "direct", "evidenceId": "e-b"}]
                }
            }
        });
        assert_eq!(projection(first), projection(second));
    }

    #[test]
    fn executable_selectors_and_bindings_remain_in_projection() {
        let first = json!({
            "experiment": {
                "householdLifecycle": {"modelId": "model-a"}
            },
            "spatial": {
                "mechanisms": {
                    "transforms": [{"sourceLayerId": "terrain-a", "direction": "direct"}]
                }
            }
        });
        let second = json!({
            "experiment": {
                "householdLifecycle": {"modelId": "model-b"}
            },
            "spatial": {
                "mechanisms": {
                    "transforms": [{"sourceLayerId": "terrain-b", "direction": "inverse"}]
                }
            }
        });
        assert_ne!(projection(first), projection(second));
    }

    #[test]
    fn duplicate_metadata_only_levels_are_rejected() {
        let base = json!({
            "experiment": {
                "demography": {"scheduleId": "baseline", "rate": 7}
            }
        });
        let dimension = structural(
            "/experiment/demography/scheduleId",
            vec![Value::from("alternative-a"), Value::from("alternative-b")],
        );
        assert!(matches!(
            validate_distinct_executable_alternatives(&base, &dimension),
            Err(
                ResearchExperimentError::StructuralDimensionDoesNotProvideDistinctExecutableAlternatives { .. }
            )
        ));
    }

    #[test]
    fn one_level_metadata_only_override_is_rejected_against_base() {
        let base = json!({
            "experiment": {
                "resources": {"modelId": "baseline", "periodsPerYear": 4}
            }
        });
        let dimension = structural(
            "/experiment/resources/modelId",
            vec![Value::from("renamed-only")],
        );
        assert!(matches!(
            validate_distinct_executable_alternatives(&base, &dimension),
            Err(
                ResearchExperimentError::StructuralDimensionDoesNotProvideDistinctExecutableAlternatives { .. }
            )
        ));
    }

    #[test]
    fn whole_object_levels_with_real_causal_differences_are_accepted() {
        let base = json!({
            "experiment": {
                "demography": {"scheduleId": "baseline", "rate": 7}
            }
        });
        let dimension = structural(
            "/experiment/demography",
            vec![
                json!({"scheduleId": "alternative-a", "rate": 7}),
                json!({"scheduleId": "alternative-b", "rate": 9}),
            ],
        );
        validate_distinct_executable_alternatives(&base, &dimension)
            .expect("causally distinct whole-object alternatives");
    }
}
