use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

fn demography() -> DemographyConfig {
    let mut config = DemographyConfig::synthetic_validation_v1();
    for band in &mut config.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut config.fertility_bands {
        band.annual_probability_per_million = 500_000;
    }
    config.minimum_birth_spacing_days = 0;
    config.male_parent_min_age_years = 18;
    config.male_parent_max_age_years_exclusive = 70;
    config
}

fn founder(
    id: u64,
    age_years: i64,
    reproductive_sex: ReproductiveSex,
    household: u64,
) -> FounderPerson {
    FounderPerson {
        id: PersonId::new(id),
        birth_day: -(age_years * 365),
        reproductive_sex,
        household: HouseholdId::new(household),
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn focal_only() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-b-focal-only",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(2),
        }],
        vec![
            founder(1, 30, ReproductiveSex::Female, 1),
            founder(2, 30, ReproductiveSex::Male, 1),
        ],
    )
}

fn with_remote_pair() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v5-area-b-with-remote-pair",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![
            FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(2),
            },
            FounderHousehold {
                id: HouseholdId::new(2),
                location: CellId::new(1),
            },
        ],
        vec![
            // Focal pair is byte-for-byte scientifically unchanged from the focal-only arm.
            founder(1, 30, ReproductiveSex::Female, 1),
            founder(2, 30, ReproductiveSex::Male, 1),
            // Older remote pair is non-colocated with the focal pair. Its female sorts earlier in
            // the persisted stochastic-coupling order while remaining in a separate parentage cell.
            founder(3, 40, ReproductiveSex::Female, 2),
            founder(4, 40, ReproductiveSex::Male, 2),
        ],
    )
}

fn focal_birth(seed: u64, founders: FounderPopulationDefinition, population: u32) -> bool {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(population)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(16),
        )
        .with_founder_population(founders)
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .events()
        .events
        .iter()
        .any(|record| {
            matches!(
                record.event,
                EventKind::Birth {
                    female_parent,
                    cell,
                    ..
                } if female_parent == PersonId::new(1) && cell == CellId::new(2)
            )
        })
}

#[test]
fn isolated_remote_fertility_candidate_does_not_change_focal_bernoulli_realization() {
    let mut divergences = Vec::new();
    for seed in 0..1_024_u64 {
        let baseline = focal_birth(seed, focal_only(), 2);
        let augmented = focal_birth(seed, with_remote_pair(), 4);
        if baseline != augmented {
            divergences.push((seed, baseline, augmented));
        }
    }

    println!(
        "focal fertility divergences after adding separate-cell remote reproductive pair: {}/1024; first={:?}",
        divergences.len(),
        divergences.iter().take(12).collect::<Vec<_>>()
    );

    assert!(
        divergences.is_empty(),
        "the unchanged focal female's fertility Bernoulli realization acquired a nonlocal dependency on a separate-cell remote reproductive pair: {}/1024 seeds diverged; first={:?}",
        divergences.len(),
        divergences.iter().take(12).collect::<Vec<_>>()
    );
}
