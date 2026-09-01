#!/usr/bin/env python3
"""Independent post-merge adversary for Audit-v3 AV3-009 / issue #416."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ANALYZER = ROOT / "scripts" / "research-identifiability.py"
spec = importlib.util.spec_from_file_location("anthrosim_research_identifiability", ANALYZER)
assert spec is not None and spec.loader is not None
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def deterministic(outputs: dict[str, float]) -> dict[str, dict[str, str]]:
    return {name: {"kind": "deterministic"} for name in outputs}


def structural_plan(*, held_out: bool = False) -> dict:
    return {
        "schemaVersion": 2,
        "analysisId": "av3-009-independent-reverification",
        "calibrationTargets": [{"observable": "score", "target": 0.0, "tolerance": 0.0}],
        "corroborationObservables": ["held"] if held_out else [],
        "corroborationDiscriminationTolerance": 0.5,
        "claim": {"parameterIds": [], "structuralHypothesis": True},
        "maxNormalizedAcceptableWidth": 0.25,
    }


def point(point_id: str, structure, *, held: float | None = None) -> dict:
    outputs = {"score": 0.0}
    if held is not None:
        outputs["held"] = held
    value = {
        "id": point_id,
        "parameters": {},
        "structure": structure,
        "outputs": outputs,
        "outputEvidence": deterministic(outputs),
    }
    return value


def expect_error(plan: dict, points: list[dict], expected: str) -> None:
    try:
        module.analyse(
            plan,
            {"schemaVersion": 2, "monteCarloDiagnostics": {}, "points": points},
        )
    except module.IdentifiabilityError as error:
        assert expected in str(error), str(error)
    else:
        raise AssertionError(f"expected fail-closed error containing {expected!r}")


def main() -> None:
    # Exact frozen AV3-009 typed-identity adversary: numeric JSON 1 and string "1"
    # must not collapse to one structural identity.
    expect_error(
        structural_plan(),
        [point("numeric-structure", 1), point("string-structure", "1")],
        "structure must be a non-empty string",
    )

    missing = point("missing-structure", "temporary")
    del missing["structure"]
    expect_error(
        structural_plan(),
        [missing],
        "requires a non-empty string structure identifier",
    )

    # Two genuinely distinct canonical strings must remain two compatible structures,
    # so a structural-identification claim cannot pass.
    result = module.analyse(
        structural_plan(),
        {
            "schemaVersion": 2,
            "monteCarloDiagnostics": {},
            "points": [point("a", "structure-a"), point("b", "structure-b")],
        },
    )
    assert result["structuralDiagnostic"] == {
        "compatibleStructures": ["structure-a", "structure-b"],
        "identified": False,
        "equifinal": True,
    }
    assert result["researchGate"]["passes"] is False
    assert result["equifinality"]["present"] is True

    # Held-out grouping must use the same exact canonical identity semantics.
    held = module.analyse(
        structural_plan(held_out=True),
        {
            "schemaVersion": 2,
            "monteCarloDiagnostics": {},
            "points": [
                point("a-held", "structure-a", held=0.0),
                point("b-held", "structure-b", held=10.0),
            ],
        },
    )
    predictions = held["discriminatingPredictions"]
    assert len(predictions) == 1
    prediction = predictions[0]
    assert prediction["structures"] == ["structure-a", "structure-b"]
    assert prediction["leftSimulationIntervalEnvelope"] == [0.0, 0.0]
    assert prediction["rightSimulationIntervalEnvelope"] == [10.0, 10.0]
    assert prediction["minimumIntervalSeparation"] == 10.0
    assert prediction["discriminating"] is True

    print("AV3-009 independent post-merge reverification: ok")


if __name__ == "__main__":
    main()
