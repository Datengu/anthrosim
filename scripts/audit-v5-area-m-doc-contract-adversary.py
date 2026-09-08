#!/usr/bin/env python3
"""Fresh Audit-v5 Area-M adversary for documentation/claim contracts."""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
AUTHORITY_DOCS = [
    ROOT / "docs" / "scientific-model.md",
    ROOT / "docs" / "research" / "odd.md",
    ROOT / "docs" / "research" / "odd-d.md",
    ROOT / "docs" / "research" / "trace.md",
]

text = "\n".join(path.read_text(encoding="utf-8") for path in AUTHORITY_DOCS).lower()

close_kin_parentage_disclosed = bool(
    re.search(
        r"(?:parentage|mating).{0,180}(?:does not|do not|without|no ).{0,120}(?:close[- ]kin|first[- ]degree|direct[- ]parent|parent[- ]child)"
        r"|(?:close[- ]kin|first[- ]degree|direct[- ]parent|parent[- ]child).{0,180}(?:parentage|mating).{0,120}(?:does not|do not|without|no )",
        text,
        re.S,
    )
)

tolerance_named = "corroborationdiscriminationtolerance" in text
nonnegative_near_tolerance = bool(
    re.search(
        r"corroborationdiscriminationtolerance.{0,220}(?:non[- ]negative|>=\s*0|greater than or equal to zero)"
        r"|(?:non[- ]negative|>=\s*0|greater than or equal to zero).{0,220}corroborationdiscriminationtolerance",
        text,
        re.S,
    )
)
threshold_domain_disclosed = tolerance_named and nonnegative_near_tolerance

survivor_window_alignment_disclosed = bool(
    re.search(
        r"survivor.{0,240}(?:same|matching).{0,100}(?:analysis window|window|boundary)"
        r"|(?:same|matching).{0,100}(?:analysis window|window|boundary).{0,240}survivor",
        text,
        re.S,
    )
)

print(f"close_kin_parentage_disclosed={close_kin_parentage_disclosed}")
print(f"threshold_domain_disclosed={threshold_domain_disclosed}")
print(f"survivor_window_alignment_disclosed={survivor_window_alignment_disclosed}")

# Current main is causally equivalent to immutable v0.3.5/v33 for these unrepaired
# contracts. Demonstrating their absence from the authoritative claim surface is
# fresh Area-M evidence for existing AV5-002, AV5-006 and AV5-008, not a new issue.
assert not close_kin_parentage_disclosed
assert not threshold_domain_disclosed
assert not survivor_window_alignment_disclosed
