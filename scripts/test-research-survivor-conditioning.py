#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-survivor-conditioning.py")
spec = importlib.util.spec_from_file_location("survivor_conditioning", SCRIPT)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)


def protocol(condition_interpretation: str, include_survival: bool = True) -> dict:
    observables = [
        {
            "id": "terminal_condition",
            "role": "primary",
            "source": "metrics.json.meanLivingConditionPermille",
            "analysisWindowId": "terminal",
            "interpretation": condition_interpretation,
        }
    ]
    ids = ["terminal_condition"]
    if include_survival:
        observables.append(
            {
                "id": "terminal_population",
                "role": "secondary",
                "source": "metrics.json.finalLivingPopulation",
                "analysisWindowId": "terminal",
                "interpretation": "Terminal living population; defined even after extinction.",
            }
        )
        ids.append("terminal_population")
    return {
        "studyId": "issue-229-synthetic",
        "observables": observables,
        "comparisons": [
            {
                "id": "control-v-treatment",
                "observableIds": ids,
            }
        ],
    }


DECLARATION = (
    "Terminal survivor condition. "
    "estimand=survivor_condition_at_boundary; "
    "conditioning=survival; "
    "death_handling=no_post_death_imputation"
)

valid = module.validate_protocol(protocol(DECLARATION))
assert valid["valid"] is True
assert valid["postDeathImputation"] == "none_automatic"

missing_estimand = module.validate_protocol(
    protocol("Terminal condition among living people.")
)
assert missing_estimand["valid"] is False
assert any("estimand=survivor_condition_at_boundary" in failure for failure in missing_estimand["failures"])
assert any("conditioning=survival" in failure for failure in missing_estimand["failures"])
assert any("death_handling=no_post_death_imputation" in failure for failure in missing_estimand["failures"])

missing_survival = module.validate_protocol(protocol(DECLARATION, include_survival=False))
assert missing_survival["valid"] is False
assert any("jointly declared survival/population observable" in failure for failure in missing_survival["failures"])

# Synthetic reversal: the treatment has a higher survivor mean only because the
# low-condition person is absent from the survivor set.
control = {
    "finalLivingPopulation": 10,
    "meanLivingConditionPermille": 740,
}
treatment = {
    "finalLivingPopulation": 9,
    "meanLivingConditionPermille": 800,
}
reversal = module.assess_pair(control, treatment)
assert reversal == {
    "survivorMeanConditionDirection": "higher",
    "livingPopulationDirection": "lower",
    "discordantDirections": True,
    "survivorConditionIsPopulationTreatmentEffect": False,
}

# Extinction leaves survivor condition undefined rather than inventing a post-death value.
extinct = module.assess_pair(
    control,
    {"finalLivingPopulation": 0, "meanLivingConditionPermille": None},
)
assert extinct["survivorMeanConditionDirection"] == "undefined"
assert extinct["survivorConditionIsPopulationTreatmentEffect"] is False

print("survivor-conditioning research gate regression checks passed")
