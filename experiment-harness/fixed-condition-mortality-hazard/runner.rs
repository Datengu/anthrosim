use std::{error::Error, thread};

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, MigrationConfig, ParameterProvenance, PopulationConfig,
    PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};
use serde_json::{json, Value};

const INITIAL_POPULATION: u32 = 2_000;
const MAX_CONDITION_MORTALITY_PPM: u32 = 200_000;

fn pick(value: &Value, pointer: &str) -> Value {
    value.pointer(pointer).cloned().unwrap_or(Value::Null)
}

fn founders(condition_permille: u16) -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    let people = (1..=u64::from(INITIAL_POPULATION))
        .map(|id| FounderPerson {
            id: PersonId::new(id),
            birth_day: -(30 * 365),
            reproductive_sex: ReproductiveSex::Male,
            household,
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille,
        })
        .collect();

    FounderPopulationDefinition::new(
        format!("fixed-condition-{condition_permille}-v1"),
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        people,
    )
}

fn zero_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn run_block(condition_permille: u16) -> Result<Vec<Value>, String> {
    let periods = [1_u16, 2, 4, 8, 12, 24];
    let mut rows = Vec::with_capacity(periods.len() * 30);

    for resource_periods in periods {
        for seed in 8_101_u64..=8_130_u64 {
            let mut resources = ResourceConfig::synthetic_validation_v1()
                .with_annual_need_units_per_person(0)
                .with_initial_stock_units_per_productivity(0)
                .with_seasonality_scale_permille(0);
            resources.periods_per_year = resource_periods;
            resources.condition_recovery_per_period = 0;
            resources.max_condition_loss_per_period = 0;
            resources.max_scarcity_mortality_probability_per_million =
                MAX_CONDITION_MORTALITY_PPM;

            let config = ExperimentConfig::new(seed, 5)
                .with_world(WorldConfig::new(1, 1))
                .with_population(
                    PopulationConfig::new(INITIAL_POPULATION)
                        .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                        .with_max_person_records(5_000),
                )
                .with_founder_population(founders(condition_permille))
                .with_demography(zero_demography())
                .with_resources(resources)
                .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

            let manifest = Simulation::new(config)
                .map_err(|e| e.to_string())?
                .run()
                .map_err(|e| e.to_string())?;
            let m = serde_json::to_value(&manifest).map_err(|e| e.to_string())?;

            rows.push(json!({
                "fixedConditionPermille": condition_permille,
                "resourcePeriodsPerYear": resource_periods,
                "seed": seed,
                "configuredResourcePeriodsPerYear": pick(&m, "/experiment/resources/periodsPerYear"),
                "configuredAnnualNeedUnitsPerPerson": pick(&m, "/experiment/resources/annualNeedUnitsPerPerson"),
                "configuredConditionRecoveryPerPeriod": pick(&m, "/experiment/resources/conditionRecoveryPerPeriod"),
                "configuredMaxConditionLossPerPeriod": pick(&m, "/experiment/resources/maxConditionLossPerPeriod"),
                "configuredMaxConditionMortalityProbabilityPerMillion": pick(&m, "/experiment/resources/maxConditionMortalityProbabilityPerMillion"),
                "configuredMigrationEnabled": pick(&m, "/experiment/migration/enabled"),
                "finalLivingPopulation": pick(&m, "/population/livingPopulation"),
                "conditionMortalityDeaths": pick(&m, "/resources/conditionMortalityDeaths"),
                "meanLivingConditionPermille": pick(&m, "/resources/meanLivingConditionPermille"),
                "resourceUnmetNeed": pick(&m, "/resources/unmetNeed"),
                "movesCompleted": pick(&m, "/migration/movesCompleted"),
                "simulatedDays": pick(&m, "/statistics/simulatedDays"),
                "stopReason": pick(&m, "/stopReason"),
                "modelSemanticsId": pick(&m, "/modelSemanticsId"),
                "gitCommit": pick(&m, "/gitCommit")
            }));
        }
    }

    Ok(rows)
}

fn main() -> Result<(), Box<dyn Error>> {
    let conditions = [250_u16, 500, 750, 900];
    let mut handles = Vec::new();
    for condition in conditions {
        handles.push(thread::spawn(move || run_block(condition)));
    }

    let mut rows = Vec::with_capacity(720);
    for handle in handles {
        rows.extend(handle.join().map_err(|_| "worker thread panicked")??);
    }
    rows.sort_by_key(|row| (
        row["fixedConditionPermille"].as_u64().unwrap_or(0),
        row["resourcePeriodsPerYear"].as_u64().unwrap_or(0),
        row["seed"].as_u64().unwrap_or(0),
    ));

    for row in rows {
        println!("{}", serde_json::to_string(&row)?);
    }
    Ok(())
}
