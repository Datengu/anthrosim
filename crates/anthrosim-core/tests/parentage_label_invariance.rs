use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, RngStreamPosition,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MaleRole {
    ExistingFather,
    SecondFather,
    Unrelated,
}

#[derive(Debug, PartialEq, Eq)]
struct ParentageOutcome {
    selected_roles: Vec<MaleRole>,
    newborn_sexes: Vec<ReproductiveSex>,
    demography_rng: [RngStreamPosition; 4],
}

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
    config.male_parent_max_age_years_exclusive = 100;
    config
}

fn founder(
    id: u64,
    age_years: i64,
    sex: ReproductiveSex,
    male_parent: Option<u64>,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(age_years * 365),
        reproductive_sex: sex,
        household: HouseholdId::new(1),
        female_parent: None,
        male_parent: male_parent.map(PersonId::new),
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn two_male_founders(swapped_labels: bool) -> (FounderPopulationDefinition, [(PersonId, MaleRole); 2]) {
    let (existing_father, unrelated) = if swapped_labels { (3, 2) } else { (2, 3) };
    let people = vec![
        founder(1, 30, ReproductiveSex::Female, None),
        founder(2, 35, ReproductiveSex::Male, None),
        founder(3, 35, ReproductiveSex::Male, None),
        founder(4, 10, ReproductiveSex::Male, Some(existing_father)),
    ];
    (
        FounderPopulationDefinition::new(
            "parentage-two-male-relabel",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::Unspecified,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            people,
        ),
        [
            (PersonId::new(existing_father), MaleRole::ExistingFather),
            (PersonId::new(unrelated), MaleRole::Unrelated),
        ],
    )
}

fn three_male_founders(rotation: u8) -> (FounderPopulationDefinition, [(PersonId, MaleRole); 3]) {
    let ids = match rotation {
        0 => [2, 3, 4],
        1 => [3, 4, 2],
        2 => [4, 2, 3],
        _ => panic!("rotation must be 0..=2"),
    };
    let existing_father = ids[0];
    let second_father = ids[1];
    let unrelated = ids[2];
    let people = vec![
        founder(1, 30, ReproductiveSex::Female, None),
        founder(2, 35, ReproductiveSex::Male, None),
        founder(3, 35, ReproductiveSex::Male, None),
        founder(4, 35, ReproductiveSex::Male, None),
        founder(5, 10, ReproductiveSex::Male, Some(existing_father)),
        founder(6, 8, ReproductiveSex::Female, Some(second_father)),
    ];
    (
        FounderPopulationDefinition::new(
            "parentage-three-male-rotation",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::Unspecified,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            people,
        ),
        [
            (PersonId::new(existing_father), MaleRole::ExistingFather),
            (PersonId::new(second_father), MaleRole::SecondFather),
            (PersonId::new(unrelated), MaleRole::Unrelated),
        ],
    )
}

fn run(
    seed: u64,
    years: u64,
    initial_population: u32,
    founders: FounderPopulationDefinition,
    roles: &[(PersonId, MaleRole)],
) -> ParentageOutcome {
    let config = ExperimentConfig::new(seed, years)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(initial_population)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(32),
        )
        .with_founder_population(founders)
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let recorded = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut selected_roles = Vec::new();
    let mut newborn_sexes = Vec::new();
    for event in &recorded.events().events {
        if let EventKind::Birth {
            male_parent,
            reproductive_sex,
            ..
        } = event.event
        {
            let role = roles
                .iter()
                .find_map(|(id, role)| (*id == male_parent).then_some(*role))
                .expect("every selected male parent must be one of the eligible adult role males");
            selected_roles.push(role);
            newborn_sexes.push(reproductive_sex);
        }
    }

    let rng = &recorded.checkpoint.rng;
    ParentageOutcome {
        selected_roles,
        newborn_sexes,
        demography_rng: [
            rng.demography_mortality,
            rng.demography_fertility,
            rng.demography_parentage,
            rng.demography_newborn_sex,
        ],
    }
}

fn run_two_males(seed: u64, years: u64, swapped_labels: bool) -> ParentageOutcome {
    let (founders, roles) = two_male_founders(swapped_labels);
    run(seed, years, 4, founders, &roles)
}

fn run_three_males(seed: u64, rotation: u8) -> ParentageOutcome {
    let (founders, roles) = three_male_founders(rotation);
    run(seed, 1, 6, founders, &roles)
}

#[test]
fn original_parentage_relabel_sweep_is_invariant() {
    for seed in 1..=1_000 {
        let a = run_two_males(seed, 1, false);
        let b = run_two_males(seed, 1, true);
        assert_eq!(a.selected_roles.len(), 1, "forced fertility must produce one birth at seed {seed}");
        assert_eq!(b.selected_roles.len(), 1, "forced fertility must produce one relabelled birth at seed {seed}");
        assert_eq!(
            a, b,
            "parentage role, newborn sex, or demography RNG positions changed under pure eligible-male PersonId relabelling at seed {seed}"
        );
    }
}

#[test]
fn three_male_cyclic_relabelling_preserves_scientific_parent_role() {
    for seed in 1..=256 {
        let base = run_three_males(seed, 0);
        assert_eq!(base.selected_roles.len(), 1);
        for rotation in 1..=2 {
            assert_eq!(
                base,
                run_three_males(seed, rotation),
                "three-male parentage assignment changed under cyclic PersonId relabelling at seed {seed}, rotation {rotation}"
            );
        }
    }
}

#[test]
fn parentage_relabel_invariance_propagates_through_two_demographic_years() {
    for seed in 1..=256 {
        let a = run_two_males(seed, 2, false);
        let b = run_two_males(seed, 2, true);
        assert_eq!(a.selected_roles.len(), 2, "forced fertility must produce two births at seed {seed}");
        assert_eq!(b.selected_roles.len(), 2, "forced fertility must produce two relabelled births at seed {seed}");
        assert_eq!(
            a, b,
            "two-year genealogy role sequence or demography RNG positions changed under pure eligible-male PersonId relabelling at seed {seed}"
        );
    }
}
