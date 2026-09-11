import copy
import json
import re
import shutil
import subprocess
from pathlib import Path

# --- Demographic baseline: provenance-only v37 -> v38 rebind ---
demog_path = Path("research/general-demography-baseline-v1/confirmatory-result.json")
old_demog = json.loads(demog_path.read_text())
new_demog_path = Path("/tmp/demog/expected-result.json")
new_demog = json.loads(new_demog_path.read_text())

guard_only_precision_fields = {"normalApproximationValidity", "confidenceSemantics"}

def demographic_science(value):
    value = copy.deepcopy(value)
    value.pop("researchId", None)
    value.pop("modelSemanticsId", None)
    for diagnostic in value.get("monteCarloPrecision", {}).values():
        precision = diagnostic.get("precision", {})
        for key in guard_only_precision_fields:
            precision.pop(key, None)
    return value

assert old_demog["modelSemanticsId"] == "anthrosim-model-semantics-v37"
assert new_demog["modelSemanticsId"] == "anthrosim-model-semantics-v38"
assert old_demog["recommendation"] == new_demog["recommendation"] == "no_universal_demographic_baseline"
assert demographic_science(old_demog) == demographic_science(new_demog), (
    "AV6-006 must not alter the demographic scientific result; v38 rebind refused"
)
assert re.fullmatch(r"research-execution-v1-[0-9a-f]{16}", new_demog["researchId"])
demog_path.write_text(new_demog_path.read_text())

demog_doc_path = Path("docs/research/general-scientific-demographic-baseline-v1.md")
demog_doc = demog_doc_path.read_text()
assert old_demog["researchId"] in demog_doc
assert "current model semantics v38" in demog_doc.lower()
demog_doc = demog_doc.replace(old_demog["researchId"], new_demog["researchId"])
demog_doc += (
    "\n\n### Audit-v6 AV6-006 v38 applicability re-verification\n\n"
    "AV6-006/#711 changes only M9 equal-cost temporary-destination spatial coupling. "
    "Fresh 780-run confirmatory execution `34555842374` under model semantics v38 reproduced "
    "the complete normalized demographic scientific result exactly after excluding only "
    "execution/model-semantics identity and pre-existing guard-only precision metadata. "
    "All six arm summaries, paired household effects, long-run classifications, Monte Carlo "
    "precision decisions and the `no_universal_demographic_baseline` recommendation therefore "
    "remain unchanged. The checked result is rebound to the fresh v38 research execution rather "
    "than being numerically rebaselined.\n"
)
demog_doc_path.write_text(demog_doc)

# --- M9.7: state-digest/provenance-only refresh under the existing strict projection ---
m97_ref_path = Path("examples/m9-controlled-aggregation-benchmark/reference-result.json")
m97_ref = json.loads(m97_ref_path.read_text())
m97_actual_path = Path("/tmp/m97/m9-benchmark-output/benchmark-summary.json")
m97_actual = json.loads(m97_actual_path.read_text())

assert m97_ref["schemaVersion"] == 2
assert m97_actual["schemaVersion"] == 3
assert m97_ref["referenceExecution"]["modelSemanticsId"] == "anthrosim-model-semantics-v35"
for key in ("benchmarkId", "definitionCanonicalSha256", "declaredSeeds", "classification", "aggregate"):
    assert m97_actual[key] == m97_ref[key], f"M9.7 {key} changed unexpectedly"
for arm in ("continuous", "intermittent"):
    assert m97_actual["arms"][arm]["configCanonicalSha256"] == m97_ref["arms"][arm]["configCanonicalSha256"]
    assert m97_actual["arms"][arm]["stateDigests"] != m97_ref["arms"][arm]["stateDigests"]

candidate = copy.deepcopy(m97_ref)
for arm in ("continuous", "intermittent"):
    candidate["arms"][arm]["stateDigests"] = m97_actual["arms"][arm]["stateDigests"]
tmp_ref = Path("/tmp/reference-result.json")
tmp_ref.write_text(json.dumps(candidate, separators=(",", ":")) + "\n")
shutil.copyfile(
    "examples/m9-controlled-aggregation-benchmark/travel-burden-reference.json",
    "/tmp/travel-burden-reference.json",
)
subprocess.run(
    [
        "python3",
        "scripts/verify-m9-aggregation-benchmark-reference.py",
        "--actual",
        str(m97_actual_path),
        "--reference",
        str(tmp_ref),
    ],
    check=True,
)

# Existing verifier has now proved that stateDigests are the only changed frozen scientific field.
m97_ref = candidate
m97_ref["referenceExecution"] = {
    "workflowRunId": 34555842586,
    "artifactId": 10182592750,
    "artifactSha256": "bf3c7d6128e24ff60577ab9493d85301315827b10942239dde9742d55ff8b408",
    "branchHeadSha": "595139a014451154f78e8cda236dd278f8fd4e41",
    "pullRequestMergeRefBuildSha": "a9b3cc80fb3ad111a01ecbe0927c92089f412def",
    "fullAggregateCanonicalSha256": m97_actual["aggregateCanonicalSha256"],
    "modelSemanticsId": "anthrosim-model-semantics-v38",
    "pullRequestMergeRefBuildStatus": "pre-merge-v38-reverification-artifact",
}
m97_ref_path.write_text(json.dumps(m97_ref, separators=(",", ":")) + "\n")
subprocess.run(
    [
        "python3",
        "scripts/verify-m9-aggregation-benchmark-reference.py",
        "--actual",
        str(m97_actual_path),
        "--reference",
        str(m97_ref_path),
    ],
    check=True,
)

m97_doc_path = Path("docs/research/m9-controlled-aggregation-benchmark-result.md")
m97_doc = m97_doc_path.read_text()
assert "**Current machine-readable reference: `anthrosim-model-semantics-v35`.**" in m97_doc
assert "## Current regression reference — model semantics v35" in m97_doc
m97_doc = m97_doc.replace(
    "**Current machine-readable reference: `anthrosim-model-semantics-v35`.**",
    "**Current machine-readable reference: `anthrosim-model-semantics-v38`.**",
    1,
)
m97_doc = m97_doc.replace(
    "## Current regression reference — model semantics v35",
    "## Historical reviewed reference — model semantics v35",
    1,
)
insertion = """## Current regression reference — model semantics v38

Audit-v6 AV6-006/#711 changes the spatial equivalence frame used only when M9 must resolve exactly equal-cost temporary destinations. The frozen M9.7 design was rerun unchanged on production PR #768. The v38 execution preserved the complete strict scientific projection and the independent M9.6 travel-burden reference exactly. Only authoritative terminal state digests changed, as expected from the new persisted M9 tie-policy/schema identity; no aggregate capability endpoint or travel-burden value was rebaselined.

Reviewed v38 execution:

- workflow run: `34555842586`;
- artifact: `10182592750`;
- artifact SHA-256: `bf3c7d6128e24ff60577ab9493d85301315827b10942239dde9742d55ff8b408`;
- reviewed branch head: `595139a014451154f78e8cda236dd278f8fd4e41`;
- pull-request merge-ref build: `a9b3cc80fb3ad111a01ecbe0927c92089f412def`;
- aggregate canonical SHA-256: `840e19aacbd1112aa2477ed29d51c3a9461ccd52d42b9974c0c6ca81ced49408`;
- reference model semantics: `anthrosim-model-semantics-v38`.

The capability conclusion remains **`capability_distinguished`**: all **8/8** paired seeds pass, median focal-person-day difference remains **31 permille**, the maximum remains **36 permille**, median intermittent peak-visitor share remains **432 permille**, and the minimum remains **396 permille**. Every frozen legacy per-seed scientific metric remains identical. The independent M9.6 travel-burden reference also reproduces exactly for every seed, including planned/observed transit, travel cost and route distance. Exact intermittent replay and active annual checkpoint/resume equivalence passed before the reference comparison.

A fail-closed refresh check substituted only the fresh continuous/intermittent terminal `stateDigests` into the v35 legacy reference and then required the repository's unchanged `verify-m9-aggregation-benchmark-reference.py` to pass. That verifier checks benchmark/definition identity, classification, aggregate endpoints, both arm configuration identities, every legacy pair projection and the independently frozen travel-burden/reconciliation fields. The checked reference therefore advances only terminal state digests and execution provenance to v38.

This is a causal state/provenance refresh required by the repaired M9 tie-state semantics, not empirical calibration or archaeological validation.

"""
marker = "## Historical reviewed reference — model semantics v35\n"
assert marker in m97_doc
m97_doc = m97_doc.replace(marker, insertion + marker, 1)
m97_doc_path.write_text(m97_doc)
