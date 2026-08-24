from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if new in text:
        return
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


path = "crates/anthrosim-core/src/temporary_mobility.rs"

replace_once(
    path,
    '''                    if let Some(program) = &self.program {\n                        let expected_model_identity =\n                            program.travel.travel_model().map(|model| model.identity());\n                        let expected_cost = program.travel.accumulated_cost_units(active.residence);\n                        if active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !program.region.contains(active.destination)\n                            || active.travel_model_identity != expected_model_identity\n                            || active.accumulated_travel_cost_units != expected_cost\n                        {''',
    '''                    if let Some(program) = &self.program {\n                        let expected_model_identity =\n                            program.travel.travel_model().map(|model| model.identity());\n                        let expected_cost = program.travel.accumulated_cost_units(active.residence);\n                        let resolution_matches = matches!(\n                            program.travel.resolution(active.residence),\n                            Some(TemporaryTravelResolution::Reachable {\n                                destination,\n                                outbound_travel_days,\n                                return_travel_days,\n                            }) if destination == active.destination\n                                && outbound_travel_days == active.outbound_travel_days\n                                && return_travel_days == active.return_travel_days\n                        );\n                        if active.region_id != program.region.region_id\n                            || active.region_identity != program.region.identity()\n                            || !resolution_matches\n                            || active.travel_model_identity != expected_model_identity\n                            || active.accumulated_travel_cost_units != expected_cost\n                        {''',
)

replace_once(
    path,
    '''    #[test]\n    fn disabled_state_keeps_legacy_compatibility_boundary() {''',
    '''    #[test]\n    fn active_journey_must_match_program_travel_resolution_exactly() {\n        let (world, population) = fixture(23);\n        let program = program(\n            &world,\n            &population,\n            TemporaryTriggerTiming::DepartureDay,\n            vec![5],\n            2,\n        );\n        let mut state =\n            TemporaryMobilityState::with_program(&population, program, &world).unwrap();\n        let mut events = EventLog::new();\n        state\n            .process_day(5, &population, &world, &mut events)\n            .unwrap();\n\n        let active = state.active_journeys[0]\n            .as_mut()\n            .expect("household 1 should have an active journey");\n        active.outbound_travel_days = active.outbound_travel_days.saturating_add(1);\n        active.arrival_day = active\n            .departure_day\n            .saturating_add(u64::from(active.outbound_travel_days));\n        active.return_departure_day = active.arrival_day.saturating_add(3);\n        active.completion_day = active\n            .return_departure_day\n            .saturating_add(u64::from(active.return_travel_days));\n\n        assert!(matches!(\n            state.validate(&population, &world),\n            Err(TemporaryMobilityValidationError::ActiveJourneyProgramMismatch { household })\n                if household == HouseholdId::new(1)\n        ));\n    }\n\n    #[test]\n    fn disabled_state_keeps_legacy_compatibility_boundary() {''',
)

print("M9.4 exact travel-resolution invariant patch applied")
