use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, MigrationConfig, PopulationConfig,
    ResourceConfig, Simulation, SimulationCheckpoint, WorldConfig,
};

fn birth_rich_checkpoint() -> SimulationCheckpoint {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    demography.minimum_birth_spacing_days = 0;
    demography.male_parent_min_age_years = 0;
    demography.male_parent_max_age_years_exclusive = 100;

    Simulation::new(
        ExperimentConfig::new(88, 2)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(200).with_max_person_records(10_000))
            .with_demography(demography)
            .with_resources(
                ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
            )
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .expect("birth-rich simulation")
    .checkpoint_at_year(2)
    .expect("two-year checkpoint")
}

fn death_rich_checkpoint() -> SimulationCheckpoint {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }

    Simulation::new(
        ExperimentConfig::new(89, 2)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(80).with_max_person_records(1_000))
            .with_demography(demography)
            .with_resources(
                ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
            )
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false)),
    )
    .expect("death-rich simulation")
    .run_recorded()
    .expect("terminal run")
    .checkpoint
}

fn event_indices(
    checkpoint: &SimulationCheckpoint,
    predicate: impl Fn(&EventKind) -> bool,
) -> Vec<usize> {
    checkpoint
        .events
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, record)| predicate(&record.event).then_some(index))
        .collect()
}

#[test]
fn duplicate_birth_substitution_cannot_replace_a_missing_person_history() {
    let mut checkpoint = birth_rich_checkpoint();
    let indices = event_indices(&checkpoint, |event| {
        matches!(event, EventKind::Birth { .. })
    });
    assert!(indices.len() >= 2, "fixture must contain multiple births");
    let replacement = checkpoint.events.events[indices[0]].event.clone();
    checkpoint.events.events[indices[1]].event = replacement;
    checkpoint = checkpoint.seal_continuation_identity();

    let error = checkpoint.validate_invariants().unwrap_err().to_string();
    assert!(error.contains("duplicate authoritative birth event") || error.contains("bijection"));
}

#[test]
fn duplicate_death_substitution_cannot_replace_a_missing_dead_person_history() {
    let mut checkpoint = death_rich_checkpoint();
    let indices = event_indices(&checkpoint, |event| {
        matches!(event, EventKind::Death { .. })
    });
    assert!(indices.len() >= 2, "fixture must contain multiple deaths");
    let replacement = checkpoint.events.events[indices[0]].event.clone();
    checkpoint.events.events[indices[1]].event = replacement;
    checkpoint = checkpoint.seal_continuation_identity();

    let error = checkpoint.validate_invariants().unwrap_err().to_string();
    assert!(error.contains("duplicate authoritative death event") || error.contains("bijection"));
}

#[test]
fn impossible_intermediate_population_metric_is_rejected_after_integrity_reseal() {
    let mut checkpoint = birth_rich_checkpoint();
    assert!(checkpoint.metrics.snapshots.len() >= 2);
    checkpoint.metrics.snapshots[0].population.living_population = checkpoint.metrics.snapshots[0]
        .population
        .living_population
        .saturating_add(1);
    checkpoint = checkpoint.seal_continuation_identity();

    let error = checkpoint.validate_invariants().unwrap_err().to_string();
    assert!(error.contains("intermediate population metrics"));
}

#[test]
fn impossible_intermediate_resource_accounting_is_rejected_after_integrity_reseal() {
    let mut checkpoint = birth_rich_checkpoint();
    assert!(checkpoint.metrics.snapshots.len() >= 2);
    checkpoint.metrics.snapshots[0].resources.final_food_stock = checkpoint.metrics.snapshots[0]
        .resources
        .final_food_stock
        .saturating_add(1);
    checkpoint = checkpoint.seal_continuation_identity();

    let error = checkpoint.validate_invariants().unwrap_err().to_string();
    assert!(error.contains("intermediate resource metrics"));
}

#[test]
fn impossible_intermediate_migration_totals_are_rejected_after_integrity_reseal() {
    let mut checkpoint = birth_rich_checkpoint();
    assert!(checkpoint.metrics.snapshots.len() >= 2);
    checkpoint.metrics.snapshots[0].migration.moves_completed = 1;
    checkpoint = checkpoint.seal_continuation_identity();

    let error = checkpoint.validate_invariants().unwrap_err().to_string();
    assert!(error.contains("intermediate migration metrics"));
}
