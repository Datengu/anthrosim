use anthrosim_core::{
    AgeProbabilityBand, DeathCause, DemographyConfig, EventKind, ExperimentConfig,
    FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
    MigrationConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,
    ReproductiveSex, ResourceConfig, Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const ONE_MILLION: u32 = 1_000_000;

fn demography(kill_age_35_plus: bool) -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    config.mortality_bands = vec![
        AgeProbabilityBand::new(0, 35, 0),
        AgeProbabilityBand::new(
            35,
            u32::MAX,
            if kill_age_35_plus { ONE_MILLION } else { 0 },
        ),
    ];
    config.fertility_bands = vec![AgeProbabilityBand::new(0, u32::MAX, ONE_MILLION)];
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founders() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-b-death-parentage-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: PersonId::new(1),
                birth_day: -(30 * 365),
                reproductive_sex: ReproductiveSex::Female,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(40 * 365),
                reproductive_sex: ReproductiveSex::Male,
                household: HouseholdId::new(1),
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn run(seed: u64, kill_age_35_plus: bool) -> anthrosim_core::RecordedRun {
    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.periods_per_year = 1;
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(8),
        )
        .with_founder_population(founders())
        .with_demography(demography(kill_age_35_plus))
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config).unwrap().run_recorded().unwrap()
}

#[test]
fn male_dying_at_day_365_mortality_boundary_cannot_authorize_same_day_birth() {
    for seed in 0..64_u64 {
        let control = run(seed, false);
        let death_arm = run(seed, true);

        let control_births = control
            .events()
            .events
            .iter()
            .filter(|record| {
                record.day == 365
                    && matches!(
                        record.event,
                        EventKind::Birth {
                            female_parent,
                            male_parent,
                            ..
                        } if female_parent == PersonId::new(1) && male_parent == PersonId::new(2)
                    )
            })
            .count();
        let control_male_deaths = control
            .events()
            .events
            .iter()
            .filter(|record| {
                matches!(
                    record.event,
                    EventKind::Death {
                        person,
                        cause: DeathCause::DemographicMortality,
                        ..
                    } if person == PersonId::new(2)
                )
            })
            .count();

        let death_arm_births = death_arm
            .events()
            .events
            .iter()
            .filter(|record| matches!(record.event, EventKind::Birth { .. }))
            .count();
        let death_arm_male_deaths = death_arm
            .events()
            .events
            .iter()
            .filter(|record| {
                record.day == 365
                    && matches!(
                        record.event,
                        EventKind::Death {
                            person,
                            cause: DeathCause::DemographicMortality,
                            ..
                        } if person == PersonId::new(2)
                    )
            })
            .count();

        assert_eq!(control_male_deaths, 0, "control male died at seed {seed}");
        assert_eq!(control_births, 1, "surviving local male did not authorize the certain day-365 birth at seed {seed}");
        assert_eq!(death_arm_male_deaths, 1, "certain older-male mortality did not occur exactly once at day 365 for seed {seed}");
        assert_eq!(death_arm_births, 0, "male killed at the coincident day-365 mortality boundary remained available to M2 parentage at seed {seed}");

        death_arm.validate_invariants().unwrap();
        control.validate_invariants().unwrap();
    }

    println!("64/64 control seeds: one focal birth and zero male deaths; 64/64 mortality-arm seeds: one day-365 male demographic death and zero births");
}
