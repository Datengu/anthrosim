use std::{fs, io, path::Path};

use anthrosim_core::SimulationCheckpoint;
use serde_json::{Map, Number, Value};

use crate::{bundle::artifact_fs, read_json, write_json};

use super::{DERIVED_ANALYSIS_SCHEMA_VERSION, DERIVED_POINT_ANALYSIS_SCHEMA_VERSION};

const RUN_EXACT_TOTAL_FIELDS: [(&str, &str); 4] = [
    (
        "migrationOriginResourceScoreTotal",
        "migration_origin_resource_score_total",
    ),
    (
        "migrationDestinationResourceScoreTotal",
        "migration_destination_resource_score_total",
    ),
    (
        "migrationOriginWaterSecurityScoreTotal",
        "migration_origin_water_security_score_total",
    ),
    (
        "migrationDestinationWaterSecurityScoreTotal",
        "migration_destination_water_security_score_total",
    ),
];

const POINT_RUN_WEIGHTED_RENAMES: [(&str, &str, &str, &str); 4] = [
    (
        "meanMigrationOriginResourceScorePermilleMoveObservedOnly",
        "runWeightedMeanOfRunMeanMigrationOriginResourceScorePermilleMoveObservedRunsOnly",
        "mean_migration_origin_resource_score_permille_move_observed_only",
        "run_weighted_mean_of_run_mean_migration_origin_resource_score_permille_move_observed_runs_only",
    ),
    (
        "meanMigrationDestinationResourceScorePermilleMoveObservedOnly",
        "runWeightedMeanOfRunMeanMigrationDestinationResourceScorePermilleMoveObservedRunsOnly",
        "mean_migration_destination_resource_score_permille_move_observed_only",
        "run_weighted_mean_of_run_mean_migration_destination_resource_score_permille_move_observed_runs_only",
    ),
    (
        "meanMigrationOriginWaterSecurityScorePermilleMoveObservedOnly",
        "runWeightedMeanOfRunMeanMigrationOriginWaterSecurityScorePermilleMoveObservedRunsOnly",
        "mean_migration_origin_water_security_score_permille_move_observed_only",
        "run_weighted_mean_of_run_mean_migration_origin_water_security_score_permille_move_observed_runs_only",
    ),
    (
        "meanMigrationDestinationWaterSecurityScorePermilleMoveObservedOnly",
        "runWeightedMeanOfRunMeanMigrationDestinationWaterSecurityScorePermilleMoveObservedRunsOnly",
        "mean_migration_destination_water_security_score_permille_move_observed_only",
        "run_weighted_mean_of_run_mean_migration_destination_water_security_score_permille_move_observed_runs_only",
    ),
];

const POINT_SUPPORT_FIELDS: [(&str, &str); 5] = [
    (
        "migrationMovesCompletedScientificallyEligibleOnly",
        "migration_moves_completed_scientifically_eligible_only",
    ),
    (
        "migrationOriginResourceScoreTotalScientificallyEligibleOnly",
        "migration_origin_resource_score_total_scientifically_eligible_only",
    ),
    (
        "migrationDestinationResourceScoreTotalScientificallyEligibleOnly",
        "migration_destination_resource_score_total_scientifically_eligible_only",
    ),
    (
        "migrationOriginWaterSecurityScoreTotalScientificallyEligibleOnly",
        "migration_origin_water_security_score_total_scientifically_eligible_only",
    ),
    (
        "migrationDestinationWaterSecurityScoreTotalScientificallyEligibleOnly",
        "migration_destination_water_security_score_total_scientifically_eligible_only",
    ),
];

const POINT_POOLED_FIELDS: [(&str, &str); 4] = [
    (
        "pooledMeanMigrationOriginResourceScorePermillePerMoveScientificallyEligibleOnly",
        "pooled_mean_migration_origin_resource_score_permille_per_move_scientifically_eligible_only",
    ),
    (
        "pooledMeanMigrationDestinationResourceScorePermillePerMoveScientificallyEligibleOnly",
        "pooled_mean_migration_destination_resource_score_permille_per_move_scientifically_eligible_only",
    ),
    (
        "pooledMeanMigrationOriginWaterSecurityScorePermillePerMoveScientificallyEligibleOnly",
        "pooled_mean_migration_origin_water_security_score_permille_per_move_scientifically_eligible_only",
    ),
    (
        "pooledMeanMigrationDestinationWaterSecurityScorePermillePerMoveScientificallyEligibleOnly",
        "pooled_mean_migration_destination_water_security_score_permille_per_move_scientifically_eligible_only",
    ),
];

const RUN_MEAN_FIELDS: [&str; 4] = [
    "migrationMeanOriginResourceScorePermille",
    "migrationMeanDestinationResourceScorePermille",
    "migrationMeanOriginWaterSecurityScorePermille",
    "migrationMeanDestinationWaterSecurityScorePermille",
];

const RUN_TOTAL_JSON_FIELDS: [&str; 4] = [
    "migrationOriginResourceScoreTotal",
    "migrationDestinationResourceScoreTotal",
    "migrationOriginWaterSecurityScoreTotal",
    "migrationDestinationWaterSecurityScoreTotal",
];

pub(super) fn upgrade_analysis_outputs(
    sweep_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let analysis = sweep_root.join("analysis");
    let runs_path = analysis.join("runs.json");
    let points_path = analysis.join("points.json");
    let summary_path = analysis.join("summary.json");

    let mut runs: Vec<Value> = read_json(&runs_path)?;
    attach_exact_run_migration_totals(sweep_root, &mut runs)?;

    let mut points: Vec<Value> = read_json(&points_path)?;
    upgrade_point_rows(&mut points, &runs)?;

    let mut summary: Value = read_json(&summary_path)?;
    let summary_object = object_mut(&mut summary, "analysis summary")?;
    summary_object.insert(
        "schemaVersion".to_owned(),
        Value::from(DERIVED_ANALYSIS_SCHEMA_VERSION),
    );
    summary_object.insert(
        "note".to_owned(),
        Value::String(
            "Descriptive analysis only. Point means pool provenance-valid scientific outcomes: durationReached and populationExtinct, while undefined denominator-based values remain null and are averaged only where defined. personRecordLimitReached is operational censoring and is excluded from scientific aggregates. Migration-quality point summaries expose two distinct estimands: an equal-run-weighted mean of each move-observed eligible run's within-run move mean, and an exact pooled per-completed-move mean using checkpoint score totals and the total eligible move count."
                .to_owned(),
        ),
    );

    write_json(&runs_path, &runs)?;
    write_json(&points_path, &points)?;
    write_json(&summary_path, &summary)?;
    upgrade_runs_csv(&analysis.join("runs.csv"), &runs)?;
    upgrade_points_csv(&analysis.join("points.csv"), &points)?;
    Ok(())
}

fn attach_exact_run_migration_totals(
    sweep_root: &Path,
    runs: &mut [Value],
) -> Result<(), Box<dyn std::error::Error>> {
    for run in runs {
        let object = object_mut(run, "derived run row")?;
        object.insert(
            "schemaVersion".to_owned(),
            Value::from(DERIVED_ANALYSIS_SCHEMA_VERSION),
        );

        let state = string_field(object, "state")?.to_owned();
        if state != "completed" {
            for (json_key, _) in RUN_EXACT_TOTAL_FIELDS {
                object.insert(json_key.to_owned(), Value::Null);
            }
            continue;
        }

        let manifest_relative_path = string_field(object, "manifestRelativePath")?;
        let checkpoint_path = sweep_root
            .join(manifest_relative_path)
            .with_file_name("checkpoint.json");
        if !artifact_fs::regular_file_exists(&checkpoint_path, "completed sweep checkpoint")? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "completed sweep run is missing checkpoint required for exact migration-quality totals: {}",
                    checkpoint_path.display()
                ),
            )
            .into());
        }
        let checkpoint: SimulationCheckpoint = read_json(&checkpoint_path)?;
        let row_moves = u64_field(object, "migrationMovesCompleted")?;
        if checkpoint.migration.moves_completed != row_moves {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "completed sweep run migration move count disagrees between run row ({row_moves}) and checkpoint ({})",
                    checkpoint.migration.moves_completed
                ),
            )
            .into());
        }

        let totals = [
            checkpoint.migration.origin_resource_score_total,
            checkpoint.migration.destination_resource_score_total,
            checkpoint.migration.origin_water_security_score_total,
            checkpoint.migration.destination_water_security_score_total,
        ];
        let maximum_total = row_moves.checked_mul(1_000).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "migration move count is too large to validate score totals",
            )
        })?;
        if totals.iter().any(|total| *total > maximum_total) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "checkpoint migration-quality score total exceeds movesCompleted * 1000",
            )
            .into());
        }
        if row_moves == 0 && totals.iter().any(|total| *total != 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "checkpoint contains non-zero migration-quality totals without completed moves",
            )
            .into());
        }

        for ((json_key, _), total) in RUN_EXACT_TOTAL_FIELDS.into_iter().zip(totals) {
            object.insert(json_key.to_owned(), Value::from(total));
        }
    }
    Ok(())
}

fn upgrade_point_rows(
    points: &mut [Value],
    runs: &[Value],
) -> Result<(), Box<dyn std::error::Error>> {
    for point in points {
        let point_object = object_mut(point, "derived point row")?;
        point_object.insert(
            "schemaVersion".to_owned(),
            Value::from(DERIVED_POINT_ANALYSIS_SCHEMA_VERSION),
        );
        let point_id = string_field(point_object, "pointId")?.to_owned();

        let eligible = runs
            .iter()
            .map(|run| self::object(run, "derived run row"))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|run| {
                run.get("pointId").and_then(Value::as_str) == Some(point_id.as_str())
                    && run.get("scientificAggregationStatus").and_then(Value::as_str)
                        == Some("eligibleScientificOutcome")
            })
            .collect::<Vec<_>>();

        let mut moves_total = 0_u64;
        let mut exact_totals = [0_u64; 4];
        let mut move_observed_runs = 0_u64;
        let mut run_mean_sums = [0_u128; 4];
        let mut run_mean_counts = [0_u64; 4];

        for run in &eligible {
            let moves = u64_field(run, "migrationMovesCompleted")?;
            moves_total = moves_total.checked_add(moves).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "scientifically eligible migration move total overflow",
                )
            })?;
            if moves > 0 {
                move_observed_runs = move_observed_runs.checked_add(1).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "move-observed run count overflow",
                    )
                })?;
            }

            for index in 0..4 {
                let exact = u64_field(run, RUN_TOTAL_JSON_FIELDS[index])?;
                exact_totals[index] = exact_totals[index].checked_add(exact).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "scientifically eligible migration-quality total overflow",
                    )
                })?;
                if let Some(mean) = optional_u64_field(run, RUN_MEAN_FIELDS[index])? {
                    run_mean_sums[index] += u128::from(mean);
                    run_mean_counts[index] = run_mean_counts[index].checked_add(1).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "run-level migration-quality support count overflow",
                        )
                    })?;
                }
            }
        }

        point_object.insert(
            "migrationMoveObservedRunsScientificallyEligibleOnly".to_owned(),
            Value::from(move_observed_runs),
        );
        point_object.insert(
            "migrationMoveOccurrenceFractionScientificallyEligibleOnly".to_owned(),
            if eligible.is_empty() {
                Value::Null
            } else {
                number_value(move_observed_runs as f64 / eligible.len() as f64)?
            },
        );

        for index in 0..4 {
            let (legacy_json, explicit_json, _, _) = POINT_RUN_WEIGHTED_RENAMES[index];
            point_object.remove(legacy_json);
            point_object.insert(
                explicit_json.to_owned(),
                if run_mean_counts[index] == 0 {
                    Value::Null
                } else {
                    number_value(run_mean_sums[index] as f64 / run_mean_counts[index] as f64)?
                },
            );
        }

        let support_values = [
            moves_total,
            exact_totals[0],
            exact_totals[1],
            exact_totals[2],
            exact_totals[3],
        ];
        for ((json_key, _), value) in POINT_SUPPORT_FIELDS.into_iter().zip(support_values) {
            point_object.insert(json_key.to_owned(), Value::from(value));
        }

        for index in 0..4 {
            let pooled = if moves_total == 0 {
                Value::Null
            } else {
                number_value(exact_totals[index] as f64 / moves_total as f64)?
            };
            point_object.insert(POINT_POOLED_FIELDS[index].0.to_owned(), pooled);
        }
    }
    Ok(())
}

fn upgrade_runs_csv(path: &Path, runs: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    let header = lines.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "derived runs CSV is empty")
    })?;
    let rows = lines.collect::<Vec<_>>();
    if rows.len() != runs.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "derived runs CSV row count does not match runs.json",
        )
        .into());
    }

    let mut output = String::new();
    output.push_str(header);
    for (_, csv_key) in RUN_EXACT_TOTAL_FIELDS {
        output.push(',');
        output.push_str(csv_key);
    }
    output.push('\n');

    for (line, run) in rows.into_iter().zip(runs) {
        output.push_str(line);
        let object = object(run, "derived run row")?;
        for (json_key, _) in RUN_EXACT_TOTAL_FIELDS {
            output.push(',');
            output.push_str(&scalar_csv(object.get(json_key).unwrap_or(&Value::Null))?);
        }
        output.push('\n');
    }
    artifact_fs::atomic_write(path, output.as_bytes(), "upgraded sweep run analysis CSV")?;
    Ok(())
}

fn upgrade_points_csv(path: &Path, points: &[Value]) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines();
    let header_line = lines.next().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "derived points CSV is empty")
    })?;
    let rows = lines.collect::<Vec<_>>();
    if rows.len() != points.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "derived points CSV row count does not match points.json",
        )
        .into());
    }

    let mut headers = header_line.split(',').map(str::to_owned).collect::<Vec<_>>();
    let mut renamed_indices = [0_usize; 4];
    for index in 0..4 {
        let (_, _, legacy_csv, explicit_csv) = POINT_RUN_WEIGHTED_RENAMES[index];
        let position = headers
            .iter()
            .position(|header| header == legacy_csv)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("derived points CSV is missing legacy weighting column {legacy_csv}"),
                )
            })?;
        headers[position] = explicit_csv.to_owned();
        renamed_indices[index] = position;
    }
    headers.extend(POINT_SUPPORT_FIELDS.iter().map(|(_, csv)| (*csv).to_owned()));
    headers.extend(POINT_POOLED_FIELDS.iter().map(|(_, csv)| (*csv).to_owned()));

    let mut output = String::new();
    output.push_str(&headers.join(","));
    output.push('\n');

    for (line, point) in rows.into_iter().zip(points) {
        let mut fields = parse_csv_line(line)?;
        if fields.len() + POINT_SUPPORT_FIELDS.len() + POINT_POOLED_FIELDS.len() != headers.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "derived points CSV field count does not match upgraded header",
            )
            .into());
        }
        let object = object(point, "derived point row")?;
        for index in 0..4 {
            let explicit_json = POINT_RUN_WEIGHTED_RENAMES[index].1;
            fields[renamed_indices[index]] =
                scalar_csv(object.get(explicit_json).unwrap_or(&Value::Null))?;
        }
        for (json_key, _) in POINT_SUPPORT_FIELDS {
            fields.push(scalar_csv(object.get(json_key).unwrap_or(&Value::Null))?);
        }
        for (json_key, _) in POINT_POOLED_FIELDS {
            fields.push(scalar_csv(object.get(json_key).unwrap_or(&Value::Null))?);
        }
        output.push_str(&csv_line(&fields));
    }

    artifact_fs::atomic_write(path, output.as_bytes(), "upgraded sweep point analysis CSV")?;
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

fn optional_u64_field(
    object: &Map<String, Value>,
    key: &str,
) -> Result<Option<u64>, io::Error> {
    match object.get(key) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("derived analysis field {key} is not an unsigned integer or null"),
            )
        }),
    }
}

fn number_value(value: f64) -> Result<Value, io::Error> {
    Number::from_f64(value).map(Value::Number).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "derived analysis produced a non-finite floating-point value",
        )
    })
}

fn scalar_csv(value: &Value) -> Result<String, io::Error> {
    match value {
        Value::Null => Ok(String::new()),
        Value::String(value) => Ok(value.clone()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cannot serialize non-scalar derived-analysis value into a CSV scalar",
        )),
    }
}

fn parse_csv_line(line: &str) -> Result<Vec<String>, io::Error> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut characters = line.chars().peekable();
    let mut quoted = false;
    while let Some(character) = characters.next() {
        match character {
            '"' if quoted && characters.peek() == Some(&'"') => {
                current.push('"');
                characters.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(character),
        }
    }
    if quoted {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "derived-analysis CSV contains an unterminated quoted field",
        ));
    }
    fields.push(current);
    Ok(fields)
}

fn csv_line(fields: &[String]) -> String {
    let mut line = fields
        .iter()
        .map(|field| csv_escape(field))
        .collect::<Vec<_>>()
        .join(",");
    line.push('\n');
    line
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn eligible_run(
        point_id: &str,
        moves: u64,
        origin_resource_mean: u64,
        destination_resource_mean: u64,
        origin_water_mean: u64,
        destination_water_mean: u64,
    ) -> Value {
        json!({
            "pointId": point_id,
            "scientificAggregationStatus": "eligibleScientificOutcome",
            "migrationMovesCompleted": moves,
            "migrationMeanOriginResourceScorePermille": origin_resource_mean,
            "migrationMeanDestinationResourceScorePermille": destination_resource_mean,
            "migrationMeanOriginWaterSecurityScorePermille": origin_water_mean,
            "migrationMeanDestinationWaterSecurityScorePermille": destination_water_mean,
            "migrationOriginResourceScoreTotal": origin_resource_mean * moves,
            "migrationDestinationResourceScoreTotal": destination_resource_mean * moves,
            "migrationOriginWaterSecurityScoreTotal": origin_water_mean * moves,
            "migrationDestinationWaterSecurityScoreTotal": destination_water_mean * moves
        })
    }

    #[test]
    fn unequal_move_counts_keep_run_weighted_and_move_weighted_estimands_distinct() {
        let runs = vec![
            eligible_run("A", 1, 0, 0, 0, 0),
            eligible_run("A", 99, 1_000, 1_000, 1_000, 1_000),
            eligible_run("B", 1, 600, 600, 600, 600),
            eligible_run("B", 99, 600, 600, 600, 600),
        ];
        let mut points = vec![json!({"pointId": "A"}), json!({"pointId": "B"})];

        upgrade_point_rows(&mut points, &runs).expect("upgrade points");
        let a = points[0].as_object().expect("A object");
        let b = points[1].as_object().expect("B object");

        let a_run_weighted = a
            .get("runWeightedMeanOfRunMeanMigrationDestinationResourceScorePermilleMoveObservedRunsOnly")
            .and_then(Value::as_f64)
            .expect("A run-weighted mean");
        let a_pooled = a
            .get("pooledMeanMigrationDestinationResourceScorePermillePerMoveScientificallyEligibleOnly")
            .and_then(Value::as_f64)
            .expect("A pooled mean");
        let b_run_weighted = b
            .get("runWeightedMeanOfRunMeanMigrationDestinationResourceScorePermilleMoveObservedRunsOnly")
            .and_then(Value::as_f64)
            .expect("B run-weighted mean");
        let b_pooled = b
            .get("pooledMeanMigrationDestinationResourceScorePermillePerMoveScientificallyEligibleOnly")
            .and_then(Value::as_f64)
            .expect("B pooled mean");

        assert_eq!(a_run_weighted, 500.0);
        assert_eq!(a_pooled, 990.0);
        assert_eq!(b_run_weighted, 600.0);
        assert_eq!(b_pooled, 600.0);
        assert!(b_run_weighted > a_run_weighted, "run weighting ranks B above A");
        assert!(a_pooled > b_pooled, "move weighting ranks A above B");
        assert_eq!(
            a.get("migrationMovesCompletedScientificallyEligibleOnly")
                .and_then(Value::as_u64),
            Some(100)
        );
        assert_eq!(
            a.get("migrationDestinationResourceScoreTotalScientificallyEligibleOnly")
                .and_then(Value::as_u64),
            Some(99_000)
        );
        assert!(
            a.get("meanMigrationDestinationResourceScorePermilleMoveObservedOnly")
                .is_none()
        );
    }

    #[test]
    fn zero_move_points_leave_both_quality_estimands_undefined() {
        let runs = vec![json!({
            "pointId": "A",
            "scientificAggregationStatus": "eligibleScientificOutcome",
            "migrationMovesCompleted": 0,
            "migrationMeanOriginResourceScorePermille": null,
            "migrationMeanDestinationResourceScorePermille": null,
            "migrationMeanOriginWaterSecurityScorePermille": null,
            "migrationMeanDestinationWaterSecurityScorePermille": null,
            "migrationOriginResourceScoreTotal": 0,
            "migrationDestinationResourceScoreTotal": 0,
            "migrationOriginWaterSecurityScoreTotal": 0,
            "migrationDestinationWaterSecurityScoreTotal": 0
        })];
        let mut points = vec![json!({"pointId": "A"})];

        upgrade_point_rows(&mut points, &runs).expect("upgrade points");
        let point = points[0].as_object().expect("point object");
        assert_eq!(
            point.get("migrationMoveObservedRunsScientificallyEligibleOnly"),
            Some(&Value::from(0))
        );
        assert_eq!(
            point.get("migrationMoveOccurrenceFractionScientificallyEligibleOnly")
                .and_then(Value::as_f64),
            Some(0.0)
        );
        assert_eq!(
            point.get("runWeightedMeanOfRunMeanMigrationOriginResourceScorePermilleMoveObservedRunsOnly"),
            Some(&Value::Null)
        );
        assert_eq!(
            point.get("pooledMeanMigrationOriginResourceScorePermillePerMoveScientificallyEligibleOnly"),
            Some(&Value::Null)
        );
    }
}
