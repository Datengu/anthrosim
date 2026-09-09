use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, ReproductiveSex, ResourceConfig, Simulation,
    WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
};

const FOCAL_FEMALE: PersonId = PersonId::new(1);
const FOCAL_MALE_YOUNGER: PersonId = PersonId::new(2);
const FOCAL_MALE_OLDER: PersonId = PersonId::new(3);
const REMOTE_FEMALE: PersonId = PersonId::new(4);
const REMOTE_MALE: PersonId = PersonId::new(5);

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
    household: HouseholdId,
) -> FounderPerson {
    FounderPerson {
        id,
        birth_day: -(age_years * 365),
        reproductive_sex,
        household,
        female_parent: None,
        male_parent: None,
        last_birth_day: None,
        condition_permille: 1_000,
    }
}

fn focal_only() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-b-parentage-focal-only",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::Unspecified,
        vec![FounderHousehold {
            id: HouseholdId::new(1),
            location: CellId::new(2),
        }],
        vec![
            founder(
                FOCAL_FEMALE,
                30,
                ReproductiveSex::Female,
                HouseholdId::new(1),
            ),
            founder(
                FOCAL_MALE_YOUNGER,
                35,
                ReproductiveSex::Male,
                HouseholdId::new(1),
            ),
            founder(
                FOCAL_MALE_OLDER,
                45,
                ReproductiveSex::Male,
                HouseholdId::new(1),
            ),
        ],
    )
}

fn with_remote_deterministic_birth() -> FounderPopulationDefinition {
    FounderPopulationDefinition::new(
        "audit-v6-area-b-parentage-with-remote",
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
            // The complete focal household is byte-for-byte unchanged from the baseline arm.
            founder(
                FOCAL_FEMALE,
                30,
                ReproductiveSex::Female,
                HouseholdId::new(1),
            ),
            founder(
                FOCAL_MALE_YOUNGER,
                35,
                ReproductiveSex::Male,
                HouseholdId::new(1),
            ),
            founder(
                FOCAL_MALE_OLDER,
                45,
                ReproductiveSex::Male,
                HouseholdId::new(1),
            ),
            // As in the v5 fertility-locality control, the older remote female sorts before the
            // focal female in persisted demographic coupling order. Her separate-cell household
            // has exactly one eligible male, so its male-parent result is scientifically
            // deterministic even though the reservoir sampler currently consumes parentage RNG.
            founder(
                REMOTE_FEMALE,
                50,
                ReproductiveSex::Female,
                HouseholdId::new(2),
            ),
            founder(REMOTE_MALE, 50, ReproductiveSex::Male, HouseholdId::new(2)),
        ],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParentageResult {
    focal_male_parent: PersonId,
    remote_birth_precedes_focal: Option<bool>,
}

fn parentage_result(
    seed: u64,
    founders: FounderPopulationDefinition,
    population_size: u32,
    include_remote: bool,
) -> ParentageResult {
    let config = ExperimentConfig::new(seed, 1)
        .with_world(WorldConfig::new(2, 1))
        .with_population(
            PopulationConfig::new(population_size)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(16),
        )
        .with_founder_population(founders)
        .with_demography(demography())
        .with_resources(
            ResourceConfig::synthetic_validation_v1().with_annual_need_units_per_person(0),
        )
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false));

    let run = Simulation::new(config).unwrap().run_recorded().unwrap();
    let mut focal = None;
    let mut remote = None;
    for record in &run.events().events {
        if let EventKind::Birth {
            female_parent,
            male_parent,
            cell,
            ..
        } = record.event
        {
            if female_parent == FOCAL_FEMALE && cell == CellId::new(2) {
                focal = Some((record.sequence, male_parent));
            }
            if include_remote && female_parent == REMOTE_FEMALE && cell == CellId::new(1) {
                assert_eq!(
                    male_parent, REMOTE_MALE,
                    "remote one-male household must have deterministic paternity"
                );
                remote = Some(record.sequence);
            }
        }
    }

    let (focal_sequence, focal_male_parent) =
        focal.expect("certain focal fertility must produce one day-365 birth");
    assert!(
        matches!(focal_male_parent, FOCAL_MALE_YOUNGER | FOCAL_MALE_OLDER),
        "focal father must come from the unchanged two-male local choice set"
    );

    ParentageResult {
        focal_male_parent,
        remote_birth_precedes_focal: if include_remote {
            Some(
                remote.expect("certain remote fertility must produce one day-365 birth")
                    < focal_sequence,
            )
        } else {
            None
        },
    }
}

#[test]
fn deterministic_remote_parentage_does_not_shift_focal_father_selection() {
    let mut divergences = Vec::new();
    for seed in 0..1_024_u64 {
        let baseline = parentage_result(seed, focal_only(), 3, false);
        let augmented = parentage_result(seed, with_remote_deterministic_birth(), 5, true);

        assert_eq!(augmented.remote_birth_precedes_focal, Some(true));
        if baseline.focal_male_parent != augmented.focal_male_parent {
            divergences.push((
                seed,
                baseline.focal_male_parent,
                augmented.focal_male_parent,
            ));
        }
    }

    println!(
        "focal paternity divergences after adding separate-cell deterministic remote parentage: {}/1024; first={:?}",
        divergences.len(),
        divergences.iter().take(12).collect::<Vec<_>>()
    );

    assert!(
        divergences.is_empty(),
        "the unchanged focal two-male parentage choice acquired a nonlocal dependency on a separate-cell birth whose own father was deterministic: {}/1024 seeds diverged; first={:?}",
        divergences.len(),
        divergences.iter().take(12).collect::<Vec<_>>()
    );
}
