use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule,
    TemporaryTravelModel, TemporaryTriggerTiming, WorldConfig, config::PROBABILITY_PER_MILLION,
    ids::{HouseholdId, PersonId},
};

fn forced_fertility_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = PROBABILITY_PER_MILLION;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 0;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn base_config() -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    ExperimentConfig::new(95_001, 1)
        .with_world(WorldConfig::new(16, 16))
        .with_population(PopulationConfig::new(200).with_target_household_size(5))
        .with_demography(forced_fertility_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1())
}

fn configured_away_run(base: ExperimentConfig) -> ExperimentConfig {
    let probe = Simulation::new(base.clone()).unwrap();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(anthrosim_core::ids::CellId::new)
        .find(|cell| {
            !(1..=probe.population().household_count() as u64).any(|raw| {
                probe.population().household_location(HouseholdId::new(raw)) == Some(*cell)
            })
        })
        .expect("world must have an unoccupied destination");
    let definition = TemporaryMobilityConfig::new(
        FocalRegion::new(
            "audit-v5-area-n-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap(),
        TemporaryMobilitySchedule::new(
            "audit-v5-area-n-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![365],
            400,
        )
        .unwrap(),
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap();
    base.with_temporary_mobility(definition)
}

type BirthSignature = (PersonId, PersonId, PersonId, HouseholdId);

fn birth_signature(run: &anthrosim_core::RecordedRun) -> Vec<BirthSignature> {
    run.events()
        .events
        .iter()
        .filter_map(|record| match record.event {
            EventKind::Birth {
                person,
                female_parent,
                male_parent,
                household,
                ..
            } => Some((person, female_parent, male_parent, household)),
            _ => None,
        })
        .collect()
}

fn main() {
    let base = base_config();
    let baseline = Simulation::new(base.clone()).unwrap().run_recorded().unwrap();
    let away = Simulation::new(configured_away_run(base))
        .unwrap()
        .run_recorded()
        .unwrap();

    let baseline_births = birth_signature(&baseline);
    let away_births = birth_signature(&away);
    println!("baseline_births={}", baseline_births.len());
    println!("away_births={}", away_births.len());
    println!("birth_signatures_equal={}", baseline_births == away_births);
    println!(
        "baseline_m4_households_evaluated={}",
        baseline.manifest.migration.households_evaluated
    );
    println!(
        "away_m4_households_evaluated={}",
        away.manifest.migration.households_evaluated
    );

    // M9 physical presence is intentionally separate from persistent M2 residence/locality.
    // Same-day M9 should reduce M4 eligibility while leaving residence-based M2 outcomes invariant.
    assert!(!baseline_births.is_empty());
    assert_eq!(baseline_births, away_births);
    assert!(
        baseline.manifest.migration.households_evaluated
            > away.manifest.migration.households_evaluated
    );
    away.validate_invariants().unwrap();
}
