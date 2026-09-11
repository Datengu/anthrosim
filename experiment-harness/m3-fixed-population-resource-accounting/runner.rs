use std::{error::Error, thread};

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig,
};
use serde_json::{json, Value};

fn pick(value: &Value, pointer: &str) -> Value {
    value.pointer(pointer).cloned().unwrap_or(Value::Null)
}

fn no_event_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn main() -> Result<(), Box<dyn Error>> {
    let periods = [1_u16, 2, 4, 8, 12, 24];
    let mut handles = Vec::new();

    for resource_periods in periods {
        handles.push(thread::spawn(move || -> Result<Vec<Value>, String> {
            let mut rows = Vec::with_capacity(30);
            for seed in 7501_u64..=7530_u64 {
                let mut resources = ResourceConfig::synthetic_validation_v1()
                    .with_productivity_scale_permille(750)
                    .with_seasonality_scale_permille(0)
                    .with_annual_need_units_per_person(100);
                resources.periods_per_year = resource_periods;
                resources.condition_recovery_per_period = 0;
                resources.max_condition_loss_per_period = 0;
                resources.max_scarcity_mortality_probability_per_million = 0;

                let config = ExperimentConfig::new(seed, 100)
                    .with_world(WorldConfig::new(64, 64))
                    .with_population(
                        PopulationConfig::new(2000)
                            .with_target_household_size(5)
                            .with_max_person_records(2_000),
                    )
                    .with_demography(no_event_demography())
                    .with_resources(resources)
                    .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

                let manifest = Simulation::new(config)
                    .map_err(|e| e.to_string())?
                    .run()
                    .map_err(|e| e.to_string())?;
                let m = serde_json::to_value(&manifest).map_err(|e| e.to_string())?;

                let max_background = m
                    .pointer("/experiment/demography/mortalityBands")
                    .and_then(Value::as_array)
                    .map(|bands| {
                        bands
                            .iter()
                            .filter_map(|band| band.get("annualProbabilityPerMillion").and_then(Value::as_u64))
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(u64::MAX);
                let max_fertility = m
                    .pointer("/experiment/demography/fertilityBands")
                    .and_then(Value::as_array)
                    .map(|bands| {
                        bands
                            .iter()
                            .filter_map(|band| band.get("annualProbabilityPerMillion").and_then(Value::as_u64))
                            .max()
                            .unwrap_or(0)
                    })
                    .unwrap_or(u64::MAX);

                rows.push(json!({
                    "seed": seed,
                    "resourcePeriodsPerYear": resource_periods,
                    "configuredResourcePeriodsPerYear": pick(&m, "/experiment/resources/periodsPerYear"),
                    "configuredMigrationEnabled": pick(&m, "/experiment/migration/enabled"),
                    "configuredProductivityScalePermille": pick(&m, "/experiment/resources/productivityScalePermille"),
                    "configuredSeasonalityScalePermille": pick(&m, "/experiment/resources/seasonalityScalePermille"),
                    "configuredAnnualNeedUnitsPerPerson": pick(&m, "/experiment/resources/annualNeedUnitsPerPerson"),
                    "configuredConditionRecoveryPerPeriod": pick(&m, "/experiment/resources/conditionRecoveryPerPeriod"),
                    "configuredMaxConditionLossPerPeriod": pick(&m, "/experiment/resources/maxConditionLossPerPeriod"),
                    "configuredMaxConditionMortalityProbabilityPerMillion": pick(&m, "/experiment/resources/maxConditionMortalityProbabilityPerMillion"),
                    "configuredMaxBackgroundMortalityProbabilityPerMillion": max_background,
                    "configuredMaxFertilityProbabilityPerMillion": max_fertility,
                    "configuredHouseholdLifecycle": pick(&m, "/experiment/householdLifecycle"),
                    "configuredTemporaryMobility": pick(&m, "/experiment/temporaryMobility"),
                    "finalLivingPopulation": pick(&m, "/population/livingPopulation"),
                    "personRecords": pick(&m, "/population/personRecords"),
                    "initialFoodStock": pick(&m, "/resources/initialFoodStock"),
                    "regeneratedFood": pick(&m, "/resources/regeneratedFood"),
                    "harvestedFood": pick(&m, "/resources/harvestedFood"),
                    "consumedFood": pick(&m, "/resources/consumedFood"),
                    "resourceUnmetNeed": pick(&m, "/resources/unmetNeed"),
                    "finalFoodStock": pick(&m, "/resources/finalFoodStock"),
                    "householdPeriodsWithUnmetNeed": pick(&m, "/resources/householdPeriodsWithUnmetNeed"),
                    "conditionMortalityDeaths": pick(&m, "/resources/conditionMortalityDeaths"),
                    "meanLivingConditionPermille": pick(&m, "/resources/meanLivingConditionPermille"),
                    "resourcePeriodsProcessed": pick(&m, "/statistics/resourcePeriodsProcessed"),
                    "movesCompleted": pick(&m, "/migration/movesCompleted"),
                    "peopleMoved": pick(&m, "/migration/peopleMoved"),
                    "simulatedDays": pick(&m, "/statistics/simulatedDays"),
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
