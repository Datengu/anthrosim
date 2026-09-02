use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, MigrationConfig,
    ParameterProvenance, PopulationConfig, PopulationInitialization, ReproductiveSex,
    ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn quiet_resources() -> ResourceConfig {
    ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0)
}

fn female(id: u64, age_days: i64, last_birth_day: Option<i64>) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -age_days,
        reproductive_sex: ReproductiveSex::Female,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day,
        condition_permille: 1_000,
    }
}

fn male(id: u64, age_days: i64) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -age_days,
        reproductive_sex: ReproductiveSex::Male,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(female: FounderPerson) -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v4-demography-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![female, male(2, 30 * 365)],
    )
}

fn run_births(female: FounderPerson, demography: DemographyConfig) -> u64 {
    let config = ExperimentConfig::new(64_002, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
        )
        .with_founder_population(founders(female))
        .with_demography(demography)
        .with_resources(quiet_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .manifest
        .population
        .births_since_start
}

fn narrow_age_band_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.schedule_id = "audit-v4-narrow-fertility-band".to_owned();
    config.mortality_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 20, 0),
        AgeProbabilityBand::new(20, 21, 1_000_000),
        AgeProbabilityBand::new(21, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn spacing_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.schedule_id = "audit-v4-spacing-boundary".to_owned();
    config.mortality_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 50, 1_000_000),
        AgeProbabilityBand::new(50, u32::MAX, 0),
    ];
    config.minimum_birth_spacing_days = 1_000;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 100;
    config
}

#[test]
fn fertility_age_band_edges_use_exact_interval_start_age() {
    let demography = narrow_age_band_demography();
    let cases = [
        (20 * 365 - 1, 0_u64, "one day before age-20 lower edge"),
        (20 * 365, 1, "exact age-20 lower edge"),
        (21 * 365 - 1, 1, "one day before age-21 upper edge"),
        (21 * 365, 0, "exact age-21 upper edge"),
    ];

    for (age_days, expected_births, label) in cases {
        let observed = run_births(female(1, age_days, None), demography.clone());
        println!("case={label} age_days={age_days} births={observed}");
        assert_eq!(
            observed, expected_births,
            "fertility band boundary mismatch at {label}"
        );
    }
}

#[test]
fn rounded_executable_birth_spacing_has_correct_exact_threshold() {
    // Requested 1000 days is not executable subannually: annual M2 scheduling rounds the effective
    // lower bound up to 3 * 365 = 1095 days. At the first day-365 M2 boundary, lastBirthDay=-729
    // is 1094 elapsed days and must suppress; -730 is exactly 1095 and must permit.
    let demography = spacing_demography();
    let cases = [
        (-729_i64, 0_u64, 1_094_u64),
        (-730_i64, 1_u64, 1_095_u64),
        (-731_i64, 1_u64, 1_096_u64),
    ];

    for (last_birth_day, expected_births, elapsed_at_boundary) in cases {
        let observed = run_births(
            female(1, 30 * 365, Some(last_birth_day)),
            demography.clone(),
        );
        println!(
            "last_birth_day={last_birth_day} elapsed_at_day365={elapsed_at_boundary} births={observed}"
        );
        assert_eq!(
            observed, expected_births,
            "birth-spacing threshold mismatch at elapsed={elapsed_at_boundary} days"
        );
    }
}
