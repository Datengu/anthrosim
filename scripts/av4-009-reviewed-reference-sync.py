from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path

REVIEW_HEAD = "25c9a11dce8052fecfbb339114a4ba1c8da00b0c"
EXPECTED_HELPERS = {
    ".github/workflows/av4-009-reviewed-reference-sync.yml",
    "scripts/av4-009-reviewed-reference-sync.py",
}

changed = set(
    subprocess.check_output(
        ["git", "diff", "--name-only", f"{REVIEW_HEAD}..HEAD"], text=True
    ).splitlines()
)
if changed != EXPECTED_HELPERS:
    raise SystemExit(f"unexpected changes since reviewed head: {sorted(changed)}")

# #304: use the exact generated expected scientific result from reviewed run 33884799723.
root304 = Path("/tmp/review304")
candidates304 = list(root304.rglob("expected-result.json"))
if len(candidates304) != 1:
    raise SystemExit(f"expected exactly one #304 expected-result.json, found {candidates304}")
result304 = json.loads(candidates304[0].read_text())
if result304.get("modelSemanticsId") != "anthrosim-model-semantics-v33":
    raise SystemExit("#304 reviewed result is not v33")
if result304.get("recommendation") != "no_universal_demographic_baseline":
    raise SystemExit("#304 recommendation unexpectedly changed")
if result304.get("runCount") != 780:
    raise SystemExit("#304 reviewed result does not contain 780 runs")
Path("research/general-demography-baseline-v1/confirmatory-result.json").write_text(
    json.dumps(result304, indent=2, sort_keys=True) + "\n"
)

# M8.6: reconstruct the preserved compact reference from the reviewed aggregate.
root8 = Path("/tmp/reviewM8")
candidates8 = list(root8.rglob("benchmark-summary.json"))
if len(candidates8) != 1:
    raise SystemExit(f"expected exactly one M8 benchmark-summary.json, found {candidates8}")
actual = json.loads(candidates8[0].read_text())
old_path = Path("examples/m8-first-evidence-grounded-benchmark/reference-result.json")
old = json.loads(old_path.read_text())
if actual.get("schemaVersion") != old.get("schemaVersion") or actual.get("benchmarkId") != old.get("benchmarkId"):
    raise SystemExit("M8 reviewed aggregate identity mismatch")
classification = actual.get("classification")
expected_classification = {
    "benchmarkClass": "fragile_spatial_structure",
    "degenerateArms": [],
    "robustMetrics": [],
    "fragileMetrics": ["terminalLargestCellSharePermille"],
}
if classification != expected_classification:
    raise SystemExit(f"unexpected reviewed M8 classification: {classification}")

primary = {}
for name, metric in actual["primaryMetrics"].items():
    entry = {
        "classification": metric["classification"],
        "robustCriteria": metric["robustCriteria"],
    }
    for arm in ("weak", "moderate", "strong"):
        entry[arm] = metric["comparisonsToFlat"][arm]
    primary[name] = entry

arms = {}
for arm_name in ("flat", "weak", "moderate", "strong"):
    arm = actual["arms"][arm_name]
    runs = arm["runs"]
    if len(runs) != 8:
        raise SystemExit(f"M8 {arm_name} does not contain 8 runs")
    spatial_ids = {r["spatialConfigIdentity"] for r in runs.values()}
    if len(spatial_ids) != 1:
        raise SystemExit(f"M8 {arm_name} spatial identities diverged")
    arms[arm_name] = {
        "experimentId": arm.get("experimentId"),
        "mechanismsCanonicalSha256": arm["mechanismsCanonicalSha256"],
        "spatialConfigIdentity": next(iter(spatial_ids)),
        "terminalDegenerateRuns": arm["terminalDegenerateRuns"],
        "runStateDigest64": {seed: run["stateDigest64"] for seed, run in sorted(runs.items())},
    }

source = actual.get("source", old.get("source", {}))
source = dict(source)
source["modelSemanticsId"] = "anthrosim-model-semantics-v33"
reference8 = {
    "schemaVersion": actual["schemaVersion"],
    "benchmarkId": actual["benchmarkId"],
    "referenceExecution": {
        "workflowRunId": 33884800100,
        "artifactId": 9941586776,
        "artifactSha256": "21b2f9dacdbf85c5b036bae7ca90d158cad5257a67cca1dae66bc17e54e9f9ba",
        "branchHeadSha": REVIEW_HEAD,
        "pullRequestMergeRefBuildSha": "8251e33388a74ab858748062d40ba2131fca1477",
        "fullAggregateCanonicalSha256": "adf2033e68b5620ef7eb328b0ca5daea2951d3cf670d0cde19e486c06a43d97d",
        "pullRequestMergeRefBuildStatus": "pre-merge-v33-reverification-artifact",
    },
    "source": source,
    "declaredSeeds": actual["declaredSeeds"],
    "classification": classification,
    "primaryMetrics": primary,
    "arms": arms,
}
old_path.write_text(json.dumps(reference8, separators=(",", ":")) + "\n")

# Record the scientific review without mutating historical v32 evidence.
memo = Path("research/general-demography-baseline-v1/model-semantics-v33-reverification.md")
memo.write_text("""# Model-semantics v33 re-verification\n\nAudit-v4 AV4-009 changes M4 spatial-candidate stochastic coupling while preserving the declared demographic controls. The first v33 abstention candidate was rejected because it collapsed all six confirmatory arms to extinction. The accepted equivalence-class sampler was therefore re-run independently before any reference update.\n\n- reviewed production head: `25c9a11dce8052fecfbb339114a4ba1c8da00b0c`\n- issue #304 review: run `33884799723`, job `101061843958`, artifact `9941547662`, artifact SHA-256 `dc439b48bb7d2a048c3fe2365698d2403a67ac2a26e340c17bb6dc8a8901fa83`\n- all 780 confirmatory runs completed\n- all three predeclared Monte Carlo precision gates returned `sufficient_stop`\n- recommendation remains `no_universal_demographic_baseline`\n- long-run environment and initialization dependence remain false; stochastic multi-regime context count remains 2\n\nThe numerical shifts are consistent with an M4 residence-coupling change rather than a demographic-mechanism rewrite. In particular, positive-growth/fission late growth remains about -1.07%/year while terminal population and extinction move from the v32 realization; positive-growth/fixed remains non-extinct. Historical v32 and earlier outputs remain evidence for their original semantics and are not relabelled.\n\nM8.6 was also reviewed on run `33884800100`, job `101062379280`, artifact `9941586776` (SHA-256 `21b2f9dacdbf85c5b036bae7ca90d158cad5257a67cca1dae66bc17e54e9f9ba`). All 32 runs completed with no degenerate arms. The benchmark remains `fragile_spatial_structure`; `terminalLargestCellSharePermille` remains fragile, while `terminalPopulationHerfindahlPerMillion` changes from fragile to not-distinctive under v33. M9.7 on the same workflow preserved its canonical scientific reference and exact replay/checkpoint-resume contracts.\n""")
