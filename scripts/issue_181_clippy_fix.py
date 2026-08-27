from pathlib import Path

path = Path("crates/anthrosim-core/src/research_readiness.rs")
text = path.read_text()
old = "        assess_compatible_records(*path, claim, &compatible, failures);\n"
new = "        assess_compatible_records(path, claim, &compatible, failures);\n"
if text.count(old) != 1:
    raise SystemExit("expected exactly one explicit-auto-deref site")
path.write_text(text.replace(old, new, 1))
