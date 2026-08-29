use std::collections::BTreeMap;

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, HouseholdLifecycleConfig,
    MigrationConfig, Population, PopulationConfig, ResourceConfig, Simulation,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, World, WorldConfig, derive_household_observability,
    derive_temporary_mobility_observability, ids::CellId, rng::RngFactory,
};
use serde::Serialize;

const DURATION_YEARS: u64 = 40;
const SEEDS: [u64; 8] = [20701, 20702, 20703, 20704, 20705, 20706, 20707, 20708];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArmAggregate {
    lifecycle_model_id: String,
    completed_runs: u64,
    population_extinct_runs: u64,
    terminal_living_population_total: u64,
    terminal_active_households_total: u64,
    terminal_largest_household_size_total: u64,
    terminal_multigenerational_households_total: u64,
    terminal_living_occupied_cells_total: u64,
    terminal_household_size_distribution: BTreeMap<u32, u64>,
    terminal_household_age_days_distribution: BTreeMap<u64, u64>,
    terminal_household_generation_span_distribution: BTreeMap<u32, u64>,
    mean_living_condition_permille_sum: u64,
    mean_living_condition_defined_runs: u64,
    unmet_need_total: u64,
    migration_moves_total: u64,
    migration_people_moved_total: u64,
    temporary_departures_total: u64,
    temporary_visitor_person_days_total: u64,
    temporary_visitor_household_days_total: u64,
    temporary_peak_visitors_max: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Comparison {
    schema_version: u32,
    purpose: &'static str,
    scientific_status: &'static str,
    seeds: Vec<u64>,
    duration_years: u64,
    founder_population: u32,
    founder_target_household_size: u16,
    alternative_max_living_members: u16,
    baseline: ArmAggregate,
    deterministic_size_fission: ArmAggregate,
}

fn replacement_demography() -> DemographyConfig {
    serde_json::from_str(include_str!(
        "../../../research/demography-controls-v1/replacement-control.json"
    ))
    .unwrap()
}

fn config(seed: u64, fission: bool) -> ExperimentConfig {
    let trigger_days = (0..DURATION_YEARS)
        .map(|year| year * 365 + 180)
        .collect::<Vec<_>>();
    let region = FocalRegion::new(
        "issue-207-structural-sensitivity-region",
        FocalRegionSource::Synthetic,
        vec![
            CellId::new(1),
            CellId::new(2),
            CellId::new(3),
            CellId::new(4),
        ],
    )
    .unwrap();
    let temporary_mobility = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            "issue-207-annual-midyear",
            TemporaryTriggerTiming::DepartureDay,
            trigger_days,
            7,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();
    let resources = ResourceConfig::synthetic_validation_v1();
    let mut config = ExperimentConfig::new(seed, DURATION_YEARS)
        .with_world(WorldConfig::new(12, 12))
        .with_population(PopulationConfig::new(120).with_target_household_size(5))
        .with_demography(replacement_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1())
        .with_temporary_mobility(temporary_mobility);
    if fission {
        config = config.with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(8, 18),
        );
    }
    config
}

fn add_bin<K: Ord + Copy>(target: &mut BTreeMap<K, u64>, key: K, count: u64) {
    *target.entry(key).or_default() += count;
}

fn aggregate(fission: bool) -> ArmAggregate {
    let mut aggregate = ArmAggregate {
        lifecycle_model_id: String::new(),
        completed_runs: 0,
        population_extinct_runs: 0,
        terminal_living_population_total: 0,
        terminal_active_households_total: 0,
        terminal_largest_household_size_total: 0,
        terminal_multigenerational_households_total: 0,
        terminal_living_occupied_cells_total: 0,
        terminal_household_size_distribution: BTreeMap::new(),
        terminal_household_age_days_distribution: BTreeMap::new(),
        terminal_household_generation_span_distribution: BTreeMap::new(),
        mean_living_condition_permille_sum: 0,
        mean_living_condition_defined_runs: 0,
        unmet_need_total: 0,
        migration_moves_total: 0,
        migration_people_moved_total: 0,
        temporary_departures_total: 0,
        temporary_visitor_person_days_total: 0,
        temporary_visitor_household_days_total: 0,
        temporary_peak_visitors_max: 0,
    };
    for seed in SEEDS {
        let run_config = config(seed, fission);
        let factory = RngFactory::new(seed);
        let world = World::generate(run_config.world, factory).unwrap();
        let initial_population =
            Population::initialize(run_config.population, &world, factory).unwrap();
        let run = Simulation::new(run_config).unwrap().run_recorded().unwrap();
        let household = derive_household_observability(
            &run.checkpoint.population,
            &run.checkpoint.experiment,
            &run.checkpoint.events,
            run.checkpoint.time.days(),
        )
        .unwrap();
        let temporary =
            derive_temporary_mobility_observability(&world, &initial_population, &run.checkpoint)
                .unwrap();
        aggregate.lifecycle_model_id = household.lifecycle_model_id.clone();
        aggregate.completed_runs +=
            u64::from(run.checkpoint.completed_years == run.checkpoint.experiment.duration_years);
        aggregate.population_extinct_runs +=
            u64::from(run.checkpoint.population.living_count() == 0);
        aggregate.terminal_living_population_total += run.checkpoint.population.living_count();
        aggregate.terminal_active_households_total += household.active_households;
        aggregate.terminal_largest_household_size_total +=
            u64::from(household.largest_living_household_size);
        aggregate.terminal_multigenerational_households_total +=
            household.multigenerational_households;
        aggregate.terminal_living_occupied_cells_total += run
            .checkpoint
            .population
            .summary()
            .living_occupied_cell_count;
        for bin in &household.living_household_size_distribution {
            add_bin(
                &mut aggregate.terminal_household_size_distribution,
                bin.living_members,
                bin.household_count,
            );
        }
        for bin in &household.living_household_age_distribution {
            add_bin(
                &mut aggregate.terminal_household_age_days_distribution,
                bin.age_days,
                bin.household_count,
            );
        }
        for bin in &household.living_household_generation_span_distribution {
            add_bin(
                &mut aggregate.terminal_household_generation_span_distribution,
                bin.generations,
                bin.household_count,
            );
        }
        if let Some(condition) = run.checkpoint.population.mean_living_condition_permille() {
            aggregate.mean_living_condition_permille_sum += u64::from(condition);
            aggregate.mean_living_condition_defined_runs += 1;
        }
        aggregate.unmet_need_total += run.manifest.resources.unmet_need;
        aggregate.migration_moves_total += run.manifest.migration.moves_completed;
        aggregate.migration_people_moved_total += run.manifest.migration.people_moved;
        aggregate.temporary_departures_total += temporary.summary.journeys_started;
        aggregate.temporary_visitor_person_days_total += temporary.summary.visitor_person_days;
        aggregate.temporary_visitor_household_days_total +=
            temporary.summary.visitor_household_days;
        aggregate.temporary_peak_visitors_max = aggregate
            .temporary_peak_visitors_max
            .max(temporary.summary.peak_visitors);
    }
    aggregate
}

fn main() {
    let comparison = Comparison {
        schema_version: 2,
        purpose: "TRACE structural sensitivity to founder-defined versus deterministic size-fission household lifecycles",
        scientific_status: "synthetic structural sensitivity; not empirical household validation",
        seeds: SEEDS.to_vec(),
        duration_years: DURATION_YEARS,
        founder_population: 120,
        founder_target_household_size: 5,
        alternative_max_living_members: 8,
        baseline: aggregate(false),
        deterministic_size_fission: aggregate(true),
    };
    println!("{}", serde_json::to_string_pretty(&comparison).unwrap());
}
