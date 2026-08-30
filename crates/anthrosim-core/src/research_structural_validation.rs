use std::collections::BTreeSet;

use serde_json::Value;

use super::{ResearchDimension, ResearchExperimentError, canonical_json_bytes};

pub(super) fn validate_distinct_executable_alternatives(
    base: &Value,
    dimension: &ResearchDimension,
) -> Result<(), ResearchExperimentError> {
    let mut projections = BTreeSet::new();
    for value in &dimension.values {
        let mut resolved = base.clone();
        let target = resolved
            .pointer_mut(&dimension.path)
            .ok_or_else(|| ResearchExperimentError::UnknownDimensionPath(dimension.path.clone()))?;
        *target = value.clone();
        strip_noncausal_metadata(&mut resolved, &mut Vec::new());
        let projection = canonical_json_bytes(&resolved)?;
        if !projections.insert(projection) {
            return Err(
                ResearchExperimentError::StructuralDimensionDoesNotProvideDistinctExecutableAlternatives {
                    id: dimension.id.clone(),
                    path: dimension.path.clone(),
                },
            );
        }
    }
    if projections.len() < 2 {
        return Err(
            ResearchExperimentError::StructuralDimensionDoesNotProvideDistinctExecutableAlternatives {
                id: dimension.id.clone(),
                path: dimension.path.clone(),
            },
        );
    }
    Ok(())
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
    if matches!(last, "provenance" | "evidenceId" | "evidenceInputId" | "scheduleId") {
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

    fn projection(mut value: Value) -> Value {
        strip_noncausal_metadata(&mut value, &mut Vec::new());
        value
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
}
