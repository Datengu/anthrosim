use std::{io, path::Path};

use serde_json::{Map, Number, Value};

use crate::{read_json, write_json};

const POOLED_DISTANCE_FIELD: &str =
    "pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly";

pub(super) fn restore_existing_pooled_distance(
    sweep_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let analysis = sweep_root.join("analysis");
    let runs: Vec<Value> = read_json(&analysis.join("runs.json"))?;
    let mut points: Vec<Value> = read_json(&analysis.join("points.json"))?;
    restore_point_rows(&mut points, &runs)?;
    write_json(&analysis.join("points.json"), &points)?;
    Ok(())
}

fn restore_point_rows(points: &mut [Value], runs: &[Value]) -> Result<(), io::Error> {
    for point in points {
        let point_object = object_mut(point, "derived point row")?;
        let point_id = string_field(point_object, "pointId")?.to_owned();
        let mut moves = 0_u128;
        let mut distance = 0_u128;

        for run in runs {
            let run = object(run, "derived run row")?;
            if run.get("pointId").and_then(Value::as_str) != Some(point_id.as_str())
                || run
                    .get("scientificAggregationStatus")
                    .and_then(Value::as_str)
                    != Some("eligibleScientificOutcome")
            {
                continue;
            }

            moves = moves
                .checked_add(u128::from(u64_field(run, "migrationMovesCompleted")?))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "scientifically eligible migration move total overflow",
                    )
                })?;
            distance = distance
                .checked_add(u128::from(u64_field(run, "migrationTotalDistanceCells")?))
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "scientifically eligible migration distance total overflow",
                    )
                })?;
        }

        point_object.insert(
            POOLED_DISTANCE_FIELD.to_owned(),
            if moves == 0 {
                Value::Null
            } else {
                number_value(distance as f64 / moves as f64)?
            },
        );
    }
    Ok(())
}

fn object<'a>(value: &'a Value, role: &str) -> Result<&'a Map<String, Value>, io::Error> {
    value.as_object().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{role} is not a JSON object"),
        )
    })
}

fn object_mut<'a>(
    value: &'a mut Value,
    role: &str,
) -> Result<&'a mut Map<String, Value>, io::Error> {
    value.as_object_mut().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{role} is not a JSON object"),
        )
    })
}

fn string_field<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str, io::Error> {
    object.get(key).and_then(Value::as_str).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("derived analysis field {key} is missing or not a string"),
        )
    })
}

fn u64_field(object: &Map<String, Value>, key: &str) -> Result<u64, io::Error> {
    object.get(key).and_then(Value::as_u64).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("derived analysis field {key} is missing or not an unsigned integer"),
        )
    })
}

fn number_value(value: f64) -> Result<Value, io::Error> {
    Number::from_f64(value).map(Value::Number).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "derived analysis produced a non-finite floating-point value",
        )
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn pooled_distance_is_reconstructed_from_exact_integer_support() {
        let runs = vec![json!({
            "pointId": "point-000002",
            "scientificAggregationStatus": "eligibleScientificOutcome",
            "migrationMovesCompleted": 226_758,
            "migrationTotalDistanceCells": 463_613
        })];
        let mut points = vec![json!({
            "pointId": "point-000002",
            "pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly": 2.044527646213143
        })];

        restore_point_rows(&mut points, &runs).expect("restore pooled distance");

        assert_eq!(
            points[0].get(POOLED_DISTANCE_FIELD).and_then(Value::as_f64),
            Some(2.0445276462131434)
        );
        assert_eq!(
            points[0].get(POOLED_DISTANCE_FIELD).and_then(Value::as_f64),
            Some(463_613_f64 / 226_758_f64)
        );
    }

    #[test]
    fn zero_move_points_keep_existing_pooled_distance_undefined() {
        let runs = vec![json!({
            "pointId": "A",
            "scientificAggregationStatus": "eligibleScientificOutcome",
            "migrationMovesCompleted": 0,
            "migrationTotalDistanceCells": 0
        })];
        let mut points = vec![json!({
            "pointId": "A",
            "pooledMeanMigrationDistanceCellsPerMoveScientificallyEligibleOnly": null
        })];

        restore_point_rows(&mut points, &runs).expect("restore zero-move point");
        assert_eq!(points[0].get(POOLED_DISTANCE_FIELD), Some(&Value::Null));
    }
}
