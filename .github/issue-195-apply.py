from pathlib import Path

migration = Path("crates/anthrosim-core/src/migration.rs")
text = migration.read_text()
old_weight = """                let weight = u64::try_from(improvement)
                    .unwrap_or(u64::MAX)
                    .saturating_add(1);"""
new_weight = """                // Strict eligibility above guarantees a positive improvement, so the
                // stochastic weight is exactly proportional to declared utility improvement.
                let weight = proportional_choice_weight(improvement);"""
if text.count(old_weight) != 1:
    raise SystemExit("expected exactly one legacy M4 +1 weighting block")
text = text.replace(old_weight, new_weight)

helper_anchor = """fn draw_bounded<R: Rng + ?Sized>(rng: &mut R, upper_exclusive: u64) -> u64 {"""
helper = """fn proportional_choice_weight(improvement: i64) -> u64 {
    debug_assert!(improvement > 0);
    u64::try_from(improvement).unwrap_or(u64::MAX)
}

"""
if text.count(helper_anchor) != 1:
    raise SystemExit("expected draw_bounded anchor exactly once")
text = text.replace(helper_anchor, helper + helper_anchor, 1)

test_anchor = """    #[test]
    fn candidate_lookup_is_bounded_and_local() {"""
tests = """    #[test]
    fn proportional_candidate_weights_match_required_ratios() {
        assert_eq!(
            [1_i64, 2].map(proportional_choice_weight),
            [1_u64, 2]
        );
        assert_eq!(
            [1_i64, 10].map(proportional_choice_weight),
            [1_u64, 10]
        );
        assert_eq!(
            [7_i64, 7].map(proportional_choice_weight),
            [7_u64, 7]
        );
    }

    #[test]
    fn proportional_candidate_weights_are_scale_invariant() {
        let base = [1_i64, 2, 10].map(proportional_choice_weight);
        let scaled = [13_i64, 26, 130].map(proportional_choice_weight);
        for index in 0..base.len() {
            assert_eq!(scaled[index], base[index] * 13);
        }
    }

"""
if text.count(test_anchor) != 1:
    raise SystemExit("expected candidate test marker exactly once")
text = text.replace(test_anchor, tests + test_anchor, 1)
migration.write_text(text)

provenance = Path("crates/anthrosim-core/src/provenance.rs")
ptext = provenance.read_text()
old_id = 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v13";'
new_id = 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v14";'
if ptext.count(old_id) != 1:
    raise SystemExit("expected v13 model semantics ID exactly once")
provenance.write_text(ptext.replace(old_id, new_id))

migration_doc = Path("docs/research/migration-v0.1.md")
dtext = migration_doc.read_text()
old_choice = """Candidates that clear the minimum utility improvement receive a weight proportional to their utility improvement. One destination is then drawn from those weights using the named `migration/choice` random stream. Candidate uncertainty uses the independent `migration/uncertainty` stream.

This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable."""
new_choice = """Candidates that strictly clear the minimum utility improvement receive a stochastic weight equal to their positive integer utility improvement above that required threshold. There is no `+1` pseudocount: improvements `[1, 2]` produce weights `[1, 2]`, `[1, 10]` produces `[1, 10]`, equal improvements receive equal weights, and multiplying every eligible improvement by one common positive factor preserves the relative selection probabilities. One destination is then drawn from those weights using the named `migration/choice` random stream. Candidate uncertainty uses the independent `migration/uncertainty` stream.

This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable. `MigrationDecisionTrace` preserves the selected weight, total eligible move weight and choice draw, so the realized weighted draw is auditable; the compact trace does not retain every unselected candidate evaluation."""
if dtext.count(old_choice) != 1:
    raise SystemExit("expected deterministic stochastic choice paragraph exactly once")
migration_doc.write_text(dtext.replace(old_choice, new_choice))

scientific = Path("docs/scientific-model.md")
stext = scientific.read_text()
old_status = "**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v13"
new_status = "**Status:** working specification for the AnthroSim v0.3.0 package / post-M9 scientific-hardening line / model semantics v14"
if stext.count(old_status) != 1:
    raise SystemExit("expected scientific model v13 status exactly once")
stext = stext.replace(old_status, new_status)
old_sentence = "A candidate must exceed the configured minimum improvement over staying. Eligible alternatives receive weights proportional to utility improvement, and one is selected through the named deterministic `migration/choice` random stream."
new_sentence = "A candidate must strictly exceed the configured minimum improvement over staying. Under v14, each eligible alternative's stochastic weight is exactly its positive integer utility improvement above that required threshold, with no `+1` pseudocount; common positive scaling therefore preserves relative choice probabilities. One destination is selected through the named deterministic `migration/choice` random stream."
if stext.count(old_sentence) != 1:
    raise SystemExit("expected scientific-model M4 weighting sentence exactly once")
scientific.write_text(stext.replace(old_sentence, new_sentence))

trace = Path("docs/research/trace-m4-proportional-choice-repair-2026-08-27.md")
trace.write_text("""# TRACE record: M4 proportional destination-choice repair (2026-08-27)

## Finding

Issue #195 identified a mismatch between the documented M4 stochastic destination-choice rule and executable weighting. Candidates are admitted only when their utility strictly exceeds origin utility plus `minimumUtilityImprovement`, so every eligible improvement is a positive integer. The executable rule nevertheless used `improvement + 1`, flattening relative preferences near the threshold.

## Repair

M4 now uses the eligible candidate's exact positive utility improvement as its stochastic weight. No pseudocount is added. Thus `[1, 2]` maps to `[1, 2]`, `[1, 10]` maps to `[1, 10]`, equal improvements remain equal, and common positive scaling preserves relative weights. Candidate eligibility, utility equations, uncertainty draws, candidate ordering, and the bounded integer draw algorithm are otherwise unchanged.

## Observability

`MigrationDecisionTrace` already preserves `selectedWeight`, `totalMoveWeight`, and `choiceDraw`. These values expose the realized selected weight and total weighted draw space. The compact trace does not preserve every unselected candidate evaluation; this repair does not expand retained history solely for #195.

## Compatibility

This changes authoritative M4 behavioural semantics and can alter migration destinations and downstream state for identical configuration and seed. `MODEL_SEMANTICS_ID` therefore advances from `anthrosim-model-semantics-v13` to `anthrosim-model-semantics-v14`. Historical artifacts remain bound to their original semantics identity.

## Acceptance evidence

The production weighting helper is exercised directly by unit coverage for exact `[1,2]`, `[1,10]`, equal-improvement, and common-scale invariance properties. Existing M4 migration, deterministic replay, checkpoint/resume, spatial, and protected benchmark gates remain required to detect unintended collateral effects.
""")
