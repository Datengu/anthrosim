use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, Simulation,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const DAYS_PER_YEAR: i64 = 365;

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn founders(split: bool) -> FounderPopulationDefinition {
    let households = if split {
        (1..=4)
            .map(|raw| FounderHousehold {
                id: HouseholdId::new(raw),
                location: CellId::new(1),
            })
            .collect()
    } else {
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }]
    };

    let people = (1..=4)
        .map(|raw| FounderPerson {
            id: PersonId::new(raw),
            birth_day: -((25 + raw as i64) * DAYS_PER_YEAR),
            reproductive_sex: if raw % 2 == 0 {
                ReproductiveSex::Male
            } else {
                ReproductiveSex::Female
            },
            household: if split {
                HouseholdId::new(raw)
            } else {
                HouseholdId::new(1)
            },
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        })
        .collect();

    FounderPopulationDefinition::new(
        if split {
            "audit-v5-area-f-resource-split-founders"
        } else {
            "audit-v5-area-f-resource-unified-founders"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
    )
}

fn temporary_mobility() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v5-area-f-resource-destination",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v5-area-f-resource-window",
        TemporaryTriggerTiming::DepartureDay,
        vec![100],
        30,
    )
    .unwrap();
    TemporaryMobilityConfig::new(
        region,
        schedule,
        TemporaryTravelModel::synthetic_validation_v1(),
    )
    .unwrap()
}

fn experiment(split: bool) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 365;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;
    resources.max_condition_loss_per_period = 0;
    resources.condition_recovery_per_period = 0;

    ExperimentConfig::new(62_001, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(4)
                .with_target_household_size(if split { 1 } else { 4 })
                .with_max_person_records(20),
        )
        .with_founder_population(founders(split))
        .with_demography(demography())
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_temporary_mobility(temporary_mobility())
}

#[derive(Debug)]
struct Totals {
    total_need: u64,
    home_need: u64,
    visitor_need: u64,
    periods: u64,
}

fn run(split: bool) -> Totals {
    let recorded = Simulation::new(experiment(split)).unwrap().run_recorded().unwrap();
    recorded.validate_invariants().unwrap();
    let observations = recorded.checkpoint.resources.period_observations();
    assert_eq!(observations.len(), 1);
    let period = &observations[0];
    assert_eq!(period.start_day, 0);
    assert_eq!(period.end_day, 365);

    Totals {
        total_need: period.total_need,
        home_need: period.home_need,
        visitor_need: period.visitor_need,
        periods: recorded.checkpoint.resources.summary(&recorded.checkpoint.population).periods_processed,
    }
}

#[test]
fn cell_level_visitor_resource_demand_is_invariant_to_household_partition_when_people_and_visit_are_identical() {
    let unified = run(false);
    let split = run(true);

    // Four people × 365 annual units with one exact 365-day resource period.
    assert_eq!(unified.total_need, 1_460);
    assert_eq!(split.total_need, 1_460);

    // Every person visits for exactly 30 days. With exactly divisible annual need this must be
    // 4 × 30 = 120 visitor units regardless of whether the four people are carried by one
    // household claim or four independent household claims.
    assert_eq!(unified.visitor_need, 120);
    assert_eq!(split.visitor_need, 120);
    assert_eq!(unified.home_need, 1_340);
    assert_eq!(split.home_need, 1_340);
    assert_eq!(unified.home_need + unified.visitor_need, unified.total_need);
    assert_eq!(split.home_need + split.visitor_need, split.total_need);
    assert_eq!(unified.periods, 1);
    assert_eq!(split.periods, 1);

    eprintln!("unified={unified:?}; split={split:?}");
}
