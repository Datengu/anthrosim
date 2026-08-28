#!/usr/bin/env python3
"""Apply a strict, provenance-bound archaeological observation model to simulated values.

This is deliberately downstream analysis. It does not modify authoritative AnthroSim state.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from fractions import Fraction
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
PER_MILLION = 1_000_000
HEX64 = set("0123456789abcdef")


def fail(message: str) -> None:
    raise ValueError(message)


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def require_object(value: Any, role: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        fail(f"{role} must be a JSON object")
    return value


def strict_keys(value: dict[str, Any], allowed: set[str], required: set[str], role: str) -> None:
    unknown = sorted(set(value) - allowed)
    missing = sorted(required - set(value))
    if unknown:
        fail(f"{role} contains unknown field(s): {', '.join(unknown)}")
    if missing:
        fail(f"{role} is missing required field(s): {', '.join(missing)}")


def require_nonempty_string(value: Any, role: str) -> str:
    if not isinstance(value, str) or not value.strip():
        fail(f"{role} must be a non-empty string")
    return value


def require_sha256(value: Any, role: str) -> str:
    text = require_nonempty_string(value, role)
    if len(text) != 64 or any(char not in HEX64 for char in text):
        fail(f"{role} must be a lowercase 64-character SHA-256 hex digest")
    return text


def require_permille_probability(value: Any, role: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or not 0 <= value <= PER_MILLION:
        fail(f"{role} must be an integer in 0..={PER_MILLION}")
    return value


def require_nonnegative_integer(value: Any, role: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        fail(f"{role} must be a non-negative integer")
    return value


def canonical_identity(value: Any, prefix: str) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return f"{prefix}-{hashlib.sha256(encoded).hexdigest()}"


def validate_source(value: Any, role: str) -> dict[str, Any]:
    source = require_object(value, role)
    strict_keys(
        source,
        {"id", "kind", "contentSha256", "reference"},
        {"id", "kind", "contentSha256", "reference"},
        role,
    )
    require_nonempty_string(source["id"], f"{role}.id")
    require_nonempty_string(source["kind"], f"{role}.kind")
    require_sha256(source["contentSha256"], f"{role}.contentSha256")
    require_nonempty_string(source["reference"], f"{role}.reference")
    return source


def validate_mapping(value: Any, index: int) -> dict[str, Any]:
    role = f"mappings[{index}]"
    mapping = require_object(value, role)
    strict_keys(
        mapping,
        {
            "mappingId",
            "simulatedVariable",
            "archaeologicalObservable",
            "relationship",
            "depositionPerMillion",
            "preservationPerMillion",
            "samplingPerMillion",
            "recoveryPerMillion",
            "assumptions",
            "uncertaintyNote",
        },
        {
            "mappingId",
            "simulatedVariable",
            "archaeologicalObservable",
            "relationship",
            "assumptions",
            "uncertaintyNote",
        },
        role,
    )
    require_nonempty_string(mapping["mappingId"], f"{role}.mappingId")
    require_nonempty_string(mapping["simulatedVariable"], f"{role}.simulatedVariable")
    relationship = require_nonempty_string(mapping["relationship"], f"{role}.relationship")
    if relationship not in {"independent_detection_count", "no_direct_observable"}:
        fail(f"{role}.relationship must be independent_detection_count or no_direct_observable")
    assumptions = mapping["assumptions"]
    if not isinstance(assumptions, list) or not assumptions:
        fail(f"{role}.assumptions must be a non-empty array")
    for assumption_index, assumption in enumerate(assumptions):
        require_nonempty_string(assumption, f"{role}.assumptions[{assumption_index}]")
    require_nonempty_string(mapping["uncertaintyNote"], f"{role}.uncertaintyNote")

    if relationship == "no_direct_observable":
        if mapping["archaeologicalObservable"] is not None:
            fail(f"{role}.archaeologicalObservable must be null for no_direct_observable")
        forbidden = [
            key
            for key in ("depositionPerMillion", "preservationPerMillion", "samplingPerMillion", "recoveryPerMillion")
            if key in mapping
        ]
        if forbidden:
            fail(f"{role} must not define detection-stage probabilities for no_direct_observable")
    else:
        require_nonempty_string(mapping["archaeologicalObservable"], f"{role}.archaeologicalObservable")
        for key in ("depositionPerMillion", "preservationPerMillion", "samplingPerMillion", "recoveryPerMillion"):
            if key not in mapping:
                fail(f"{role} is missing required field {key} for independent_detection_count")
            require_permille_probability(mapping[key], f"{role}.{key}")
    return mapping


def validate_model(value: Any) -> dict[str, Any]:
    model = require_object(value, "observation model")
    strict_keys(
        model,
        {
            "schemaVersion",
            "observationModelId",
            "comparisonId",
            "simulationSource",
            "evidenceSource",
            "evidenceRole",
            "mappings",
        },
        {
            "schemaVersion",
            "observationModelId",
            "comparisonId",
            "simulationSource",
            "evidenceSource",
            "evidenceRole",
            "mappings",
        },
        "observation model",
    )
    if model["schemaVersion"] != SCHEMA_VERSION:
        fail(f"observation model schemaVersion must be {SCHEMA_VERSION}")
    require_nonempty_string(model["observationModelId"], "observationModelId")
    require_nonempty_string(model["comparisonId"], "comparisonId")
    validate_source(model["simulationSource"], "simulationSource")
    validate_source(model["evidenceSource"], "evidenceSource")
    role = require_nonempty_string(model["evidenceRole"], "evidenceRole")
    if role not in {"calibration", "validation", "corroboration", "descriptive"}:
        fail("evidenceRole must be calibration, validation, corroboration, or descriptive")
    mappings = model["mappings"]
    if not isinstance(mappings, list) or not mappings:
        fail("mappings must be a non-empty array")
    validated = [validate_mapping(mapping, index) for index, mapping in enumerate(mappings)]
    ids = [mapping["mappingId"] for mapping in validated]
    if len(ids) != len(set(ids)):
        fail("mappingId values must be unique")
    return model


def validate_simulated(value: Any) -> dict[str, Any]:
    simulated = require_object(value, "simulated input")
    strict_keys(
        simulated,
        {"schemaVersion", "simulationSourceId", "values"},
        {"schemaVersion", "simulationSourceId", "values"},
        "simulated input",
    )
    if simulated["schemaVersion"] != 1:
        fail("simulated input schemaVersion must be 1")
    require_nonempty_string(simulated["simulationSourceId"], "simulationSourceId")
    values = require_object(simulated["values"], "simulated input values")
    for key, value_item in values.items():
        require_nonempty_string(key, "simulated variable name")
        require_nonnegative_integer(value_item, f"simulated input values.{key}")
    return simulated


def detection_fraction(mapping: dict[str, Any]) -> Fraction:
    probability = Fraction(1, 1)
    for key in ("depositionPerMillion", "preservationPerMillion", "samplingPerMillion", "recoveryPerMillion"):
        probability *= Fraction(mapping[key], PER_MILLION)
    return probability


def apply_model(model: dict[str, Any], simulated: dict[str, Any]) -> dict[str, Any]:
    if simulated["simulationSourceId"] != model["simulationSource"]["id"]:
        fail("simulated input simulationSourceId does not match observation model simulationSource.id")

    values = simulated["values"]
    results: list[dict[str, Any]] = []
    for mapping in model["mappings"]:
        variable = mapping["simulatedVariable"]
        if variable not in values:
            fail(f"simulated input does not define required variable {variable!r}")
        source_value = require_nonnegative_integer(values[variable], f"simulated value {variable}")
        if mapping["relationship"] == "no_direct_observable":
            results.append(
                {
                    "mappingId": mapping["mappingId"],
                    "simulatedVariable": variable,
                    "simulatedValue": source_value,
                    "archaeologicalObservable": None,
                    "status": "not_comparable",
                    "absenceSemantics": "no_defensible_direct_archaeological_mapping",
                }
            )
            continue

        probability = detection_fraction(mapping)
        expected = probability * source_value
        results.append(
            {
                "mappingId": mapping["mappingId"],
                "simulatedVariable": variable,
                "simulatedValue": source_value,
                "archaeologicalObservable": mapping["archaeologicalObservable"],
                "status": "comparable_through_observation_model",
                "distribution": {
                    "family": "binomial",
                    "trials": source_value,
                    "successProbability": {
                        "numerator": probability.numerator,
                        "denominator": probability.denominator,
                    },
                },
                "expectedDetectedCount": {
                    "numerator": expected.numerator,
                    "denominator": expected.denominator,
                },
                "absenceSemantics": (
                    "simulated_absence" if source_value == 0 else "non_detection_possible_after_deposition_preservation_sampling_recovery"
                ),
            }
        )

    model_identity = canonical_identity(model, "archaeological-observation-model-v1")
    simulated_identity = canonical_identity(simulated, "archaeological-simulated-input-v1")
    result = {
        "schemaVersion": 1,
        "observationModelId": model["observationModelId"],
        "observationModelIdentity": model_identity,
        "comparisonId": model["comparisonId"],
        "simulationSource": model["simulationSource"],
        "evidenceSource": model["evidenceSource"],
        "evidenceRole": model["evidenceRole"],
        "simulatedInputIdentity": simulated_identity,
        "results": results,
    }
    result["resultIdentity"] = canonical_identity(result, "archaeological-observation-result-v1")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Apply a versioned archaeological observation model")
    parser.add_argument("--model", type=Path, required=True)
    parser.add_argument("--simulated", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    try:
        model = validate_model(load_json(args.model))
        simulated = validate_simulated(load_json(args.simulated))
        result = apply_model(model, simulated)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except (OSError, json.JSONDecodeError, ValueError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
