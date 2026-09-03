#!/usr/bin/env python3
from pathlib import Path

p = Path("crates/anthrosim-core/tests/migration_household_label_invariance.rs")
text = p.read_text(encoding="utf-8")
old = """    let (household_one_cell, household_two_cell, person_one_household, person_two_household) =
        if swapped_labels { (4, 1, 2, 1) } else { (1, 4, 1, 2) };
"""
new = """    let (household_one_cell, household_two_cell, person_one_household, person_two_household) =
        if swapped_labels {
            (4, 1, 2, 1)
        } else {
            (1, 4, 1, 2)
        };
"""
if text.count(old) != 1:
    raise SystemExit("expected tuple-format target once")
text = text.replace(old, new, 1)
old = '    assert!(informative > 0, "regression did not exercise a migration outcome");\n'
new = '''    assert!(
        informative > 0,
        "regression did not exercise a migration outcome"
    );
'''
if text.count(old) != 1:
    raise SystemExit("expected assert-format target once")
text = text.replace(old, new, 1)
p.write_text(text, encoding="utf-8")
Path("scripts/tmp_fmt_av4_003.py").unlink()
