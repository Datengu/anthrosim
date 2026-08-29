use std::collections::BTreeMap;

use anthrosim_core::{
    AgeProbabilityBand, DemographyConfig, ExperimentConfig, FounderGenealogyStatus,
    FounderHousehold, FounderPerson, FounderPopulationDefinition, HouseholdLifecycleConfig,
    MigrationConfig, ParameterProvenance, PopulationConfig, ReproductiveSex, ResourceConfig,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    time::SimTime,
};

fn quiet_demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 0;
    }
    config
}

fn certain_fertility() -> DemographyConfig {
    let mut config = quiet_demography();
    config.fertility_bands = vec![
        AgeProbabilityBand::new(0, 18, 0),
        AgeProbabilityBand::new(18, 45, 1_000_000),
        AgeProbabilityBand::new(45, u32::MAX, 0),
    ];
    config
}

fn no_pressure_resources() -> ResourceConfig {
    let mut config = ResourceConfig::synthetic_validation_v1();
    config.annual_need_units_per_person = 0;
    config.max_scarcity_mortality_probability_per_million = 0;
    config
}

fn definition(id: &str, ages_by_id: &[u16]) -> FounderPopulationDefinition {
    let people = ages_by_id
        .iter()
        .enumerate()
        .map(|(index, &age)| FounderPerson {
            id: PersonId::new(index as u64 + 1),
            birth_day: -(i64::from(age) * 365),
            reproductive_sex: if age % 2 == 0 {
                ReproductiveSex::Female
            } else {
                ReproductiveSex::Male
            },
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        })
        .collect();
    FounderPopulationDefinition::new(
        id,
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        people,
    )
}

fn run_declared(
    definition: FounderPopulationDefinition,
    demography: DemographyConfig,
    max: u16,
) -> anthrosim_core::RecordedRun {
    let n = definition.people.len() as u32;
    let config = ExperimentConfig::new(32401, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(PopulationConfig::new(n))
        .with_founder_population(definition)
        .with_demography(demography)
        .with_resources(no_pressure_resources())
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
        .with_household_lifecycle(
            HouseholdLifecycleConfig::deterministic_dependency_fission_v2(max, 18),
        );
    Simulation::new(config).unwrap().run_recorded().unwrap()
}

fn household_age_composition(run: &anthrosim_core::RecordedRun) -> Vec<Vec<u64>> {
    let population = &run.checkpoint.population;
    let time = SimTime::from_days(run.checkpoint.time.days());
    let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
    for raw in 1..=population.person_count() as u64 {
        let person = population.person(PersonId::new(raw)).unwrap();
        if person.is_alive() {
            groups
                .entry(person.household.0)
                .or_default()
                .push(person.age_days_at(time).unwrap());
        }
    }
    let mut result = groups.into_values().collect::<Vec<_>>();
    for ages in &mut result {
        ages.sort_unstable();
    }
    result.sort();
    result
}

#[test]
fn consistent_person_id_relabelling_preserves_unlabelled_age_composition() {
    let ages = [60, 50, 40, 30, 20, 15, 10, 5];
    let mut reversed = ages;
    reversed.reverse();
    let a = run_declared(definition("relabel-a", &ages), quiet_demography(), 4);
    let b = run_declared(definition("relabel-b", &reversed), quiet_demography(), 4);
    assert_eq!(household_age_composition(&a), household_age_composition(&b));
}

#[test]
fn threshold_crossing_birth_boundary_cannot_create_newborn_only_household() {
    let mut founders = Vec::new();
    for (index, age) in [40_u16, 38, 35, 32, 30].into_iter().enumerate() {
        founders.push(FounderPerson {
            id: PersonId::new(index as u64 + 1),
            birth_day: -(i64::from(age) * 365),
            reproductive_sex: if index < 4 {
                ReproductiveSex::Female
            } else {
                ReproductiveSex::Male
            },
            household: HouseholdId::new(1),
            female_parent: None,
            male_parent: None,
            last_birth_day: None,
            condition_permille: 1_000,
        });
    }
    let definition = FounderPopulationDefinition::new(
        "birth-boundary",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(1),
        }],
        founders,
    );
    let run = run_declared(definition, certain_fertility(), 5);
    assert_eq!(run.checkpoint.population.living_count(), 9);
    let composition = household_age_composition(&run);
    assert_eq!(composition.len(), 2);
    let independent_age_days = 18 * 365;
    assert!(
        composition
            .iter()
            .all(|ages| ages.iter().any(|&age| age >= independent_age_days))
    );
    assert!(
        composition
            .iter()
            .all(|ages| !ages.iter().all(|&age| age == 0))
    );
}

#[test]
fn insufficient_independent_anchors_defer_target_ceiling_instead_of_orphaning_dependents() {
    let ages = [30, 10, 9, 8, 7, 6, 5, 4, 3];
    let run = run_declared(definition("one-anchor", &ages), quiet_demography(), 5);
    assert_eq!(run.checkpoint.population.household_count(), 1);
    assert_eq!(
        household_age_composition(&run)
            .iter()
            .map(Vec::len)
            .max(),
        Some(9)
    );
}
