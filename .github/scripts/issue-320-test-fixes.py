from pathlib import Path

p = Path('crates/anthrosim-core/src/founder_initialization.rs')
s = p.read_text()

def insert_after_in_test(text: str, test_name: str, needle: str, addition: str) -> str:
    start = text.index(f'fn {test_name}()')
    next_test = text.find('\n    #[test]', start + 1)
    end = len(text) if next_test < 0 else next_test
    body = text[start:end]
    assert needle in body, (test_name, needle)
    body = body.replace(needle, needle + addition, 1)
    return text[:start] + body + text[end:]

s = insert_after_in_test(
    s,
    'prior_birth_reproductive_age_support_uses_fertility_schedule_boundaries',
    '        let mut definition = valid_definition();\n',
    '        // Isolate prior-birth history from the separate parent-age rule.\n'
    '        definition.people[2].female_parent = None;\n'
    '        definition.people[2].male_parent = None;\n',
)
s = insert_after_in_test(
    s,
    'founder_reproductive_history_tracks_custom_fertility_support',
    '        let mut definition = valid_definition();\n',
    '        // Isolate the mother-age rule from the child\'s unrelated prior-birth history.\n'
    '        definition.people[2].last_birth_day = None;\n',
)
p.write_text(s)

p = Path('crates/anthrosim-core/src/simulation.rs')
s = p.read_text()
old_import = '''        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
        },'''
new_import = '''        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
            FounderPopulationError,
        },'''
assert old_import in s
s = s.replace(old_import, new_import, 1)
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
s = s.replace(marker, checkpoint_test + marker, 1)
p.write_text(s)
