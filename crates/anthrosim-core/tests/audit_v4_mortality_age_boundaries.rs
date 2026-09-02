use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn founder(age_days: i64) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v4-mortality-age-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![FounderPerson {
            id: PersonId::new(1),
            birth_day: -age_days,
            reproductive_sex: ReproductiveSex::Male,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        }],
    )
}

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.schedule_id = "audit-v4-mortality-age-band".to_owned();
    config.mortality_bands = vec![
        AgeProbabilityBand::new(0, 20, 0),
        AgeProbabilityBand::new(20, 21, 1_000_000),
        AgeProbabilityBand::new(21, u32::MAX, 0),
    ];
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.periods_per_year = 1;
    config.annual_need_units_per_person = 0;
    config.condition_recovery_per_period = 0;
    config.max_condition_loss_per_period = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn deaths(age_days: i64) -> u64 {
    let config = ExperimentConfig::new(64_003, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(1)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(2),
        )
        .with_founder_population(founder(age_days))
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .population
        .deaths_since_start
}

#[test]
fn background_mortality_age_band_uses_exact_interval_start_age() {
    let cases = [
        (20 * 365 - 1, 0_u64, "one day before age-20 lower edge"),
        (20 * 365, 1_u64, "exact age-20 lower edge"),
        (21 * 365 - 1, 1_u64, "one day before age-21 upper edge"),
        (21 * 365, 0_u64, "exact age-21 upper edge"),
    ];

    for (age_days, expected_deaths, label) in cases {
        let observed = deaths(age_days);
        println!("case={label} interval_start_age_days={age_days} deaths={observed}");
        assert_eq!(
            observed, expected_deaths,
            "background-mortality age-band boundary mismatch at {label}"
        );
    }
}
