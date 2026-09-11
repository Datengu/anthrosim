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


def normalize_top_level(value):
    value = copy.deepcopy(value)
    value.pop("researchId", None)
    value.pop("modelSemanticsId", None)
    value.pop("arms", None)
    value.pop("pairedHouseholdEffects", None)
    for diagnostic in value.get("monteCarloPrecision", {}).values():
        precision = diagnostic.get("precision", {})
        for key in guard_only_precision_fields:
            precision.pop(key, None)
    return value


def arm_key(arm):
    return (
        arm["demography"],
        arm["householdLifecycle"],
        arm["resourceProductivityScalePermille"],
        arm["founderAgeCeilingYears"],
    )


def paired_key(effect):
    return (
        effect["demography"],
        effect["fissionHouseholdLifecycle"],
        effect["fixedHouseholdLifecycle"],
        effect["resourceProductivityScalePermille"],
        effect["founderAgeCeilingYears"],
    )


# AV6-004 changes only exact equal-remainder allocation under scarce-resource ties. The fresh
# v37 confirmatory run demonstrates that this legitimately propagates into two fixed-founder
# controls (negative-growth and replacement) and the paired household effects derived from them.
# Everything else in the scientific payload must reproduce exactly before the canonical result is
# refreshed; this prevents a broad evidence overwrite from hiding an unrelated model change.
allowed_changed_arm_demographies = {
    "negative_growth_control_v1",
    "replacement_control_v1",
}
allowed_changed_paired_demographies = {
    "negative_growth_control_v1",
    "replacement_control_v1",
}

assert normalize_top_level(checked) == normalize_top_level(expected), (
    "fresh v37 result changed top-level scientific content outside the AV6-004 downstream surfaces"
)

checked_arms = {arm_key(arm): arm for arm in checked["arms"]}
expected_arms = {arm_key(arm): arm for arm in expected["arms"]}
assert checked_arms.keys() == expected_arms.keys()
changed_arm_keys = {key for key in checked_arms if checked_arms[key] != expected_arms[key]}
expected_changed_arm_keys = {
    key
    for key in checked_arms
    if key[0] in allowed_changed_arm_demographies and key[1] == "fixed_founder_v1"
}
assert changed_arm_keys == expected_changed_arm_keys, (
    f"unexpected AV6-004 downstream arm changes: observed={sorted(changed_arm_keys)!r} "
    f"expected={sorted(expected_changed_arm_keys)!r}"
)

checked_pairs = {paired_key(effect): effect for effect in checked["pairedHouseholdEffects"]}
expected_pairs = {paired_key(effect): effect for effect in expected["pairedHouseholdEffects"]}
assert checked_pairs.keys() == expected_pairs.keys()
changed_pair_keys = {key for key in checked_pairs if checked_pairs[key] != expected_pairs[key]}
expected_changed_pair_keys = {
    key for key in checked_pairs if key[0] in allowed_changed_paired_demographies
}
assert changed_pair_keys == expected_changed_pair_keys, (
    f"unexpected AV6-004 downstream paired-effect changes: observed={sorted(changed_pair_keys)!r} "
    f"expected={sorted(expected_changed_pair_keys)!r}"
)

checked_digest = hashlib.sha256(
    json.dumps(checked, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
expected_digest = hashlib.sha256(
    json.dumps(expected, sort_keys=True, separators=(",", ":")).encode()
).hexdigest()
print(f"checked_sha256={checked_digest}")
print(f"v37_sha256={expected_digest}")
print(f"checked_research_id={checked['researchId']}")
print(f"v37_research_id={expected['researchId']}")
print(f"validated_changed_arms={sorted(changed_arm_keys)!r}")
print(f"validated_changed_paired_effects={sorted(changed_pair_keys)!r}")

checked_path.write_text(expected_path.read_text())
print("demographic_reference_refreshed=v36_result_to_fresh_v37_result")
