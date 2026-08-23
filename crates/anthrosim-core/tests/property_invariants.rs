use anthrosim_core::{
    ExperimentConfig, MigrationConfig, PopulationConfig, ResourceConfig, Simulation, StopReason,
    WorldConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct GeneratedCase {
    seed: u64,
    width: u32,
    height: u32,
    population: u32,
    household_size: u16,
    productivity_scale: u16,
    seasonality_scale: u16,
    migration_enabled: bool,
    migration_radius: u16,
}

fn generated_cases() -> Vec<GeneratedCase> {
    let mut cases = Vec::new();
    for seed in [41_001, 41_002] {
        for (width, height) in [(2, 2), (4, 3)] {
            for population in [8, 17] {
                for household_size in [1, 5] {
                    for (productivity_scale, seasonality_scale) in [(250, 0), (1_000, 1_000)] {
                        for (migration_enabled, migration_radius) in [(false, 1), (true, 1)] {
                            cases.push(GeneratedCase {
                                seed,
                                width,
                                height,
                                population,
                                household_size,
                                productivity_scale,
                                seasonality_scale,
                                migration_enabled,
                                migration_radius,
                            });
                        }
                    }
                }
            }
        }
    }
    cases.sort_unstable();
    cases
}

fn config_for(case: GeneratedCase, years: u64) -> ExperimentConfig {
    let resources = ResourceConfig::synthetic_validation_v1()
        .with_productivity_scale_permille(case.productivity_scale)
        .with_seasonality_scale_permille(case.seasonality_scale);
    let migration = MigrationConfig::synthetic_validation_v1()
        .with_enabled(case.migration_enabled)
        .with_candidate_radius_cells(case.migration_radius);

    ExperimentConfig::new(case.seed, years)
        .with_world(WorldConfig::new(case.width, case.height))
        .with_population(
            PopulationConfig::new(case.population)
                .with_target_household_size(case.household_size)
                .with_max_person_records(5_000),
        )
        .with_resources(resources)
        .with_migration(migration)
}

#[test]
fn generated_valid_configs_preserve_cross_system_invariants_and_determinism() {
    // This is a deliberately bounded generated state-space rather than a random
    // fuzz test. Cases are sorted from smaller to larger values, so the first
    // reported failure is a deterministic, reproducible minimal tuple within
    // this declared domain.
    for case in generated_cases() {
        let config = config_for(case, 3);

        let first = Simulation::new(config.clone())
            .unwrap_or_else(|error| panic!("generated case {case:?} was rejected: {error}"))
            .run_recorded()
            .unwrap_or_else(|error| panic!("generated case {case:?} failed to run: {error}"));
        first
            .validate_invariants()
            .unwrap_or_else(|error| panic!("generated case {case:?} violated invariants: {error}"));

        let second = Simulation::new(config)
            .unwrap_or_else(|error| panic!("generated replay case {case:?} was rejected: {error}"))
            .run_recorded()
            .unwrap_or_else(|error| panic!("generated replay case {case:?} failed: {error}"));
        second.validate_invariants().unwrap_or_else(|error| {
            panic!("generated replay case {case:?} violated invariants: {error}")
        });

        assert_eq!(
            first.manifest, second.manifest,
            "same-seed generated case diverged: {case:?}"
        );
        assert_eq!(
            first.checkpoint, second.checkpoint,
            "same-seed checkpoint diverged: {case:?}"
        );
        assert!(matches!(
            first.manifest.stop_reason,
            StopReason::DurationReached
                | StopReason::PopulationExtinct
                | StopReason::PersonRecordLimitReached
        ));
    }
}

#[test]
fn generated_checkpoint_resume_matches_uninterrupted_execution() {
    for case in generated_cases().into_iter().step_by(8) {
        let config = config_for(case, 4);
        let uninterrupted = Simulation::new(config.clone())
            .unwrap_or_else(|error| panic!("generated case {case:?} was rejected: {error}"))
            .run_recorded()
            .unwrap_or_else(|error| panic!("generated case {case:?} failed to run: {error}"));
        uninterrupted.validate_invariants().unwrap_or_else(|error| {
            panic!("generated uninterrupted case {case:?} violated invariants: {error}")
        });

        // A population can legitimately terminate before year two. Only assert
        // resume equivalence when the requested annual checkpoint exists.
        let Ok(checkpoint) = Simulation::new(config).unwrap().checkpoint_at_year(2) else {
            continue;
        };
        checkpoint.validate_invariants().unwrap_or_else(|error| {
            panic!("generated checkpoint case {case:?} violated invariants: {error}")
        });
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap_or_else(|error| panic!("generated checkpoint case {case:?} rejected: {error}"))
            .run_recorded()
            .unwrap_or_else(|error| panic!("generated resumed case {case:?} failed: {error}"));
        resumed.validate_invariants().unwrap_or_else(|error| {
            panic!("generated resumed case {case:?} violated invariants: {error}")
        });

        assert_eq!(
            resumed.manifest, uninterrupted.manifest,
            "generated checkpoint/resume manifest mismatch: {case:?}"
        );
        assert_eq!(
            resumed.checkpoint, uninterrupted.checkpoint,
            "generated checkpoint/resume state mismatch: {case:?}"
        );
    }
}

#[test]
fn generated_invalid_resource_configs_are_rejected_without_normalization() {
    for invalid_scale in [1_001_u16, 1_250, u16::MAX] {
        let mut resources = ResourceConfig::synthetic_validation_v1();
        resources.productivity_scale_permille = invalid_scale;
        let config = ExperimentConfig::new(41_900 + u64::from(invalid_scale), 1)
            .with_world(WorldConfig::new(2, 2))
            .with_population(PopulationConfig::new(4))
            .with_resources(resources);

        assert!(
            Simulation::new(config).is_err(),
            "invalid productivity scale was silently accepted: {invalid_scale}"
        );
    }

    for invalid_scale in [1_001_u16, 1_250, u16::MAX] {
        let mut resources = ResourceConfig::synthetic_validation_v1();
        resources.seasonality_scale_permille = invalid_scale;
        let config = ExperimentConfig::new(42_900 + u64::from(invalid_scale), 1)
            .with_world(WorldConfig::new(2, 2))
            .with_population(PopulationConfig::new(4))
            .with_resources(resources);

        assert!(
            Simulation::new(config).is_err(),
            "invalid seasonality scale was silently accepted: {invalid_scale}"
        );
    }
}
