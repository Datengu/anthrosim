use crate::{
    checkpoint::state_digest64_with_temporary_mobility,
    config::{DemographyConfig, ExperimentConfig, PopulationConfig, ResourceConfig, WorldConfig},
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    migration::MigrationSystem,
    rng::RngFactory,
    simulation::Simulation,
    temporary_mobility::HouseholdPresence,
    world::World,
};

fn stable_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn stable_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn active_checkpoint(seed: u64, duration_years: u64) -> crate::SimulationCheckpoint {
    let config = ExperimentConfig::new(seed, duration_years)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(20).with_target_household_size(5))
        .with_demography(stable_demography())
        .with_resources(stable_resources());
    let mut checkpoint = Simulation::new(config).unwrap().checkpoint_at_year(0).unwrap();
    let world = World::generate(
        checkpoint.experiment.world,
        RngFactory::new(checkpoint.experiment.seed),
    )
    .unwrap();
    let household = HouseholdId::new(1);
    let residence = checkpoint.population.household_location(household).unwrap();
    let destination = (1..=world.cell_count() as u64)
        .map(CellId::new)
        .find(|&cell| cell != residence)
        .unwrap();
    checkpoint
        .temporary_mobility
        .set_presence(
            household,
            HouseholdPresence::Visiting {
                journey: TemporaryJourneyId::new(1),
                destination,
            },
            &checkpoint.population,
            &world,
        )
        .unwrap();

    let migration = MigrationSystem::from_checkpoint_state(
        &checkpoint.population,
        &world,
        &checkpoint.experiment.migration,
        checkpoint.migration.clone(),
    )
    .unwrap();
    checkpoint.state_digest64 = state_digest64_with_temporary_mobility(
        checkpoint.time.days(),
        world.digest64(),
        checkpoint.population.digest64(),
        checkpoint.resources.digest64(),
        migration.digest64(),
        &checkpoint.temporary_mobility,
    );
    if let Some(snapshot) = checkpoint.metrics.snapshots.last_mut() {
        snapshot.state_digest64 = checkpoint.state_digest64;
    }
    checkpoint
}

#[test]
fn active_presence_round_trips_through_checkpoint_integrity() {
    let checkpoint = active_checkpoint(9_001, 2);
    let source_presence = checkpoint.temporary_mobility.clone();
    let source_digest = checkpoint.state_digest64;

    checkpoint.validate_invariants().unwrap();
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();

    assert_eq!(resumed.temporary_mobility, source_presence);
    assert_eq!(resumed.state_digest64, source_digest);
    resumed.validate_invariants().unwrap();
}

#[test]
fn active_temporary_household_is_excluded_from_m4_without_changing_residence() {
    let seed = 9_002;
    let baseline = Simulation::new(
        ExperimentConfig::new(seed, 1)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(20).with_target_household_size(5))
            .with_demography(stable_demography())
            .with_resources(stable_resources()),
    )
    .unwrap()
    .run_recorded()
    .unwrap();

    let checkpoint = active_checkpoint(seed, 1);
    let household = HouseholdId::new(1);
    let residence = checkpoint.population.household_location(household).unwrap();
    let active = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        baseline.manifest.migration.households_evaluated
            - active.manifest.migration.households_evaluated,
        active.manifest.migration.decision_boundaries
    );
    assert_eq!(
        active.checkpoint.population.household_location(household),
        Some(residence)
    );
    assert_eq!(
        active
            .checkpoint
            .temporary_mobility
            .is_at_residence(household),
        Some(false)
    );
    active.validate_invariants().unwrap();
}
