from pathlib import Path

p = Path('crates/anthrosim-core/src/founder_initialization.rs')
s = p.read_text()
needle = '''        let mut definition = valid_definition();
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = -80 * year;

        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year - 1);'''
replacement = '''        let mut definition = valid_definition();
        let year = DAYS_PER_YEAR as i64;
        // Isolate the founder's own prior-birth history from the separate parent-age rule.
        definition.people[2].female_parent = None;
        definition.people[2].male_parent = None;
        definition.people[0].birth_day = -80 * year;

        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year - 1);'''
assert needle in s
s = s.replace(needle, replacement)
needle = '''        let mut definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = child_birth - 20 * year;'''
replacement = '''        let mut definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;
        // This test varies the mother's configured reproductive-age support only; the child's
        // unrelated prior-birth history would otherwise be outside the deliberately narrow band.
        definition.people[2].last_birth_day = None;
        definition.people[0].birth_day = child_birth - 20 * year;'''
assert needle in s
s = s.replace(needle, replacement)
p.write_text(s)

p = Path('crates/anthrosim-core/src/simulation.rs')
s = p.read_text()
s = s.replace(
'''        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
        },''',
'''        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
            FounderPopulationError,
        },''')
marker = '''    #[test]
    fn malformed_evidence_is_rejected_by_core_construction() {'''
assert marker in s
checkpoint_test = r'''    #[test]
    fn checkpoint_resume_rejects_impossible_embedded_founder_parent_age_after_reseal() {
        let definition = FounderPopulationDefinition::new(
            "checkpoint-founder-age-test-v1",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![FounderHousehold {
                id: HouseholdId::new(1),
                location: CellId::new(1),
            }],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -(25 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -(30 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(1),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: PersonId::new(3),
                    birth_day: -(5 * DAYS_PER_YEAR as i64),
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: Some(PersonId::new(1)),
                    male_parent: Some(PersonId::new(2)),
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
            ],
        );
        let mut demography = DemographyConfig::synthetic_validation_v1();
        for band in &mut demography.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        // Retain positive fertility support so the declared reproductive-age envelope remains
        // explicit while making this test independent of mortality.
        let config = ExperimentConfig::new(58, 2)
            .with_world(WorldConfig::new(1, 1))
            .with_population(
                PopulationConfig::new(3)
                    .with_initialization(PopulationInitialization::DeclaredFounderStateV1),
            )
            .with_founder_population(definition)
            .with_demography(demography)
            .with_resources(no_pressure_resources())
            .with_migration(disabled_migration());

        let mut checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let founder = checkpoint.experiment.founder_population.as_mut().unwrap();
        let child_birth = founder.people[2].birth_day;
        founder.people[0].birth_day = child_birth - 1;
        checkpoint = checkpoint.seal_continuation_identity();

        assert!(matches!(
            Simulation::from_checkpoint(checkpoint),
            Err(SimulationError::Population(PopulationError::FounderPopulation(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Female,
                    age_days: 1,
                    ..
                }
            )))
        ));
    }

'''
s = s.replace(marker, checkpoint_test + marker)
p.write_text(s)
