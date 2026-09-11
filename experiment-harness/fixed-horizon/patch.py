from pathlib import Path
import sys

root = Path(sys.argv[1])
migration = root / "crates/anthrosim-core/src/migration.rs"
text = migration.read_text()
start = text.index("        let period_need_per_person = fixed_annual_quantity_for_period(")
end_marker = "        self.decision_order.clear();\n"
end = text.index(end_marker, start)
replacement = '''        // EXPERIMENT-ONLY COUNTERFACTUAL: every M4 evaluation uses a fixed
        // quarter-year resource-support demand while decision frequency varies.
        // With annual need = 100 this is exactly 25 units/person per evaluation.
        let _ = decision_index_in_year;
        let period_need_per_person = fixed_annual_quantity_for_period(
            u64::from(annual_food_need),
            0,
            4,
        )
        .ok_or(MigrationError::InternalInvariant(
            "fixed-quarter migration planning horizon could not be allocated",
        ))?;

'''
migration.write_text(text[:start] + replacement + text[end:])

provenance = root / "crates/anthrosim-core/src/provenance.rs"
p = provenance.read_text()
old = 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v37";'
new = 'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v37-fixed-quarter-horizon-experiment";'
if p.count(old) != 1:
    raise SystemExit("unexpected v37 provenance source")
provenance.write_text(p.replace(old, new))
