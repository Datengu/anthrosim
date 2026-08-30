use std::{io, path::Path};

use crate::ensemble::EnsembleRunSettings;

#[path = "sweep_legacy.rs"]
mod legacy;
mod sweep_weighting;

pub(crate) use legacy::SweepDimensions;

const DERIVED_ANALYSIS_SCHEMA_VERSION: u32 = 6;
const DERIVED_POINT_ANALYSIS_SCHEMA_VERSION: u32 = 7;

pub(crate) fn execute_sweep(
    directory: &Path,
    base_settings: EnsembleRunSettings,
    seeds: Vec<u64>,
    dimensions: SweepDimensions,
    retry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let execution = legacy::execute_sweep(directory, base_settings, seeds, dimensions, retry);
    let upgrade = if directory.join("analysis").is_dir() {
        sweep_weighting::upgrade_analysis_outputs(directory)
    } else {
        Ok(())
    };

    match (execution, upgrade) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Err(execution_error), Err(upgrade_error)) => Err(io::Error::other(format!(
            "sweep execution failed: {execution_error}; derived-analysis weighting upgrade also failed: {upgrade_error}"
        ))
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::Value;

    use super::*;

    fn small_settings() -> EnsembleRunSettings {
        EnsembleRunSettings {
            years: 0,
            world_width: 4,
            world_height: 4,
            population: 12,
            household_size: 4,
            max_person_records: 100,
            resource_productivity_scale_permille: 1_000,
            resource_seasonality_scale_permille: 1_000,
            annual_food_need: 100,
            disable_migration: false,
            migration_radius: 3,
            temporary_mobility: None,
            spatial: None,
        }
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after Unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("anthrosim-{label}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn current_sweep_outputs_publish_explicit_migration_weighting_contract() {
        let root = temp_path("sweep-migration-weighting-contract");
        let dimensions = SweepDimensions {
            population: vec![12],
            household_size: vec![],
            resource_productivity_scale_permille: vec![],
            resource_seasonality_scale_permille: vec![],
            annual_food_need: vec![],
            disable_migration: vec![],
            migration_radius: vec![],
        };

        execute_sweep(&root, small_settings(), vec![1], dimensions, false)
            .expect("zero-year sweep");

        let runs: Value = crate::read_json(&root.join("analysis/runs.json")).expect("runs json");
        let run = &runs.as_array().expect("run array")[0];
        assert_eq!(run.get("schemaVersion"), Some(&Value::from(6)));
        assert_eq!(
            run.get("migrationOriginResourceScoreTotal"),
            Some(&Value::from(0))
        );
        assert_eq!(
            run.get("migrationDestinationResourceScoreTotal"),
            Some(&Value::from(0))
        );

        let points: Value =
            crate::read_json(&root.join("analysis/points.json")).expect("points json");
        let point = &points.as_array().expect("point array")[0];
        assert_eq!(point.get("schemaVersion"), Some(&Value::from(7)));
        assert_eq!(
            point.get("migrationMovesCompletedScientificallyEligibleOnly"),
            Some(&Value::from(0))
        );
        assert_eq!(
            point.get("pooledMeanMigrationDestinationResourceScorePermillePerMoveScientificallyEligibleOnly"),
            Some(&Value::Null)
        );
        assert_eq!(
            point.get("runWeightedMeanOfRunMeanMigrationDestinationResourceScorePermilleMoveObservedRunsOnly"),
            Some(&Value::Null)
        );
        assert!(
            point
                .get("meanMigrationDestinationResourceScorePermilleMoveObservedOnly")
                .is_none()
        );

        let points_csv = fs::read_to_string(root.join("analysis/points.csv")).expect("points csv");
        assert!(points_csv.contains(
            "run_weighted_mean_of_run_mean_migration_destination_resource_score_permille_move_observed_runs_only"
        ));
        assert!(points_csv.contains(
            "pooled_mean_migration_destination_resource_score_permille_per_move_scientifically_eligible_only"
        ));
        assert!(!points_csv.contains(
            ",mean_migration_destination_resource_score_permille_move_observed_only,"
        ));

        fs::remove_dir_all(root).expect("cleanup");
    }
}
