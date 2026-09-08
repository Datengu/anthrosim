use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig, Simulation,
    TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
    TemporaryTriggerTiming, WorldConfig, derive_temporary_mobility_observability,
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

fn temporary_mobility() -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "audit-v5-area-f-destination",
        FocalRegionSource::Synthetic,
        vec![CellId::new(2)],
    )
    .unwrap();
    let schedule = TemporaryMobilitySchedule::new(
        "audit-v5-area-f-single-window",
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
            "audit-v5-area-f-split-founders"
        } else {
            "audit-v5-area-f-unified-founders"
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        households,
        people,
    )
}

fn experiment(split: bool) -> ExperimentConfig {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.seasonality_scale_permille = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(60_001, 1)
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

fn report(split: bool) -> anthrosim_core::TemporaryMobilityObservabilityReport {
    let simulation = Simulation::new(experiment(split)).unwrap();
    let world = simulation.world().clone();
    let initial_population = simulation.population().clone();
    let run = simulation.run_recorded().unwrap();
    derive_temporary_mobility_observability(&world, &initial_population, &run.checkpoint).unwrap()
}

#[test]
fn person_level_aggregation_exposure_is_invariant_to_household_partition_when_all_people_follow_the_same_visit() {
    let unified = report(false);
    let split = report(true);

    assert_eq!(unified.summary.journeys_started, 1);
    assert_eq!(split.summary.journeys_started, 4);
    assert_eq!(unified.summary.visitor_household_days, 30);
    assert_eq!(split.summary.visitor_household_days, 120);

    assert_eq!(unified.summary.visitor_person_days, 120);
    assert_eq!(split.summary.visitor_person_days, 120);
    assert_eq!(unified.summary.peak_visitors, 4);
    assert_eq!(split.summary.peak_visitors, 4);
    assert_eq!(
        unified.summary.mean_visitors_millipersons,
        split.summary.mean_visitors_millipersons
    );
    assert_eq!(
        unified.summary.total_living_person_days,
        split.summary.total_living_person_days
    );
    assert_eq!(
        unified.summary.persistent_residence_person_days,
        split.summary.persistent_residence_person_days
    );
    assert_eq!(
        unified.summary.at_residence_person_days,
        split.summary.at_residence_person_days
    );
    assert_eq!(
        unified.summary.outbound_transit_person_days,
        split.summary.outbound_transit_person_days
    );
    assert_eq!(
        unified.summary.return_transit_person_days,
        split.summary.return_transit_person_days
    );

    let unified_destination = unified.cells.iter().find(|row| row.cell == CellId::new(2)).unwrap();
    let split_destination = split.cells.iter().find(|row| row.cell == CellId::new(2)).unwrap();
    assert_eq!(unified_destination.visitor_person_days, 120);
    assert_eq!(split_destination.visitor_person_days, 120);
    assert_eq!(unified_destination.peak_visitors, 4);
    assert_eq!(split_destination.peak_visitors, 4);
    assert_eq!(unified_destination.visitor_household_days, 30);
    assert_eq!(split_destination.visitor_household_days, 120);

    eprintln!(
        "unified=(journeys={}, visitor_person_days={}, visitor_household_days={}, peak={}); split=(journeys={}, visitor_person_days={}, visitor_household_days={}, peak={})",
        unified.summary.journeys_started,
        unified.summary.visitor_person_days,
        unified.summary.visitor_household_days,
        unified.summary.peak_visitors,
        split.summary.journeys_started,
        split.summary.visitor_person_days,
        split.summary.visitor_household_days,
        split.summary.peak_visitors,
    );
}
