use anthrosim_core::{
    config::{
        DemographyConfig, ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig,
        WorldConfig,
    },
    simulation::Simulation,
    time::DAYS_PER_YEAR,
};

const NEGATIVE: &str =
    include_str!("../../../research/demography-controls-v1/negative-growth-control.json");
const REPLACEMENT: &str =
    include_str!("../../../research/demography-controls-v1/replacement-control.json");
const POSITIVE: &str =
    include_str!("../../../research/demography-controls-v1/positive-growth-control.json");

fn parse_control(source: &str) -> DemographyConfig {
    serde_json::from_str(source).expect("committed demographic control must deserialize")
}

fn null_demographic_config(
    seed: u64,
    years: u64,
    world_side: u32,
    initial_population: u32,
    demography: DemographyConfig,
) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    // The control benchmark isolates M2. Resource state may evolve, but it cannot create an
    // additional condition-mediated death hazard and M4 cannot relocate households.
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(seed, years)
        .with_world(WorldConfig::new(world_side, world_side))
        .with_population(PopulationConfig::new(initial_population).with_max_person_records(100_000))
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn living_at_year(checkpoint: &anthrosim_core::checkpoint::SimulationCheckpoint, year: u64) -> u64 {
    let day = year * DAYS_PER_YEAR;
    checkpoint
        .metrics
        .snapshots
        .iter()
        .rev()
        .find(|snapshot| snapshot.day == day)
        .unwrap_or_else(|| panic!("missing annual metric snapshot for year {year}"))
        .population
        .living_population
}

fn mean_late_log_growth(control: &str) -> f64 {
    const START_YEAR: u64 = 80;
    const END_YEAR: u64 = 160;
    const SEEDS: [u64; 8] = [101, 211, 307, 401, 503, 601, 701, 809];

    let mut total = 0.0;
    for seed in SEEDS {
        let config = null_demographic_config(seed, END_YEAR, 1, 600, parse_control(control));
        let recorded = Simulation::new(config)
            .expect("control fixture must initialize")
            .run_recorded()
            .expect("control fixture must complete");
        let start = living_at_year(&recorded.checkpoint, START_YEAR) as f64;
        let end = living_at_year(&recorded.checkpoint, END_YEAR) as f64;
        assert!(
            start > 0.0 && end > 0.0,
            "control fixture unexpectedly went extinct"
        );
        total += (end / start).ln() / (END_YEAR - START_YEAR) as f64;
    }
    total / SEEDS.len() as f64
}

#[test]
fn committed_controls_bracket_an_intrinsic_replacement_regime() {
    let negative = mean_late_log_growth(NEGATIVE);
    let replacement = mean_late_log_growth(REPLACEMENT);
    let positive = mean_late_log_growth(POSITIVE);

    assert!(
        negative < -0.002,
        "negative control should decline after founder transients, observed r={negative}"
    );
    assert!(
        replacement.abs() < 0.0025,
        "replacement control should remain close to zero late growth, observed r={replacement}"
    );
    assert!(
        positive > 0.002,
        "positive control should grow after founder transients, observed r={positive}"
    );
    assert!(negative < replacement && replacement < positive);
}

#[test]
fn negative_control_preserves_legacy_schedule_behavior_not_a_retuned_default() {
    let mut legacy = DemographyConfig::synthetic_validation_v1();
    let negative = parse_control(NEGATIVE);
    legacy.schedule_id = negative.schedule_id.clone();
    assert_eq!(legacy, negative);
}

#[test]
fn local_male_availability_is_visible_as_a_structural_fertility_suppressor() {
    const SEEDS: [u64; 4] = [41, 137, 263, 379];
    let mut concentrated_births = 0_u64;
    let mut dispersed_births = 0_u64;

    for seed in SEEDS {
        let concentrated = Simulation::new(null_demographic_config(
            seed,
            50,
            1,
            600,
            parse_control(REPLACEMENT),
        ))
        .expect("concentrated fixture must initialize")
        .run_recorded()
        .expect("concentrated fixture must complete");
        concentrated_births += concentrated
            .checkpoint
            .population
            .summary()
            .births_since_start;

        let dispersed = Simulation::new(null_demographic_config(
            seed,
            50,
            32,
            600,
            parse_control(REPLACEMENT),
        ))
        .expect("dispersed fixture must initialize")
        .run_recorded()
        .expect("dispersed fixture must complete");
        dispersed_births += dispersed.checkpoint.population.summary().births_since_start;
    }

    assert!(
        concentrated_births > dispersed_births,
        "same demographic schedule should expose fewer realized births when local-male opportunities are structurally scarcer: concentrated={concentrated_births}, dispersed={dispersed_births}"
    );
}
