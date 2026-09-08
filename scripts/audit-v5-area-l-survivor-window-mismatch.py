#!/usr/bin/env python3
"""Fresh Audit-v5 Area-L adversary for survivor-conditioning window alignment."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "scripts" / "research-survivor-conditioning.py"
SPEC = importlib.util.spec_from_file_location("research_survivor_conditioning", TARGET)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def observable(observable_id: str, source: str, window: str, interpretation: str) -> dict:
    return {
        "id": observable_id,
        "role": "primary" if observable_id == "condition" else "secondary",
        "source": source,
        "analysisWindowId": window,
        "interpretation": interpretation,
    }


def protocol(population_window: str) -> dict:
    condition_interpretation = " ".join(
        (
            MODULE.ESTIMAND_TOKEN,
            MODULE.CONDITIONING_TOKEN,
            MODULE.DEATH_HANDLING_TOKEN,
        )
    )
    return {
        "schemaVersion": 1,
        "studyId": "audit-v5-area-l-survivor-window",
        "analysisWindows": [
            {"id": "early", "startDay": 0, "endDay": 30},
            {"id": "terminal", "startDay": 335, "endDay": 365},
        ],
        "observables": [
            observable(
                "condition",
                "metrics.json.meanLivingConditionPermille",
                "terminal",
                condition_interpretation,
            ),
            observable(
                "living_population",
                "metrics.population.finalLivingPopulation",
                population_window,
                "population outcome used to disclose survival conditioning",
            ),
        ],
        "comparisons": [
            {
                "id": "treatment-effect",
                "observableIds": ["condition", "living_population"],
            }
        ],
    }


def main() -> int:
    matched = MODULE.validate_protocol(protocol("terminal"))
    mismatched = MODULE.validate_protocol(protocol("early"))

    assert matched["valid"], matched

    print(
        "matched_window_valid=",
        matched["valid"],
        "; mismatched_window_valid=",
        mismatched["valid"],
        "; mismatched_failures=",
        mismatched["failures"],
        sep="",
    )

    # Scientific oracle: the jointly declared survival/population observable must
    # measure the same analysis boundary/window as the survivor-conditioned
    # condition estimand. An early-window population observable cannot disclose
    # terminal survivor selection.
    assert not mismatched["valid"], (
        "survivor-conditioning gate accepted a terminal survivor-conditioned "
        "condition comparison whose only survival/population observable is bound "
        "to a different analysis window"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())