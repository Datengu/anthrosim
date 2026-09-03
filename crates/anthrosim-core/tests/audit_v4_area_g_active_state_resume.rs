use std::collections::BTreeSet;

use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig,
    ids::{CellId, HouseholdId},
};

fn stochastic_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 20_000;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 100_000;
    }
    config
}

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 12;
    config.annual_need_units_per_person = 365;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn base_config(seed: u64) -> ExperimentConfig {
    ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(5, 5))
        .with_population(
            PopulationConfig::new(40)
                .with_target_household_size(4)
                .with_max_person_records(400),
        )
        .with_demography(stochastic_demography())
        .with_resources(resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn configured(seed: u64) -> ExperimentConfig {
    let base = base_config(seed);
    let probe = Simulation::new(base.clone()).expect("probe simulation");
    let occupied = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| probe.population().household_location(HouseholdId::new(raw)))
        .collect::<BTreeSet<_>>();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|cell| !occupied.contains(cell))
        .expect("fixture needs one unoccupied destination");
    let temporary = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "audit-v4-area-g-resume-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .expect("focal region"),
        TemporaryMobilitySchedule::new(
            "audit-v4-area-g-resume-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![300],
            180,
        )
        .expect("temporary schedule"),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .expect("temporary mobility config");
    base.with_temporary_mobility(temporary)
}

#[test]
fn active_m9_resource_and_demography_state_resumes_exactly_across_process_seeds() {
    for seed in 1..=32 {
        let config = configured(seed);
        let uninterrupted = Simulation::new(config.clone())
            .expect("uninterrupted simulation")
            .run_recorded()
            .expect("uninterrupted run");

        let paused = Simulation::new(config)
            .expect("checkpoint source")
            .checkpoint_at_year(1)
            .expect("annual checkpoint");
        assert!(
            (1..=paused.population.household_count() as u64).any(|raw| paused
                .temporary_mobility
                .active_journey(HouseholdId::new(raw))
                .is_some()),
            "seed {seed}: annual checkpoint must preserve at least one active M9 journey"
        );
        assert!(
            !paused.resources.period_observations().is_empty(),
            "seed {seed}: checkpoint must preserve nontrivial M3 resource history"
        );

        let resumed = Simulation::from_checkpoint(paused)
            .expect("resume active scientific state")
            .run_recorded()
            .expect("resumed run");

        let expected = &uninterrupted.checkpoint;
        let actual = &resumed.checkpoint;
        assert_eq!(actual.state_digest64, expected.state_digest64, "seed {seed}");
        assert_eq!(
            actual.continuation_digest64, expected.continuation_digest64,
            "seed {seed}: continuation identity drifted"
        );
        assert_eq!(actual.population, expected.population, "seed {seed}");
        assert_eq!(
            actual.temporary_mobility, expected.temporary_mobility,
            "seed {seed}"
        );
        assert_eq!(actual.resources, expected.resources, "seed {seed}");
        assert_eq!(actual.migration, expected.migration, "seed {seed}");
        assert_eq!(actual.rng, expected.rng, "seed {seed}");
        assert_eq!(actual.events, expected.events, "seed {seed}");
        assert_eq!(actual.metrics, expected.metrics, "seed {seed}");
    }
}
