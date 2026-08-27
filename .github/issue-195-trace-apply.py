from pathlib import Path

migration = Path("crates/anthrosim-core/src/migration.rs")
text = migration.read_text()

utility_struct = '''#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationUtilityBreakdown {
    pub resource_score_permille: u16,
    pub water_security_score_permille: u16,
    pub kin_score_permille: u16,
    pub travel_penalty_permille: u16,
    pub uncertainty_penalty_permille: u16,
    pub relocation_risk_penalty_permille: u16,
    pub total_utility: i32,
}
'''
choice_struct = '''
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationCandidateChoiceWeight {
    pub cell: CellId,
    pub utility: i32,
    pub weight: u64,
}
'''
if text.count(utility_struct) != 1:
    raise SystemExit("expected utility struct exactly once")
text = text.replace(utility_struct, utility_struct + choice_struct, 1)

trace_anchor = '''    pub selected_weight: u64,
    pub total_move_weight: u64,
    pub choice_draw: u64,
'''
trace_replacement = '''    pub selected_weight: u64,
    pub total_move_weight: u64,
    pub choice_draw: u64,
    /// Stable candidate-order table for every eligible alternative in the weighted draw.
    pub eligible_candidate_weights: Vec<MigrationCandidateChoiceWeight>,
'''
if text.count(trace_anchor) != 1:
    raise SystemExit("expected trace weight fields exactly once")
text = text.replace(trace_anchor, trace_replacement, 1)

text = text.replace(
    '''    /// v2 represented move-conditional means as null when the move observation set was empty.
    /// v3 distinguishes the nominal requested decrement from exact realized condition loss in traces.
    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
    '''    /// v2 represented move-conditional means as null when the move observation set was empty.
    /// v3 distinguishes the nominal requested decrement from exact realized condition loss in traces.
    /// v4 preserves the complete eligible-candidate weight table for recorded M4 choices.
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;''',
    1,
)
text = text.replace(
    '''    /// v2 carries the explicit nominal/realized travel-condition fields in retained decision traces.
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
    '''    /// v2 carries the explicit nominal/realized travel-condition fields in retained decision traces.
    /// v3 preserves the complete eligible-candidate weight table in retained decision traces.
    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
    1,
)
text = text.replace(
    '''    /// v2 records nominal and realized travel-condition effects separately in migration artifacts.
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;''',
    '''    /// v2 records nominal and realized travel-condition effects separately in migration artifacts.
    /// v3 records all eligible candidate weights for each retained M4 choice trace.
    pub const CURRENT_SCHEMA_VERSION: u32 = 3;''',
    1,
)

push_anchor = '''        if self.recorded_decision_traces.len()
            < usize::try_from(config.max_recorded_decision_traces).unwrap_or(usize::MAX)
        {
            self.recorded_decision_traces.push(MigrationDecisionTrace {'''
push_replacement = '''        if self.recorded_decision_traces.len()
            < usize::try_from(config.max_recorded_decision_traces).unwrap_or(usize::MAX)
        {
            let eligible_candidate_weights = self
                .evaluations
                .iter()
                .map(|evaluation| MigrationCandidateChoiceWeight {
                    cell: evaluation.cell,
                    utility: evaluation.utility.total_utility,
                    weight: evaluation.weight,
                })
                .collect();
            self.recorded_decision_traces.push(MigrationDecisionTrace {'''
if text.count(push_anchor) != 1:
    raise SystemExit("expected recorded trace push anchor exactly once")
text = text.replace(push_anchor, push_replacement, 1)

field_anchor = '''                selected_weight: selected.weight,
                total_move_weight: total_weight,
                choice_draw,
                nominal_travel_condition_cost_per_person: nominal_condition_cost_per_person,'''
field_replacement = '''                selected_weight: selected.weight,
                total_move_weight: total_weight,
                choice_draw,
                eligible_candidate_weights,
                nominal_travel_condition_cost_per_person: nominal_condition_cost_per_person,'''
if text.count(field_anchor) != 1:
    raise SystemExit("expected trace construction weight fields exactly once")
text = text.replace(field_anchor, field_replacement, 1)
migration.write_text(text)

lib = Path("crates/anthrosim-core/src/lib.rs")
ltext = lib.read_text()
old_export = '''pub use migration::{
    MigrationCheckpointState, MigrationConfigError, MigrationDecisionTrace, MigrationError,
    MigrationSummary, MigrationSystem, MigrationUtilityBreakdown, bounded_candidate_cells,
    candidate_count_upper_bound, migration_pressure_permille, validate_migration_config,
};'''
new_export = '''pub use migration::{
    MigrationCandidateChoiceWeight, MigrationCheckpointState, MigrationConfigError,
    MigrationDecisionTrace, MigrationError, MigrationSummary, MigrationSystem,
    MigrationUtilityBreakdown, bounded_candidate_cells, candidate_count_upper_bound,
    migration_pressure_permille, validate_migration_config,
};'''
if ltext.count(old_export) != 1:
    raise SystemExit("expected migration export block exactly once")
lib.write_text(ltext.replace(old_export, new_export, 1))

checkpoint = Path("crates/anthrosim-core/src/checkpoint.rs")
ctext = checkpoint.read_text()
old_checkpoint_versions = '''    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 10;
    pub const PRE_TRAVEL_CONDITION_OBSERVABILITY_SCHEMA_VERSION: u32 = 11;
    pub const CURRENT_SCHEMA_VERSION: u32 = 12;'''
new_checkpoint_versions = '''    pub const PRE_CONTINUATION_IDENTITY_SCHEMA_VERSION: u32 = 10;
    pub const PRE_TRAVEL_CONDITION_OBSERVABILITY_SCHEMA_VERSION: u32 = 11;
    pub const PRE_M4_CHOICE_WEIGHT_TRACE_SCHEMA_VERSION: u32 = 12;
    pub const CURRENT_SCHEMA_VERSION: u32 = 13;'''
if ctext.count(old_checkpoint_versions) != 1:
    raise SystemExit("expected checkpoint schema block exactly once")
checkpoint.write_text(ctext.replace(old_checkpoint_versions, new_checkpoint_versions, 1))

manifest = Path("crates/anthrosim-core/src/manifest.rs")
mtext = manifest.read_text()
old_manifest_versions = '''    pub const PRE_COMPOSED_EVIDENCE_CLOSURE_SCHEMA_VERSION: u32 = 14;
    pub const PRE_TRAVEL_CONDITION_OBSERVABILITY_SCHEMA_VERSION: u32 = 15;
    pub const CURRENT_SCHEMA_VERSION: u32 = 16;'''
new_manifest_versions = '''    pub const PRE_COMPOSED_EVIDENCE_CLOSURE_SCHEMA_VERSION: u32 = 14;
    pub const PRE_TRAVEL_CONDITION_OBSERVABILITY_SCHEMA_VERSION: u32 = 15;
    pub const PRE_M4_CHOICE_WEIGHT_TRACE_SCHEMA_VERSION: u32 = 16;
    pub const CURRENT_SCHEMA_VERSION: u32 = 17;'''
if mtext.count(old_manifest_versions) != 1:
    raise SystemExit("expected manifest schema block exactly once")
manifest.write_text(mtext.replace(old_manifest_versions, new_manifest_versions, 1))

test = Path("crates/anthrosim-core/tests/migration_behavior.rs")
ttext = test.read_text()
old_assertions = '''        assert!(trace.destination_utility.total_utility > trace.origin_utility.total_utility);
        assert_eq!(trace.decision_day, trace.completed_day);
        assert!(trace.people_moved > 0);'''
new_assertions = '''        assert!(trace.destination_utility.total_utility > trace.origin_utility.total_utility);
        assert_eq!(trace.decision_day, trace.completed_day);
        assert!(trace.people_moved > 0);
        assert!(!trace.eligible_candidate_weights.is_empty());
        assert_eq!(
            trace
                .eligible_candidate_weights
                .iter()
                .map(|candidate| candidate.weight)
                .sum::<u64>(),
            trace.total_move_weight
        );
        let required = trace.origin_utility.total_utility.saturating_add(
            i32::try_from(config.migration.minimum_utility_improvement).unwrap_or(i32::MAX),
        );
        for candidate in &trace.eligible_candidate_weights {
            assert!(candidate.utility > required);
            assert_eq!(
                candidate.weight,
                u64::try_from(i64::from(candidate.utility) - i64::from(required)).unwrap()
            );
        }
        let selected = trace
            .eligible_candidate_weights
            .iter()
            .find(|candidate| candidate.cell == trace.destination)
            .expect("selected destination must be present in eligible weight table");
        assert_eq!(selected.weight, trace.selected_weight);
        assert_eq!(selected.utility, trace.destination_utility.total_utility);'''
if ttext.count(old_assertions) != 1:
    raise SystemExit("expected migration behavior assertion block exactly once")
test.write_text(ttext.replace(old_assertions, new_assertions, 1))

migration_doc = Path("docs/research/migration-v0.1.md")
dtext = migration_doc.read_text()
old_doc = '''This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable. `MigrationDecisionTrace` preserves the selected weight, total eligible move weight and choice draw, so the realized weighted draw is auditable; the compact trace does not retain every unselected candidate evaluation.'''
new_doc = '''This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable. `MigrationDecisionTrace` preserves the stable candidate-order table of every eligible candidate's cell, utility and exact weight, together with the selected weight, total move weight and choice draw. The exact probability assigned to every eligible alternative is therefore reconstructible as `candidateWeight / totalMoveWeight` for every retained trace.'''
if dtext.count(old_doc) != 1:
    raise SystemExit("expected migration observability paragraph exactly once")
dtext = dtext.replace(old_doc, new_doc, 1)
old_trace_bullet = '''- selected and total stochastic-choice weights plus the choice draw;'''
new_trace_bullet = '''- every eligible candidate's cell, utility and exact stochastic-choice weight, plus the selected weight, total move weight and choice draw;'''
if dtext.count(old_trace_bullet) != 1:
    raise SystemExit("expected migration trace bullet exactly once")
migration_doc.write_text(dtext.replace(old_trace_bullet, new_trace_bullet, 1))

scientific = Path("docs/scientific-model.md")
stext = scientific.read_text()
old_science = '''A candidate must strictly exceed the configured minimum improvement over staying. Under v14, each eligible alternative's stochastic weight is exactly its positive integer utility improvement above that required threshold, with no `+1` pseudocount; common positive scaling therefore preserves relative choice probabilities. One destination is selected through the named deterministic `migration/choice` random stream.'''
new_science = '''A candidate must strictly exceed the configured minimum improvement over staying. Under v14, each eligible alternative's stochastic weight is exactly its positive integer utility improvement above that required threshold, with no `+1` pseudocount; common positive scaling therefore preserves relative choice probabilities. One destination is selected through the named deterministic `migration/choice` random stream. Retained M4 decision traces preserve every eligible candidate's cell, utility and exact weight so the full categorical choice distribution can be reconstructed for recorded moves.'''
if stext.count(old_science) != 1:
    raise SystemExit("expected scientific M4 v14 paragraph exactly once")
scientific.write_text(stext.replace(old_science, new_science, 1))

trace = Path("docs/research/trace-m4-proportional-choice-repair-2026-08-27.md")
trtext = trace.read_text()
old_observability = '''`MigrationDecisionTrace` already preserves `selectedWeight`, `totalMoveWeight`, and `choiceDraw`. These values expose the realized selected weight and total weighted draw space. The compact trace does not preserve every unselected candidate evaluation; this repair does not expand retained history solely for #195.'''
new_observability = '''`MigrationDecisionTrace` now preserves a stable candidate-order `eligibleCandidateWeights` table containing every eligible candidate's cell, utility and exact weight, in addition to `selectedWeight`, `totalMoveWeight`, and `choiceDraw`. For every retained move, a reviewer can reconstruct the complete categorical distribution as each candidate weight divided by the preserved total and can verify that the selected destination belongs to that table. The trace expansion remains bounded by the existing recorded-decision-trace cap and bounded M4 candidate radius.'''
if trtext.count(old_observability) != 1:
    raise SystemExit("expected TRACE observability paragraph exactly once")
trace.write_text(trtext.replace(old_observability, new_observability, 1))
