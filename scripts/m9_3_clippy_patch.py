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

# Add compact trigger/execution context types before the day outcome.
replace_once(
    path,
    '''#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub struct TemporaryMobilityDayOutcome {''',
    '''#[derive(Debug, Clone, Copy)]\nstruct TriggerContext {\n    trigger_index: u32,\n    trigger_day: u64,\n    household: HouseholdId,\n    evaluation_day: u64,\n}\n\nstruct TemporaryExecutionContext<'a> {\n    population: &'a Population,\n    world: &'a World,\n    events: &'a mut EventLog,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub struct TemporaryMobilityDayOutcome {''',
)

# Replace evaluate_trigger call with compact contexts.
old = '''                match self.evaluate_trigger(\n                    &program,\n                    trigger_index,\n                    trigger_day,\n                    household,\n                    day,\n                    population,\n                    world,\n                    events,\n                )? {'''
new = '''                let trigger = TriggerContext {\n                    trigger_index,\n                    trigger_day,\n                    household,\n                    evaluation_day: day,\n                };\n                let mut context = TemporaryExecutionContext {\n                    population,\n                    world,\n                    events,\n                };\n                match self.evaluate_trigger(&program, trigger, &mut context)? {'''
replace_once(path, old, new)

# Replace evaluate_trigger signature and local names.
old = '''    fn evaluate_trigger(\n        &mut self,\n        program: &TemporaryMobilityProgram,\n        trigger_index: u32,\n        trigger_day: u64,\n        household: HouseholdId,\n        day: u64,\n        population: &Population,\n        world: &World,\n        events: &mut EventLog,\n    ) -> Result<TriggerEvaluation, TemporaryMobilityExecutionError> {'''
new = '''    fn evaluate_trigger(\n        &mut self,\n        program: &TemporaryMobilityProgram,\n        trigger: TriggerContext,\n        context: &mut TemporaryExecutionContext<'_>,\n    ) -> Result<TriggerEvaluation, TemporaryMobilityExecutionError> {\n        let TriggerContext {\n            trigger_index,\n            trigger_day,\n            household,\n            evaluation_day: day,\n        } = trigger;\n        let population = context.population;\n        let world = context.world;'''
replace_once(path, old, new)

# Redirect skip calls to compact helper.
text = Path(path).read_text()
text = text.replace(
'''            self.record_skip(\n                program,\n                trigger_index,\n                trigger_day,\n                household,\n                day,\n                reason,\n                events,\n            );''',
'''            self.record_skip(program, trigger, reason, context.events);''')
Path(path).write_text(text)

# Event push inside evaluate_trigger uses context.events.
replace_once(
    path,
    '''        events.push_authoritative(\n            day,\n            EventKind::TemporaryJourneyDeparted {''',
    '''        context.events.push_authoritative(\n            day,\n            EventKind::TemporaryJourneyDeparted {''',
)
replace_once(
    path,
    '''            self.arrive(index, day, population, events)?;''',
    '''            self.arrive(index, day, population, context.events)?;''',
)

# Compact record_skip signature.
old = '''    fn record_skip(\n        &mut self,\n        program: &TemporaryMobilityProgram,\n        trigger_index: u32,\n        trigger_day: u64,\n        household: HouseholdId,\n        day: u64,\n        reason: TemporaryJourneyIneligibility,\n        events: &mut EventLog,\n    ) {\n        self.mark_trigger_processed(trigger_index, household);'''
new = '''    fn record_skip(\n        &mut self,\n        program: &TemporaryMobilityProgram,\n        trigger: TriggerContext,\n        reason: TemporaryJourneyIneligibility,\n        events: &mut EventLog,\n    ) {\n        let TriggerContext {\n            trigger_index,\n            trigger_day,\n            household,\n            evaluation_day: day,\n        } = trigger;\n        self.mark_trigger_processed(trigger_index, household);'''
replace_once(path, old, new)

# Compact test helper timing into a tuple.
old = '''fn test_active_journey(\n    household: HouseholdId,\n    journey: TemporaryJourneyId,\n    destination: CellId,\n    population: &Population,\n    departure_day: u64,\n    arrival_day: u64,\n    return_departure_day: u64,\n    completion_day: u64,\n) -> ActiveTemporaryJourney {'''
new = '''fn test_active_journey(\n    household: HouseholdId,\n    journey: TemporaryJourneyId,\n    destination: CellId,\n    population: &Population,\n    timing: (u64, u64, u64, u64),\n) -> ActiveTemporaryJourney {\n    let (departure_day, arrival_day, return_departure_day, completion_day) = timing;'''
replace_once(path, old, new)

# Update the three test helper calls.
text = Path(path).read_text()
text = text.replace(
'''                population,\n                0,\n                999_998,\n                999_999,\n                1_000_000,\n            ))''',
'''                population,\n                (0, 999_998, 999_999, 1_000_000),\n            ))''')
text = text.replace(
'''                population,\n                0,\n                0,\n                999_999,\n                1_000_000,\n            ))''',
'''                population,\n                (0, 0, 999_999, 1_000_000),\n            ))''')
text = text.replace(
'''                population,\n                0,\n                0,\n                1,\n                1_000_000,\n            ))''',
'''                population,\n                (0, 0, 1, 1_000_000),\n            ))''')
Path(path).write_text(text)

print("M9.3 Clippy refactor applied")
