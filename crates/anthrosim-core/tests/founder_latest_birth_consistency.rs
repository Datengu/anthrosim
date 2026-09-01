use anthrosim_core::{
    DemographyConfig, EventKind, ExperimentConfig, FounderGenealogyStatus, FounderHousehold,
    FounderPerson, FounderPopulationDefinition, MigrationConfig, ParameterProvenance,
    PopulationConfig, PopulationInitialization, RecordedRun, ReproductiveSex, ResourceConfig,
    Simulation, WorldConfig,
    ids::{CellId, HouseholdId, PersonId},
    time::DAYS_PER_YEAR,
};

const MOTHER: PersonId = PersonId::new(1);
const KNOWN_CHILD_BIRTH_DAY: i64 = -100;

fn founders(last_birth_day: Option<i64>) -> FounderPopulationDefinition {
    let household = HouseholdId::new(1);
    FounderPopulationDefinition::new(
        "founder-latest-known-birth-v1",
        ParameterProvenance::SyntheticValidation,
        FounderGenealogyStatus::CompleteLivingDirectParents,
        vec![FounderHousehold {
            id: household,
            location: CellId::new(1),
        }],
        vec![
            FounderPerson {
                id: MOTHER,
                birth_day: -(30 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Female,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(2),
                birth_day: -(30 * DAYS_PER_YEAR as i64),
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: None,
                male_parent: None,
                last_birth_day: None,
                condition_permille: 1_000,
            },
            FounderPerson {
                id: PersonId::new(3),
                birth_day: KNOWN_CHILD_BIRTH_DAY,
                reproductive_sex: ReproductiveSex::Male,
                household,
                female_parent: Some(MOTHER),
                male_parent: Some(PersonId::new(2)),
                last_birth_day: None,
                condition_permille: 1_000,
            },
        ],
    )
}

fn config(last_birth_day: Option<i64>) -> ExperimentConfig {
    let mut demography = DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 0;
    }
    for band in &mut demography.fertility_bands {
        if band.annual_probability_per_million > 0 {
            band.annual_probability_per_million = 1_000_000;
        }
    }

    let mut resources = ResourceConfig::synthetic_validation_v1();
    resources.annual_need_units_per_person = 0;
    resources.max_scarcity_mortality_probability_per_million = 0;

    ExperimentConfig::new(74_396, 1)
        .with_world(WorldConfig::new(1, 1))
        .with_population(
            PopulationConfig::new(3)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(20),
        )
        .with_founder_population(founders(last_birth_day))
        .with_demography(demography)
        .with_resources(resources)
        .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
}

fn births(run: &RecordedRun) -> usize {
    run.events()
        .events
        .iter()
        .filter(|record| matches!(record.event, EventKind::Birth { .. }))
        .count()
}

#[test]
fn declared_last_birth_cannot_predate_a_known_later_declared_child() {
    let error = Simulation::new(config(Some(-2_000)))
        .expect_err("stale lastBirthDay older than a known child must fail closed");
    assert!(
        error
            .to_string()
            .contains("predates later explicitly declared child birth day"),
        "unexpected construction error: {error}"
    );
}

#[test]
fn omitted_last_birth_uses_latest_explicit_child_for_first_boundary_spacing() {
    let run = Simulation::new(config(None))
        .expect("explicit child chronology is a valid founder history")
        .run_recorded()
        .unwrap();

    let definition = run
        .checkpoint
        .experiment
        .founder_population
        .as_ref()
        .expect("declared founder definition remains checkpointed");
    assert_eq!(
        definition.last_birth_day(MOTHER),
        Some(KNOWN_CHILD_BIRTH_DAY)
    );
    assert_eq!(
        births(&run),
        0,
        "the known child at day -100 leaves only 465 days before the first annual boundary"
    );
    run.validate_invariants().unwrap();
}

#[test]
fn last_birth_equal_to_latest_explicit_child_remains_valid() {
    let run = Simulation::new(config(Some(KNOWN_CHILD_BIRTH_DAY)))
        .expect("matching explicit reproductive history must remain valid")
        .run_recorded()
        .unwrap();
    assert_eq!(births(&run), 0);
    run.validate_invariants().unwrap();
}

#[test]
fn later_last_birth_may_represent_an_unrepresented_child() {
    let run = Simulation::new(config(Some(-50)))
        .expect("a later lastBirthDay may represent an unrepresented or non-living child")
        .run_recorded()
        .unwrap();

    let definition = run
        .checkpoint
        .experiment
        .founder_population
        .as_ref()
        .unwrap();
    assert_eq!(definition.last_birth_day(MOTHER), Some(-50));
    assert_eq!(births(&run), 0);
    run.validate_invariants().unwrap();
}
