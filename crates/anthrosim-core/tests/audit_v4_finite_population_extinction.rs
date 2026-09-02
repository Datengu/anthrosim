use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation,
    StopReason, WorldConfig,
};

const YEARS: u64 = 120;
const REPLICATES: u64 = 200;

fn run(seed: u64, population: u32) -> (bool, u64) {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.condition_recovery_per_period = 0;
    resources.max_condition_loss_per_period = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(seed, YEARS)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(population)
                .with_target_household_size(5)
                .with_max_person_records(100_000),
        )
        .with_demography(DemographyConfig::synthetic_validation_v1())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let result = Simulation::new(config).unwrap().run_recorded().unwrap();
    (
        result.manifest.stop_reason == StopReason::PopulationExtinct,
        result.manifest.population.living_population,
    )
}

fn summarize(population: u32) -> (u64, f64, f64) {
    let mut extinctions = 0_u64;
    let mut all_terminal_sum = 0_u64;
    let mut survivor_terminal_sum = 0_u64;
    let mut survivors = 0_u64;

    for replicate in 0..REPLICATES {
        let seed = 74_000 + replicate;
        let (extinct, terminal) = run(seed, population);
        extinctions += u64::from(extinct);
        all_terminal_sum += terminal;
        if !extinct {
            survivors += 1;
            survivor_terminal_sum += terminal;
        }
    }

    let all_mean = all_terminal_sum as f64 / REPLICATES as f64;
    let survivor_mean = if survivors == 0 {
        0.0
    } else {
        survivor_terminal_sum as f64 / survivors as f64
    };
    (extinctions, all_mean, survivor_mean)
}

#[test]
fn finite_population_extinction_and_survivor_censoring_are_quantified() {
    let (small_extinctions, small_all_mean, small_survivor_mean) = summarize(20);
    let (large_extinctions, large_all_mean, large_survivor_mean) = summarize(200);

    println!(
        "small_n=20 replicates={REPLICATES} years={YEARS} extinctions={small_extinctions} extinction_rate={:.4} all_terminal_mean={small_all_mean:.3} survivor_terminal_mean={small_survivor_mean:.3} censoring_shift={:.3}",
        small_extinctions as f64 / REPLICATES as f64,
        small_survivor_mean - small_all_mean
    );
    println!(
        "large_n=200 replicates={REPLICATES} years={YEARS} extinctions={large_extinctions} extinction_rate={:.4} all_terminal_mean={large_all_mean:.3} survivor_terminal_mean={large_survivor_mean:.3} censoring_shift={:.3}",
        large_extinctions as f64 / REPLICATES as f64,
        large_survivor_mean - large_all_mean
    );

    // This is an audit measurement, not a preordained scientific pass/fail threshold. Basic
    // invariants ensure the experiment itself produced interpretable finite-population evidence.
    assert!(small_extinctions <= REPLICATES);
    assert!(large_extinctions <= REPLICATES);
    assert!(small_all_mean.is_finite() && small_survivor_mean.is_finite());
    assert!(large_all_mean.is_finite() && large_survivor_mean.is_finite());
}
