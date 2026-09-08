use anthrosim_core::{ExperimentConfig, PopulationConfig, Simulation, WorldConfig};

fn config(duration_years: u64) -> ExperimentConfig {
    ExperimentConfig::new(65_001, duration_years)
        .with_world(WorldConfig::new(8, 8))
        .with_population(
            PopulationConfig::new(100)
                .with_target_household_size(5)
                .with_max_person_records(2_000),
        )
}

#[test]
fn declared_future_horizon_does_not_change_the_common_prefix_trajectory() {
    // Arm A declares a two-year experiment and reaches its terminal boundary normally.
    let short = Simulation::new(config(2)).unwrap().run_recorded().unwrap();
    short.validate_invariants().unwrap();

    // Arm B declares one extra future year but is serialized at the same day-730 boundary.
    let long_prefix = Simulation::new(config(3))
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    long_prefix.validate_invariants().unwrap();

    // The stop horizon is preserved experiment provenance and therefore intentionally differs.
    assert_ne!(short.checkpoint.experiment.duration_years, long_prefix.experiment.duration_years);
    assert_eq!(short.checkpoint.time.days(), 730);
    assert_eq!(long_prefix.time.days(), 730);

    // Merely declaring an additional future stop boundary must not feed backward into the causal
    // trajectory before the common day-730 prefix ends.
    assert_eq!(short.checkpoint.state_digest64, long_prefix.state_digest64);
    assert_eq!(short.checkpoint.population, long_prefix.population);
    assert_eq!(short.checkpoint.resources, long_prefix.resources);
    assert_eq!(short.checkpoint.migration, long_prefix.migration);
    assert_eq!(short.checkpoint.temporary_mobility, long_prefix.temporary_mobility);
    assert_eq!(short.checkpoint.events, long_prefix.events);
    assert_eq!(short.checkpoint.metrics, long_prefix.metrics);

    eprintln!(
        "common_prefix_day={}; short_horizon={}; long_horizon={}; state={:016x}; people={}; events={}; metrics={}",
        730,
        short.checkpoint.experiment.duration_years,
        long_prefix.experiment.duration_years,
        short.checkpoint.state_digest64,
        short.checkpoint.population.person_count(),
        short.checkpoint.events.events.len(),
        short.checkpoint.metrics.snapshots.len(),
    );
}
