use anthrosim_core::{
    DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
    Simulation, WorldConfig,
};

const REPLICATES: u64 = 8_192;
const ANNUAL_MORTALITY_PER_MILLION: u32 = 500_000;
const CADENCES: [u16; 4] = [1, 4, 12, 365];

fn controlled_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = ANNUAL_MORTALITY_PER_MILLION;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn run_one(seed: u64, periods_per_year: u16) -> bool {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = periods_per_year;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let checkpoint = Simulation::new(
        ExperimentConfig::new(seed, 1)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(1).with_max_person_records(8))
            .with_demography(controlled_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .expect("controlled mortality-cadence simulation")
    .run_recorded()
    .expect("controlled one-year run")
    .checkpoint;

    checkpoint.population.living_count() == 0
}

#[test]
fn annual_background_mortality_distribution_is_stable_across_m3_cadence() {
    let mut counts = Vec::new();
    for periods in CADENCES {
        let deaths = (1..=REPLICATES)
            .filter(|&seed| run_one(seed, periods))
            .count() as u64;
        let estimate_per_million = deaths * 1_000_000 / REPLICATES;
        println!(
            "periods_per_year={periods} deaths={deaths}/{REPLICATES} estimate_per_million={estimate_per_million}"
        );
        counts.push((periods, deaths, estimate_per_million));
    }

    // Each arm should remain close to the declared 0.5 annual risk. At n=8192 and p=0.5,
    // one-binomial standard error is about 0.00552; ±0.03 is >5 standard errors and therefore
    // deliberately fail-safe against ordinary Monte Carlo noise while still detecting a material
    // cadence-induced annual-risk shift.
    let lower = REPLICATES * 47 / 100;
    let upper = REPLICATES * 53 / 100;
    for (periods, deaths, _) in &counts {
        assert!(
            (*deaths >= lower) && (*deaths <= upper),
            "M3 cadence {periods} produced {deaths}/{REPLICATES} deaths, outside the predeclared 47%-53% annual-risk band"
        );
    }

    let min_deaths = counts.iter().map(|(_, deaths, _)| *deaths).min().unwrap();
    let max_deaths = counts.iter().map(|(_, deaths, _)| *deaths).max().unwrap();
    let spread = max_deaths - min_deaths;
    let maximum_allowed_spread = REPLICATES * 4 / 100;
    assert!(
        spread <= maximum_allowed_spread,
        "M3 cadence changed one-year background mortality materially: min={min_deaths}, max={max_deaths}, spread={spread}/{REPLICATES} (>4 percentage points)"
    );
}
