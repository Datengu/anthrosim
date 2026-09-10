#!/usr/bin/env python3
"""Current-state derivative for AV6-010 post-merge re-verification.

The immutable #728 discovery adversary is intentionally red when the defect is present:
its terminal guard raises after proving that a contradictory 0/30 sample was accepted,
verified, and replayed against 30/30 authoritative completed runs. On repaired current
main, the unchanged historical harness now exits earlier because the contradictory sample
is correctly rejected. This wrapper preserves the original adversary byte-for-byte and
changes only that discovery-era terminal expectation into a re-verification expectation.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = ROOT / "scripts" / "audit-v6-area-h-monte-carlo-sample-binding.py"

EXPECTED_BINDING_REJECTION = (
    "research-monte-carlo-confirmatory: confirmatory sample-value binding failed for "
    "observable 'run_completed_probability', seed 8601: submitted 0.0, authoritative 1.0 "
    "from research.analysis.runs.state"
)
EXPECTED_PROVENANCE_REJECTION = (
    "research-analysis-provenance: analysis command failed with exit code 1; "
    "provenance was not published"
)
DISCOVERY_DEFECT_ORACLE = "predeclared sample-binding oracle failed"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--study-binary", type=Path, required=True)
    parser.add_argument("--research-binary", type=Path, required=True)
    args = parser.parse_args()

    completed = subprocess.run(
        [
            sys.executable,
            str(ORIGINAL),
            "--study-binary",
            str(args.study_binary),
            "--research-binary",
            str(args.research_binary),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    combined = completed.stdout + "\n" + completed.stderr
    assert completed.returncode != 0, (
        "re-verification failed: the historical discovery adversary unexpectedly completed "
        "without rejecting the contradictory sample"
    )
    assert EXPECTED_BINDING_REJECTION in combined, (
        "re-verification failed: historical adversary did not reach the repaired semantic "
        "sample-value binding rejection\nstdout:\n"
        + completed.stdout
        + "\nstderr:\n"
        + completed.stderr
    )
    assert EXPECTED_PROVENANCE_REJECTION in combined, (
        "re-verification failed: contradictory analysis was not rejected by the canonical "
        "analysis-provenance execution path"
    )
    assert DISCOVERY_DEFECT_ORACLE not in combined, (
        "re-verification failed: the original defect-presence oracle was still reached, so "
        "the contradictory sample remains acceptance-valid"
    )

    print("historical_adversary_positive_control=pass")
    print("contradictory_sample_binding=rejected")
    print("contradictory_analysis_provenance=not_published")
    print("discovery_defect_oracle_reached=false")
    print("av6_010_current_state_reverification=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
