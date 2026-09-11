import copy
import hashlib
import json
from pathlib import Path

checked_path = Path("research/general-demography-baseline-v1/confirmatory-result.json")
expected_path = Path("/tmp/issue304-v37/expected-result.json")

checked = json.loads(checked_path.read_text())
expected = json.loads(expected_path.read_text())

assert checked["modelSemanticsId"] == "anthrosim-model-semantics-v36"
assert expected["modelSemanticsId"] == "anthrosim-model-semantics-v37"
assert checked["recommendation"] == expected["recommendation"] == "no_universal_demographic_baseline"

guard_only_precision_fields = {"normalApproximationValidity", "confidenceSemantics"}


def comparable_science(value):
    value = copy.deepcopy(value)
    value.pop("researchId", None)
    value.pop("modelSemanticsId", None)
    for diagnostic in value.get("monteCarloPrecision", {}).values():
        precision = diagnostic.get("precision", {})
        for key in guard_only_precision_fields:
            precision.pop(key, None)
    return value


checked_science = comparable_science(checked)
expected_science = comparable_science(expected)
checked_bytes = json.dumps(checked_science, sort_keys=True, separators=(",", ":")).encode()
expected_bytes = json.dumps(expected_science, sort_keys=True, separators=(",", ":")).encode()
checked_digest = hashlib.sha256(checked_bytes).hexdigest()
expected_digest = hashlib.sha256(expected_bytes).hexdigest()

print(f"checked_normalized_sha256={checked_digest}")
print(f"v37_normalized_sha256={expected_digest}")
print(f"normalized_scientific_payload_equal={checked_science == expected_science}")
print(f"checked_research_id={checked['researchId']}")
print(f"v37_research_id={expected['researchId']}")

assert checked_science == expected_science, (
    "fresh v37 780-run confirmatory result changed scientific payload beyond approved "
    "identity/guard-only metadata; refusing reference rebind"
)

text = checked_path.read_text()
old = '"modelSemanticsId": "anthrosim-model-semantics-v36"'
new = '"modelSemanticsId": "anthrosim-model-semantics-v37"'
assert text.count(old) == 1
checked_path.write_text(text.replace(old, new, 1))
print("demographic_reference_rebound=v36_to_v37")
