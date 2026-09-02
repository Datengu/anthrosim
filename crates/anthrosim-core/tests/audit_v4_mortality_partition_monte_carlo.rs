use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig,
};

const POPULATION: u32 = 30_000;
const ANNUAL_MORTALITY_PPM: u32 = 200_000;

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = ANNUAL_MORTALITY_PPM;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn resources(periods_per_year: u16) -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = periods_per_year;
    config.annual_need_units_per_person = 0;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn deaths(periods_per_year: u16) -> u64 {
    let config = ExperimentConfig::new(84_221, 1)
        .with_world(WorldConfig::new(16, 16))
        .with_population(
            PopulationConfig::new(POPULATION)
                .with_target_household_size(5)
                .with_max_person_records(u64::from(POPULATION) + 10),
        )
        .with_demography(demography())
        .with_resources(resources(periods_per_year))
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    run.manifest.population.deaths_since_start
}

fn sigma_for_binomial(n: f64, p: f64) -> f64 {
    (n * p * (1.0 - p)).sqrt()
}

#[test]
fn annual_background_mortality_is_statistically_stable_across_m3_partitions() {
    let expected = f64::from(POPULATION) * f64::from(ANNUAL_MORTALITY_PPM) / 1_000_000.0;
    let sigma = sigma_for_binomial(f64::from(POPULATION), 0.2);
    let tolerance = 6.0 * sigma + 2.0;

    let mut observations = Vec::new();
    for periods in [1_u16, 12, 365] {
        let observed = deaths(periods);
        let deviation = (observed as f64 - expected).abs();
        println!(
            "periods_per_year={periods} observed_deaths={observed} expected={expected:.3} deviation={deviation:.3} sigma={sigma:.3} z={:.3}",
            deviation / sigma
        );
        assert!(
            deviation <= tolerance,
            "annual mortality materially departs from configured 0.2 risk at {periods} M3 periods/year: observed={observed}, expected={expected:.3}, tolerance={tolerance:.3}"
        );
        observations.push((periods, observed));
    }

    let combined_sigma = (2.0_f64).sqrt() * sigma;
    let pair_tolerance = 6.0 * combined_sigma + 2.0;
    for left in 0..observations.len() {
        for right in (left + 1)..observations.len() {
            let (left_periods, left_deaths) = observations[left];
            let (right_periods, right_deaths) = observations[right];
            let difference = left_deaths.abs_diff(right_deaths) as f64;
            println!(
                "partition_pair={left_periods}v{right_periods} absolute_death_difference={difference:.3} combined_sigma={combined_sigma:.3} z={:.3}",
                difference / combined_sigma
            );
            assert!(
                difference <= pair_tolerance,
                "M3 partition count changes one-year mortality beyond a conservative six-sigma Monte Carlo envelope: {left_periods} periods -> {left_deaths}, {right_periods} periods -> {right_deaths}, tolerance={pair_tolerance:.3}"
            );
        }
    }
}
