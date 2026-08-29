use anthrosim_core::ids::{CellId, HouseholdId, PersonId};
use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, WorldConfig,
};

const DAYS_PER_YEAR: i64 = 365;

fn founder_definition(reverse_labels: bool) -> FounderPopulationDefinition {
    let ages_years = [60_i64, 50, 40, 30, 20, 15, 10, 5];
    let ordered_ages: Vec<_> = if reverse_labels {
        ages_years.iter().copied().rev().collect()
    } else {
        ages_years.to_vec()
    };
    let people = ordered_ages
        .into_iter()
        .enumerate()
        .map(|(index, age_years)| FounderPerson {
            id: PersonId::new(index as u64 + 1),
            birth_day: -(age_years * DAYS_PER_YEAR),
            reproductive_sex: ReproductiveSex::Female,
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        })
        .collect();

    FounderPopulationDefinition::new(
        "audit-v2-area-c-fission-relabel",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        people,
    )
}

fn config(reverse_labels: bool) -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    demography.schedule_id = "audit-v2-area-c-no-events".to_owned();
    demography.mortality_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];
    demography.fertility_bands = vec![AgeProbabilityBand::new(0, u32::MAX, 0)];

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(32301, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(8))
        .with_founder_population(founder_definition(reverse_labels))
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_size_fission_v1(4))
}

fn household_birth_days(reverse_labels: bool) -> Vec<Vec<i64>> {
    let run = Simulation::new(config(reverse_labels))
        .unwrap()
        .run_recorded()
        .unwrap();
    let mut by_household = vec![Vec::new(); run.checkpoint.population.household_count()];
    for raw in 1..=8_u64 {
        let person = run
            .checkpoint
            .population
            .person(PersonId::new(raw))
            .unwrap();
        by_household[(person.household.0 - 1) as usize].push(person.birth_day);
    }
    for days in &mut by_household {
        days.sort_unstable();
    }
    by_household
}

#[test]
fn audit_probe_fission_household_composition_depends_on_person_id_relabelling() {
    let forward = household_birth_days(false);
    let reversed = household_birth_days(true);

    assert_eq!(forward.len(), 2);
    assert_eq!(reversed.len(), 2);
    assert_ne!(forward, reversed);

    assert_eq!(
        forward[0],
        vec![
            -(60 * DAYS_PER_YEAR),
            -(50 * DAYS_PER_YEAR),
            -(40 * DAYS_PER_YEAR),
            -(30 * DAYS_PER_YEAR),
        ]
    );
    assert_eq!(
        reversed[0],
        vec![
            -(20 * DAYS_PER_YEAR),
            -(15 * DAYS_PER_YEAR),
            -(10 * DAYS_PER_YEAR),
            -(5 * DAYS_PER_YEAR),
        ]
    );
}
