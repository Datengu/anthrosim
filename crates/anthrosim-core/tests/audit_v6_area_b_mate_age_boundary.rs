use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const FEMALE: PersonId = PersonId::new(1);
const MALE: PersonId = PersonId::new(2);

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founder(
    id: PersonId,
    age_years: i64,
    reproductive_sex: ReproductiveSex,
) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(age_years * 365),
        reproductive_sex,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn founders(male_age_years: Option<i64>) -> FounderPopulationDefinition {
    let mut people = vec![founder(FEMALE, 30, ReproductiveSex::Female)];
    if let Some(age_years) = male_age_years {
        people.push(founder(MALE, age_years, ReproductiveSex::Male));
    }
    FounderPopulationDefinition::new(
        match male_age_years {
            None => "audit-v6-area-b-no-male",
            Some(17) => "audit-v6-area-b-male-age-17",
            Some(18) => "audit-v6-area-b-male-age-18",
            Some(_) => "audit-v6-area-b-other-male-age",
        },
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        people,
    )
}

fn birth_days(seed: u64, male_age_years: Option<i64>) -> Vec<u64> {
    let population_size = if male_age_years.is_some() { 2 } else { 1 };
    let config = ExperimentConfig::new(seed, 2)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(population_size)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders(male_age_years))
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut days = Vec::new();
    for record in &run.events().events {
        if let EventKind::Birth {
            female_parent,
            male_parent,
            ..
        } = record.event
        {
            assert_eq!(female_parent, FEMALE, "only the founder female can reproduce");
            assert_eq!(male_parent, MALE, "the sole eligible male must be the father");
            days.push(record.day);
        }
    }
    days
}

#[test]
fn mate_limitation_uses_male_age_at_elapsed_interval_start() {
    for seed in 0..64_u64 {
        let no_male = birth_days(seed, None);
        assert!(
            no_male.is_empty(),
            "certain fertility must not create births without any male; seed={seed}, births={no_male:?}"
        );

        let crossing_male = birth_days(seed, Some(17));
        assert_eq!(
            crossing_male,
            vec![730],
            "a male aged 17 at day 0 must not enable the [0,365) fertility exposure merely because he turns 18 during it, but must enable [365,730); seed={seed}"
        );

        let eligible_male = birth_days(seed, Some(18));
        assert_eq!(
            eligible_male,
            vec![365, 730],
            "an otherwise identical male already aged 18 at day 0 must enable both annual fertility exposures; seed={seed}"
        );
    }
}
