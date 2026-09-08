use std::collections::BTreeSet;

use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, HouseholdId,
    MigrationConfig, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityProgram,
    TemporaryMobilitySchedule, TemporaryTravelResolution, TemporaryTravelTable,
    TemporaryTriggerTiming, WorldConfig,
    config::PROBABILITY_PER_MILLION,
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

fn config() -> ExperimentConfig {
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

fn all_households_away_program(config: &ExperimentConfig) -> TemporaryMobilityProgram {
    let probe = Simulation::new(config.clone()).unwrap();
    let residences: BTreeSet<_> = (1..=probe.population().household_count() as u64)
        .filter_map(|raw| probe.population().household_location(HouseholdId::new(raw)))
        .collect();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(anthrosim_core::ids::CellId::new)
        .find(|cell| !residences.contains(cell))
        .expect("world must have an unoccupied destination");
    let region = FocalRegion::new(
        "audit-v5-area-n-region",
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let resolutions = (1..=probe.world().cell_count() as u64)
        .map(|raw| {
            let origin = anthrosim_core::ids::CellId::new(raw);
            if region.contains(origin) {
                TemporaryTravelResolution::Unreachable
            } else {
                TemporaryTravelResolution::Reachable {
                    destination,
                    outbound_travel_days: 0,
                    return_travel_days: 0,
                }
            }
        })
        .collect();
    let travel = TemporaryTravelTable::new(resolutions, &region, probe.world()).unwrap();
    TemporaryMobilityProgram::new(
        region,
        TemporaryMobilitySchedule::new(
            "audit-v5-area-n-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![365],
            400,
        )
        .unwrap(),
        travel,
        probe.world(),
    )
    .unwrap()
}

fn birth_signature(run: &anthrosim_core::RecordedRun) -> Vec<(u64, u64, u64, u64)> {
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
            } => Some((
                person.get(),
                female_parent.get(),
                male_parent.get(),
                household.get(),
            )),
            _ => None,
        })
        .collect()
}

fn main() {
    let base = config();
    let baseline = Simulation::new(base.clone()).unwrap().run_recorded().unwrap();
    let program = all_households_away_program(&base);
    let away = Simulation::new_with_temporary_mobility(base, program)
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
    // The same-day M9 transition should suppress M4 eligibility while leaving residence-based
    // M2 fertility/parentage outcomes invariant. A failure would be a fresh cross-system defect.
    assert!(!baseline_births.is_empty());
    assert_eq!(baseline_births, away_births);
    assert!(baseline.manifest.migration.households_evaluated > away.manifest.migration.households_evaluated);
    away.validate_invariants().unwrap();
}