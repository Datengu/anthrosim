from pathlib import Path

path = Path("crates/anthrosim-core/src/temporary_mobility.rs")
text = path.read_text()

old_import = '''    use crate::{\n        config::{PopulationConfig, WorldConfig},\n        focal_region::FocalRegionSource,\n        rng::RngFactory,\n    };'''
new_import = '''    use crate::{\n        config::{\n            DemographyConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,\n            WorldConfig,\n        },\n        focal_region::FocalRegionSource,\n        founder_initialization::{\n            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,\n        },\n        population::ReproductiveSex,\n        rng::RngFactory,\n    };'''
if text.count(old_import) != 1:
    raise SystemExit(f"expected one test import target, found {text.count(old_import)}")
text = text.replace(old_import, new_import, 1)

marker = '''    #[test]\n    fn disabled_state_keeps_legacy_compatibility_boundary() {'''
if text.count(marker) != 1:
    raise SystemExit(f"expected one disabled-state marker, found {text.count(marker)}")

block = r'''    fn tied_execution_population(world: &World, swapped_household_labels: bool) -> Population {
        let center = CellId::new(5);
        let role_a_household = if swapped_household_labels {
            HouseholdId::new(2)
        } else {
            HouseholdId::new(1)
        };
        let role_b_household = if swapped_household_labels {
            HouseholdId::new(1)
        } else {
            HouseholdId::new(2)
        };
        let definition = FounderPopulationDefinition::new(
            if swapped_household_labels {
                "m9-authoritative-household-relabel-b"
            } else {
                "m9-authoritative-household-relabel-a"
            },
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::Unspecified,
            vec![
                FounderHousehold {
                    id: HouseholdId::new(1),
                    location: center,
                },
                FounderHousehold {
                    id: HouseholdId::new(2),
                    location: center,
                },
            ],
            vec![
                FounderPerson {
                    id: crate::ids::PersonId::new(1),
                    birth_day: -(30 * 365),
                    reproductive_sex: ReproductiveSex::Female,
                    household: role_a_household,
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
                FounderPerson {
                    id: crate::ids::PersonId::new(2),
                    birth_day: -(40 * 365),
                    reproductive_sex: ReproductiveSex::Male,
                    household: role_b_household,
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 1_000,
                },
            ],
        );
        Population::initialize_declared_founder_state_v1(
            PopulationConfig::new(2)
                .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
                .with_max_person_records(10),
            &definition,
            world,
            &DemographyConfig::synthetic_validation_v1(),
        )
        .unwrap()
    }

    fn authoritative_tied_destinations(
        tie_seed: u64,
        swapped_household_labels: bool,
    ) -> Vec<(u64, CellId)> {
        let world = World::generate(WorldConfig::new(3, 3), RngFactory::new(7_007))
            .unwrap()
            .with_model_field_overlay(Some(&[1_000; 9]), None, None)
            .unwrap();
        let population = tied_execution_population(&world, swapped_household_labels);
        let region = FocalRegion::new(
            "m9-authoritative-household-relabel",
            FocalRegionSource::Synthetic,
            vec![CellId::new(2), CellId::new(8)],
        )
        .unwrap();
        let travel = TemporaryTravelModel::synthetic_validation_v1()
            .derive_table_with_tie_seed(&region, &world, tie_seed)
            .unwrap();
        assert_eq!(travel.equal_cost_destination_count(CellId::new(5)), Some(2));
        let program = TemporaryMobilityProgram::new(
            region,
            TemporaryMobilitySchedule::new(
                "m9-authoritative-household-relabel",
                TemporaryTriggerTiming::DepartureDay,
                vec![0],
                3,
            )
            .unwrap(),
            travel,
            &world,
        )
        .unwrap();
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();
        state
            .process_day(0, &population, &world, &mut events)
            .unwrap();

        let mut outcomes = events
            .events
            .iter()
            .filter_map(|record| match record.event {
                EventKind::TemporaryJourneyDeparted {
                    destination,
                    destination_tie_coupling_key: Some(coupling_key),
                    ..
                } => Some((coupling_key, destination)),
                _ => None,
            })
            .collect::<Vec<_>>();
        outcomes.sort_unstable_by_key(|(coupling_key, _)| *coupling_key);
        assert_eq!(outcomes.len(), 2);
        outcomes
    }

    #[test]
    fn authoritative_tied_destination_is_household_label_invariant() {
        let mut saw_top = false;
        let mut saw_bottom = false;
        for tie_seed in 1..=1_000 {
            let baseline = authoritative_tied_destinations(tie_seed, false);
            let relabelled = authoritative_tied_destinations(tie_seed, true);
            assert_eq!(
                baseline, relabelled,
                "authoritative M9 tie resolution diverged under pure HouseholdId relabelling at seed {tie_seed}"
            );
            for (_, destination) in baseline {
                saw_top |= destination == CellId::new(2);
                saw_bottom |= destination == CellId::new(8);
            }
        }
        assert!(
            saw_top && saw_bottom,
            "authoritative regression did not exercise both tied destinations"
        );
    }

'''
text = text.replace(marker, block + marker, 1)
path.write_text(text)
