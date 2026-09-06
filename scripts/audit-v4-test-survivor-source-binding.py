#!/usr/bin/env python3
"""Fresh Audit-v4 Area L survivor-observable semantic-binding adversary."""

from __future__ import annotations

import importlib.util
import json
from copy import deepcopy
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "research-survivor-conditioning.py"
spec = importlib.util.spec_from_file_location("survivor_conditioning", SCRIPT)
if spec is None or spec.loader is None:
    raise RuntimeError("cannot load survivor-conditioning gate")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

BASE = json.loads((ROOT / "examples" / "study-protocol-v1.json").read_text())
TOKENS = (
    "estimand=survivor_condition_at_boundary "
    "conditioning=survival "
    "death_handling=no_post_death_imputation"
)


def protocol_with(joint_source: str) -> dict:
    protocol = deepcopy(BASE)
    protocol["observables"] = [
        {
            "id": "survivor_condition",
            "role": "primary",
            "source": "metrics.resources.meanLivingConditionPermille",
            "analysisWindowId": "primary_window",
            "interpretation": TOKENS,
        },
        {
            "id": "joint_survival",
            "role": "secondary",
            "source": joint_source,
            "analysisWindowId": "primary_window",
            "interpretation": "joint population outcome",
        },
    ]
    protocol["comparisons"][0]["observableIds"] = [
        "survivor_condition",
        "joint_survival",
    ]
    return protocol


def main() -> None:
    genuine = module.validate_protocol(protocol_with("metrics.population.finalLivingPopulation"))
    assert genuine["valid"] is True

    no_marker = module.validate_protocol(protocol_with("derived.unrelated_observable"))
    assert no_marker["valid"] is False
    assert any("without a jointly declared survival/population observable" in x for x in no_marker["failures"])

    fake = module.validate_protocol(protocol_with("derived.not_a_real_mortality_observable"))
    print(f"audit_v4_area_l_genuine_joint_survival_valid={genuine['valid']}")
    print(f"audit_v4_area_l_unrelated_source_valid={no_marker['valid']}")
    print(f"audit_v4_area_l_fake_mortality_source_valid={fake['valid']}")
    print(f"audit_v4_area_l_fake_mortality_failures={len(fake['failures'])}")

    # Intended adversarial assertion: a free-form label that merely contains the
    # substring 'mortality' is not evidence that a real survival/population output
    # has been jointly declared. Frozen v0.3.4 currently accepts it.
    assert fake["valid"] is False, (
        "survivor-conditioning gate accepted a fabricated joint-survival source "
        "solely because its free-form label contains the substring 'mortality'"
    )


if __name__ == "__main__":
    main()
