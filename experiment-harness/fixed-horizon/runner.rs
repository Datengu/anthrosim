use std::{error::Error, thread};
use anthrosim_core::{ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation, WorldConfig};
use serde_json::{Value, json};

fn pick(value: &Value, pointer: &str) -> Value {
    value.pointer(pointer).cloned().unwrap_or(Value::Null)
}

fn main() -> Result<(), Box<dyn Error>> {
    let decisions = [1_u16, 2, 4, 8, 12, 24];
    let mut handles = Vec::new();

    for decision_periods in decisions {
        handles.push(thread::spawn(move || -> Result<Vec<Value>, String> {
            let mut rows = Vec::with_capacity(30);
            for seed in 5101_u64..=5130_u64 {
                let mut resources = ResourceConfig::synthetic_validation_v1()
                    .with_productivity_scale_permille(150)
                    .with_seasonality_scale_permille(1000)
                    .with_annual_need_units_per_person(100);
                resources.periods_per_year = 4;
                let migration = MigrationConfig::synthetic_validation_v1()
                    .with_enabled(true)
                    .with_candidate_radius_cells(3)
                    .with_decision_periods_per_year(decision_periods);
                let config = ExperimentConfig::new(seed, 100)
                    .with_world(WorldConfig::new(64, 64))
                    .with_population(PopulationConfig::new(2000).with_target_household_size(5).with_max_person_records(1_000_000))
                    .with_resources(resources)
                    .with_migration(migration);
                let manifest = Simulation::new(config).map_err(|e| e.to_string())?.run().map_err(|e| e.to_string())?;
                let m = serde_json::to_value(&manifest).map_err(|e| e.to_string())?;
                rows.push(json!({
                    "seed": seed,
                    "resourcePeriodsPerYear": 4,
                    "decisionPeriodsPerYear": decision_periods,
                    "fixedPlanningPeriodsPerYear": 4,
                    "configuredResourcePeriodsPerYear": pick(&m, "/experiment/resources/periodsPerYear"),
                    "configuredDecisionPeriodsPerYear": pick(&m, "/experiment/migration/decisionPeriodsPerYear"),
                    "configuredRadius": pick(&m, "/experiment/migration/candidateRadiusCells"),
                    "configuredTravelConditionCostPerCell": pick(&m, "/experiment/migration/travelConditionCostPerCell"),
                    "finalLivingPopulation": pick(&m, "/population/livingPopulation"),
                    "resourceUnmetNeed": pick(&m, "/resources/unmetNeed"),
                    "conditionMortalityDeaths": pick(&m, "/resources/conditionMortalityDeaths"),
                    "decisionBoundaries": pick(&m, "/migration/decisionBoundaries"),
                    "householdsEvaluated": pick(&m, "/migration/householdsEvaluated"),
                    "householdsUnderPressure": pick(&m, "/migration/householdsUnderPressure"),
                    "movesCompleted": pick(&m, "/migration/movesCompleted"),
                    "peopleMoved": pick(&m, "/migration/peopleMoved"),
                    "totalDistanceCells": pick(&m, "/migration/totalDistanceCells"),
                    "travelConditionCostTotal": pick(&m, "/migration/travelConditionCostTotal"),
                    "stopReason": pick(&m, "/stopReason"),
                    "modelSemanticsId": pick(&m, "/modelSemanticsId"),
                    "gitCommit": pick(&m, "/gitCommit")
                }));
            }
            Ok(rows)
        }));
    }

    let mut rows = Vec::with_capacity(180);
    for handle in handles {
        rows.extend(handle.join().map_err(|_| "worker thread panicked")??);
    }
    rows.sort_by_key(|row| (row["decisionPeriodsPerYear"].as_u64().unwrap_or(0), row["seed"].as_u64().unwrap_or(0)));
    for row in rows {
        println!("{}", serde_json::to_string(&row).map_err(|e| e.to_string())?);
    }
    Ok(())
}
