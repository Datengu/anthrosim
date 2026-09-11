use std::{error::Error, thread};
use anthrosim_core::{
    ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation, WorldConfig,
};
use serde_json::{json, Value};

fn pick(value: &Value, pointer: &str) -> Value {
    value.pointer(pointer).cloned().unwrap_or(Value::Null)
}

fn main() -> Result<(), Box<dyn Error>> {
    let periods = [1_u16, 2, 4, 8, 12, 24];
    let mut handles = Vec::new();

    for resource_periods in periods {
        handles.push(thread::spawn(move || -> Result<Vec<Value>, String> {
            let mut rows = Vec::with_capacity(30);
            for seed in 7101_u64..=7130_u64 {
                let mut resources = ResourceConfig::synthetic_validation_v1()
                    .with_productivity_scale_permille(150)
                    .with_seasonality_scale_permille(1000)
                    .with_annual_need_units_per_person(100);
                resources.periods_per_year = resource_periods;

                let migration = MigrationConfig::synthetic_validation_v1().with_enabled(false);

                let config = ExperimentConfig::new(seed, 100)
                    .with_world(WorldConfig::new(64, 64))
                    .with_population(
                        PopulationConfig::new(2000)
                            .with_target_household_size(5)
                            .with_max_person_records(1_000_000),
                    )
                    .with_resources(resources)
                    .with_migration(migration);

                let manifest = Simulation::new(config)
                    .map_err(|e| e.to_string())?
                    .run()
                    .map_err(|e| e.to_string())?;
                let m = serde_json::to_value(&manifest).map_err(|e| e.to_string())?;

                rows.push(json!({
                    "seed": seed,
                    "resourcePeriodsPerYear": resource_periods,
                    "configuredResourcePeriodsPerYear": pick(&m, "/experiment/resources/periodsPerYear"),
                    "configuredMigrationEnabled": pick(&m, "/experiment/migration/enabled"),
                    "finalLivingPopulation": pick(&m, "/population/livingPopulation"),
                    "resourceUnmetNeed": pick(&m, "/resources/unmetNeed"),
                    "conditionMortalityDeaths": pick(&m, "/resources/conditionMortalityDeaths"),
                    "meanLivingConditionPermille": pick(&m, "/resources/meanLivingConditionPermille"),
                    "simulatedDays": pick(&m, "/statistics/simulatedDays"),
                    "resourcePeriodsProcessed": pick(&m, "/statistics/resourcePeriodsProcessed"),
                    "migrationDecisionBoundaries": pick(&m, "/statistics/migrationDecisionBoundaries"),
                    "movesCompleted": pick(&m, "/migration/movesCompleted"),
                    "peopleMoved": pick(&m, "/migration/peopleMoved"),
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
    rows.sort_by_key(|row| (
        row["resourcePeriodsPerYear"].as_u64().unwrap_or(0),
        row["seed"].as_u64().unwrap_or(0),
    ));
    for row in rows {
        println!("{}", serde_json::to_string(&row)?);
    }
    Ok(())
}
