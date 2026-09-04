from pathlib import Path

path = Path("crates/anthrosim-core/src/temporary_mobility.rs")
text = path.read_text()


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one target, found {count}")
    text = text.replace(old, new, 1)


replace_once(
    '''    pub residence: CellId,\n    pub destination: CellId,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub travel_model_identity: Option<String>,''',
    '''    pub residence: CellId,\n    pub destination: CellId,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub destination_tie_coupling_key: Option<u64>,\n    #[serde(default, skip_serializing_if = "Option::is_none")]\n    pub travel_model_identity: Option<String>,''',
    "active journey tie-key field",
)
replace_once(
    '''        digest_u64(hash, self.residence.0);\n        digest_u64(hash, self.destination.0);\n        match &self.travel_model_identity {''',
    '''        digest_u64(hash, self.residence.0);\n        digest_u64(hash, self.destination.0);\n        match self.destination_tie_coupling_key {\n            None => digest_u64(hash, 0),\n            Some(coupling_key) => {\n                digest_u64(hash, 1);\n                digest_u64(hash, coupling_key);\n            }\n        }\n        match &self.travel_model_identity {''',
    "active journey digest",
)
replace_once(
    '''impl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 4;''',
    '''impl TemporaryMobilityState {\n    pub const CURRENT_SCHEMA_VERSION: u32 = 5;''',
    "temporary mobility state schema",
)
replace_once(
    '''                        let resolution_matches = matches!(\n                            program.travel.resolution(active.residence),\n                            Some(TemporaryTravelResolution::Reachable {\n                                destination,\n                                outbound_travel_days,\n                                return_travel_days,\n                            }) if destination == active.destination\n                                && outbound_travel_days == active.outbound_travel_days\n                                && return_travel_days == active.return_travel_days\n                        );\n                        if active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !resolution_matches\n                            || active.travel_model_identity != expected_model_identity\n                            || active.accumulated_travel_cost_units != expected_cost''',
    '''                        let tied_origin = program\n                            .travel\n                            .equal_cost_destination_count(active.residence)\n                            .is_some_and(|count| count > 1);\n                        let resolution = match active.destination_tie_coupling_key {\n                            Some(coupling_key) if tied_origin => program.travel.resolution_for_coupling_key(\n                                active.residence,\n                                coupling_key,\n                                active.trigger_index.unwrap_or(0),\n                            ),\n                            None if !tied_origin => program.travel.resolution(active.residence),\n                            _ => None,\n                        };\n                        let resolution_matches = matches!(\n                            resolution,\n                            Some(TemporaryTravelResolution::Reachable {\n                                destination,\n                                outbound_travel_days,\n                                return_travel_days,\n                            }) if destination == active.destination\n                                && outbound_travel_days == active.outbound_travel_days\n                                && return_travel_days == active.return_travel_days\n                        );\n                        if active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !resolution_matches\n                            || active.travel_model_identity != expected_model_identity\n                            || active.accumulated_travel_cost_units != expected_cost''',
    "active journey program validation",
)
replace_once(
    '''            trigger_day,\n            residence,\n            destination,\n            travel_model_identity: travel_model_identity.clone(),''',
    '''            trigger_day,\n            residence,\n            destination,\n            destination_tie_coupling_key: program\n                .travel\n                .equal_cost_destination_count(residence)\n                .is_some_and(|count| count > 1)\n                .then_some(destination_tie_coupling_key),\n            travel_model_identity: travel_model_identity.clone(),''',
    "active journey construction",
)
replace_once(
    '''                residence,\n                destination,\n                destination_tie_coupling_key: program\n                    .travel\n                    .equal_cost_destination_count(residence)\n                    .is_some_and(|count| count > 1)\n                    .then_some(destination_tie_coupling_key),\n                travel_model_identity,''',
    '''                residence,\n                destination,\n                destination_tie_coupling_key: active.destination_tie_coupling_key,\n                travel_model_identity,''',
    "departure event tie-key reuse",
)
replace_once(
    '''        residence: population.household_location(household).unwrap(),\n        destination,\n        travel_model_identity: None,''',
    '''        residence: population.household_location(household).unwrap(),\n        destination,\n        destination_tie_coupling_key: None,\n        travel_model_identity: None,''',
    "test active journey constructor",
)

old_import = '''    use crate::{\n        config::{PopulationConfig, WorldConfig},\n        focal_region::FocalRegionSource,\n        rng::RngFactory,\n    };'''
new_import = '''    use crate::{\n        config::{\n            DemographyConfig, ParameterProvenance, PopulationConfig, PopulationInitialization,\n            WorldConfig,\n        },\n        focal_region::FocalRegionSource,\n        founder_initialization::{\n            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,\n        },\n        population::ReproductiveSex,\n        rng::RngFactory,\n    };'''
replace_once(old_import, new_import, "test imports")

marker = '''    #[test]\n    fn disabled_state_keeps_legacy_compatibility_boundary() {'''
if text.count(marker) != 1:
    raise SystemExit(f"authoritative regression marker: expected one target, found {text.count(marker)}")

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
