use anthrosim_core::{
    DemographyConfig, ExperimentConfig, FounderGenealogyStatus, FounderHousehold, FounderPerson,
    FounderPopulationDefinition, MigrationConfig, ParameterProvenance, PopulationConfig,
    ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const DAYS_PER_YEAR: i64 = 365;
const CERTAIN: u32 = 1_000_000;

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-g-declared-founders",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(2),
            },
        ],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(25 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: -(27 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(4),
                birth_day: -(32 * DAYS_PER_YEAR),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(2),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = CERTAIN;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn config(extreme: bool) -> ExperimentConfig {
    let mut population = PopulationConfig::new(4).with_max_person_records(100);
    if extreme {
        population.target_household_size = 99;
        population.synthetic_max_age_years = 1;
        population.synthetic_male_permille = 1_000;
    } else {
        population.target_household_size = 1;
        population.synthetic_max_age_years = 120;
        population.synthetic_male_permille = 0;
    }

    ExperimentConfig::new(64_001, 2)
        .with_world(WorldConfig::new(2, 1))
        .with_population(population)
        .with_founder_population(founders())
        .with_demography(demography())
        .with_resources(resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

#[test]
fn declared_founder_initialization_and_dynamics_ignore_synthetic_only_population_knobs() {
    let baseline_sim = Simulation::new(config(false)).unwrap();
    let extreme_sim = Simulation::new(config(true)).unwrap();

    // The declared founder definition is authoritative: these synthetic-only knobs must not alter
    // the day-zero materialized population in any way.
    assert_eq!(baseline_sim.population(), extreme_sim.population());
    assert_eq!(baseline_sim.world(), extreme_sim.world());

    let baseline = baseline_sim.run_recorded().unwrap();
    let extreme = extreme_sim.run_recorded().unwrap();
    baseline.validate_invariants().unwrap();
    extreme.validate_invariants().unwrap();

    // The ExperimentConfig objects intentionally differ because the dormant knobs are still
    // preserved provenance. Causal state and authoritative dynamic history must nevertheless be
    // identical when declared-founder mode makes those fields scientifically inert.
    assert_ne!(baseline.checkpoint.experiment.population, extreme.checkpoint.experiment.population);
    assert_eq!(baseline.checkpoint.state_digest64, extreme.checkpoint.state_digest64);
    assert_eq!(baseline.checkpoint.population, extreme.checkpoint.population);
    assert_eq!(baseline.checkpoint.resources, extreme.checkpoint.resources);
    assert_eq!(baseline.checkpoint.migration, extreme.checkpoint.migration);
    assert_eq!(baseline.checkpoint.temporary_mobility, extreme.checkpoint.temporary_mobility);
    assert_eq!(baseline.checkpoint.events, extreme.checkpoint.events);
    assert_eq!(baseline.checkpoint.metrics, extreme.checkpoint.metrics);

    let births = baseline
        .checkpoint
        .events
        .events
        .iter()
        .filter(|record| matches!(record.event, anthrosim_core::EventKind::Birth { .. }))
        .count();
    assert!(births > 0, "the comparison must exercise downstream demographic dynamics");

    eprintln!(
        "baseline_knobs=(target=1,max_age=120,male=0); extreme_knobs=(target=99,max_age=1,male=1000); births={births}; final_state={:016x}; people={}",
        baseline.checkpoint.state_digest64,
        baseline.checkpoint.population.person_count(),
    );
}
