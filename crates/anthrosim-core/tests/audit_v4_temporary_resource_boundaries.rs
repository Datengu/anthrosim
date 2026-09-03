use anthrosim_core::ids::{CellId, HouseholdId};
use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,
    ParameterProvenance, PopulationConfig, ResourceConfig, Simulation, TemporaryMobilityConfig,
    TemporaryMobilitySchedule, TemporaryTravelModel, TemporaryTravelResolution,
    TemporaryTriggerTiming, WorldConfig,
};

const HORIZON_DAYS: u64 = 365;

fn no_event_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn base_config(periods_per_year: u16) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1()
        .with_annual_need_units_per_person(365)
        .with_initial_stock_units_per_productivity(1_000)
        .with_annual_regeneration_units_per_productivity(1_000);
    resources.periods_per_year = periods_per_year;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(96_500, 1)
        .with_world(WorldConfig::new(4, 4))
        .with_population(PopulationConfig::new(1).with_target_household_size(1))
        .with_demography(no_event_demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn mobility(
    base: &ExperimentConfig,
    trigger_day: u64,
    stay_duration_days: u32,
) -> TemporaryMobilityConfig {
    let probe = Simulation::new(base.clone()).unwrap();
    let household = HouseholdId::new(1);
    let residence = probe.population().household_location(household).unwrap();
    let destination = (1..=probe.world().cell_count() as u64)
        .map(CellId::new)
        .find(|candidate| *candidate != residence)
        .unwrap();
    let region = FocalRegion::new(
        format!("audit-v4-temporary-resource-{trigger_day}-{stay_duration_days}"),
        FocalRegionSource::Synthetic,
        vec![destination],
    )
    .unwrap();
    let travel_model = TemporaryTravelModel::new(
        format!("audit-v4-fast-travel-{trigger_day}-{stay_duration_days}"),
        ParameterProvenance::SyntheticValidation,
        1_000_000,
        u16::MAX,
    )
    .unwrap();
    let config = TemporaryMobilityConfig::new(
        region,
        TemporaryMobilitySchedule::new(
            format!("audit-v4-departure-{trigger_day}-{stay_duration_days}"),
            TemporaryTriggerTiming::DepartureDay,
            vec![trigger_day],
            stay_duration_days,
        )
        .unwrap(),
        travel_model,
    )
    .unwrap();

    let program = config.derive_program(probe.world()).unwrap();
    match program.travel.resolution(residence).unwrap() {
        TemporaryTravelResolution::Reachable {
            destination: observed,
            outbound_travel_days,
            return_travel_days,
        } => {
            assert_eq!(observed, destination);
            assert_eq!(outbound_travel_days, 1);
            assert_eq!(return_travel_days, 1);
        }
        TemporaryTravelResolution::Unreachable => panic!("fast-travel fixture is unreachable"),
    }
    config
}

fn expected_visiting_days(trigger_day: u64, stay_duration_days: u32) -> u64 {
    let arrival = trigger_day + 1;
    if arrival >= HORIZON_DAYS {
        return 0;
    }
    let visit_end = arrival.saturating_add(u64::from(stay_duration_days));
    visit_end.min(HORIZON_DAYS) - arrival
}

#[test]
fn temporary_resource_attribution_is_partition_and_boundary_invariant() {
    let periods = [1_u16, 4, 12, 365];
    let trigger_days = [
        0_u64, 1, 89, 90, 91, 92, 181, 182, 183, 272, 273, 274, 363, 364,
    ];
    let stay_durations = [1_u32, 2, 5];
    let mut cases = 0_u64;

    for periods_per_year in periods {
        for trigger_day in trigger_days {
            for stay_duration_days in stay_durations {
                let base = base_config(periods_per_year);
                let temporary = mobility(&base, trigger_day, stay_duration_days);
                let checkpoint = Simulation::new(base.with_temporary_mobility(temporary))
                    .unwrap()
                    .checkpoint_at_year(1)
                    .unwrap();
                checkpoint.validate_invariants().unwrap();

                let observations = checkpoint.resources.period_observations();
                assert_eq!(
                    observations.len(),
                    usize::from(periods_per_year),
                    "period count mismatch: periods={periods_per_year} trigger={trigger_day} stay={stay_duration_days}"
                );

                let mut observed_days = 0_u64;
                let mut home_need = 0_u64;
                let mut visitor_need = 0_u64;
                let mut total_need = 0_u64;
                for observation in observations {
                    let duration = observation.end_day - observation.start_day;
                    observed_days += duration;
                    home_need += observation.home_need;
                    visitor_need += observation.visitor_need;
                    total_need += observation.total_need;
                    assert_eq!(
                        observation.home_need + observation.visitor_need,
                        observation.total_need,
                        "period attribution mismatch: periods={periods_per_year} trigger={trigger_day} stay={stay_duration_days} observation={observation:?}"
                    );
                    assert_eq!(
                        observation.total_need,
                        duration,
                        "one-person 365-unit/year demand should be exactly one unit/day: periods={periods_per_year} trigger={trigger_day} stay={stay_duration_days} observation={observation:?}"
                    );
                }

                let expected_visitor = expected_visiting_days(trigger_day, stay_duration_days);
                assert_eq!(observed_days, HORIZON_DAYS);
                assert_eq!(total_need, HORIZON_DAYS);
                assert_eq!(
                    visitor_need, expected_visitor,
                    "visitor attribution changed at a journey/resource boundary: periods={periods_per_year} trigger={trigger_day} stay={stay_duration_days}"
                );
                assert_eq!(
                    home_need,
                    HORIZON_DAYS - expected_visitor,
                    "residence+transit provisioning changed at a journey/resource boundary: periods={periods_per_year} trigger={trigger_day} stay={stay_duration_days}"
                );
                cases += 1;
            }
        }
    }

    println!("audit_v4_temporary_resource_boundary_cases={cases}");
    assert_eq!(cases, 4 * 14 * 3);
}
