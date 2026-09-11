from pathlib import Path
import sys

root = Path(sys.argv[1])
migration = root / "crates/anthrosim-core/src/migration.rs"
text = migration.read_text()
start = text.index("        let period_need_per_person = fixed_annual_quantity_for_period(")
end_marker = "        self.decision_order.clear();\n"
end = text.index(end_marker, start)
replacement = '''        // EXPERIMENT-ONLY COUNTERFACTUAL: every M4 evaluation uses the demand
        // associated with the current canonical quarter-year planning horizon,
        // while decision opportunity frequency varies independently. This keeps
        // D=4 exactly on the ordinary v37 quarter allocations (91/91/91/92 days).
        let _ = decision_index_in_year;
        let day_in_year = {
            let remainder = day % 365;
            if remainder == 0 { 365 } else { remainder }
        };
        let planning_period_index = if day_in_year <= 91 {
            0
        } else if day_in_year <= 182 {
            1
        } else if day_in_year <= 273 {
            2
        } else {
            3
        };
        let period_need_per_person = fixed_annual_quantity_for_period(
            u64::from(annual_food_need),
            planning_period_index,
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
