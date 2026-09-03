from __future__ import annotations

import json
import re
from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    Path(path).write_text(text, encoding="utf-8")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise SystemExit(f"missing expected {label}: {old!r}")
    return text.replace(old, new, 1)


for path in ("docs/scientific-model.md", "docs/research/odd.md", "docs/research/odd-d.md"):
    text = read(path)
    text = replace_once(text, "current model semantics v28", "current model semantics v29", path)
    write(path, text)

path = "docs/scientific-model.md"
text = read(path)
marker = "Parent eligibility likewise uses residence rather than visitor co-presence.\n"
paragraph = (
    "Under v29, each residence cell's living male parentage candidates are ordered by their "
    "persisted person `stochastic_coupling_rank` before the existing age/sex eligibility filter "
    "and uniform `demography/parentage` reservoir sample are consumed. Canonical `PersonId` or "
    "packed-record order is therefore not a causal parent-selection input; locality, eligibility, "
    "uniform selection probability and the independent parentage RNG stream are unchanged.\n"
)
if paragraph not in text:
    text = replace_once(text, marker, marker + "\n" + paragraph, "scientific-model parentage marker")
write(path, text)

path = "docs/research/odd.md"
text = read(path)
marker = (
    "Households are persistent resource-sharing and mobility units. They are not asserted to be tribes, clans, "
    "lineages, marriages or universal nuclear families. Living household members normally share persistent "
    "residence; newborns join the female parent's household. M4 permanent migration moves the living household "
    "as a unit. M9 temporary mobility moves the household through a temporary journey state while preserving residence.\n"
)
paragraph = (
    "Under v29, residence-local eligible male parentage candidates are coupled to the independent "
    "`demography/parentage` stream through persistent person stochastic-coupling rank rather than canonical "
    "`PersonId`/record order; the existing uniform reservoir-selection rule is otherwise unchanged.\n"
)
if paragraph not in text:
    text = replace_once(text, marker, marker + "\n" + paragraph, "ODD household marker")
write(path, text)

result = json.loads(read("research/general-demography-baseline-v1/confirmatory-result.json"))
if result["modelSemanticsId"] != "anthrosim-model-semantics-v29":
    raise SystemExit("#304 machine result is not v29")
if result["recommendation"] != "no_universal_demographic_baseline" or result["runCount"] != 780:
    raise SystemExit("unexpected #304 scientific conclusion/design")

path = "docs/research/general-scientific-demographic-baseline-v1.md"
text = read(path)
confirmation = "\n".join(
    [
        "Current confirmation:",
        "",
        "- fixed household control: `fixed_founder_v1`;",
        "- structural treatment: `deterministic_dependency_fission_v2`;",
        "- fresh process seeds per arm: **130** (`3042001..3042130`);",
        "- design: **3 demography schedules × 2 household-lifecycle arms × 130 seeds = 780 completed runs**;",
        "- founder age ceiling: **60 years**;",
        "- resource productivity: **1000 permille**;",
        "- current model semantics: `anthrosim-model-semantics-v29`;",
        f'- current preserved research execution identity: `{result["researchId"]}`.',
        "",
        "The v29 causal semantics change in Audit-v4 AV4-005/#495 removes arbitrary canonical male-person ordering from M2 parentage RNG assignment while preserving the study's high-level conclusion. The unchanged confirmatory design was reviewed under v29 in issue-304 workflow run `33813558679`, job `100840609676` (artifact `9915805924`, SHA-256 `607dbdf2e86db582fe7b519c1bf9ea1ad8d69ba02ffc282f934c4f5d4240d45c`); all 780 runs completed and all three predeclared Monte Carlo precision gates returned `sufficient_stop`. Historical v25, v26, v27 and v28 results remain immutable provenance rather than being relabelled as v29 evidence.",
    ]
)
text, n = re.subn(
    r"Current confirmation:\n\n.*?\n\nThe numerical table below",
    confirmation + "\n\nThe numerical table below",
    text,
    count=1,
    flags=re.S,
)
if n != 1:
    raise SystemExit("failed to replace #304 confirmation block")

arms = {(a["demography"], a["householdLifecycle"]): a for a in result["arms"]}
order = (
    ("negative_growth_control_v1", "deterministic_dependency_fission_v2"),
    ("negative_growth_control_v1", "fixed_founder_v1"),
    ("positive_growth_control_v1", "deterministic_dependency_fission_v2"),
    ("positive_growth_control_v1", "fixed_founder_v1"),
    ("replacement_control_v1", "deterministic_dependency_fission_v2"),
    ("replacement_control_v1", "fixed_founder_v1"),
)
rows = [
    "| Demography | Household lifecycle | Late growth %/yr (95% MC CI) | Mean N240 | Extinction | Mate limitation |",
    "|---|---|---:|---:|---:|---:|",
]
for key in order:
    arm = arms[key]
    g = arm["lateGrowthRatePerYear"]
    e = arm["extinction"]
    m = arm["mateLimitationFraction"]
    rows.append(
        f'| `{key[0]}` | `{key[1]}` | {100*g["mean"]:.3f} [{100*g["ci95Lower"]:.3f}, {100*g["ci95Upper"]:.3f}] | '
        f'{arm["terminalPopulation"]["mean"]:.1f} | {100*e["estimate"]:.1f}% | {100*m["mean"]:.1f}% |'
    )
table = "\n".join(rows)
text, n = re.subn(
    r"\| Demography \| Household lifecycle \| Late growth %/yr \(95% MC CI\) \| Mean N240 \| Extinction \| Mate limitation \|\n.*?\n\n## Same-seed household-lifecycle effects",
    table + "\n\n## Same-seed household-lifecycle effects",
    text,
    count=1,
    flags=re.S,
)
if n != 1:
    raise SystemExit("failed to replace #304 numerical table")

paired = {p["demography"]: p for p in result["pairedHouseholdEffects"]}
bullets = "\n".join(
    f'- `{name}`: fission-minus-fixed mean N240 = **{paired[name]["fissionMinusFixedTerminalPopulation"]["mean"]:.1f} people** across 130 same-seed pairs.'
    for name in ("negative_growth_control_v1", "positive_growth_control_v1", "replacement_control_v1")
)
text, n = re.subn(
    r"The current paired summary represents every declared fixed-versus-v2 same-seed contrast: \*\*3 groups × 130 pairs = 390/390 contrasts\*\*\.\n\n.*?\n\nThese paired effects",
    "The current paired summary represents every declared fixed-versus-v2 same-seed contrast: **3 groups × 130 pairs = 390/390 contrasts**.\n\n"
    + bullets
    + "\n\nThese paired effects",
    text,
    count=1,
    flags=re.S,
)
if n != 1:
    raise SystemExit("failed to replace #304 paired effects")
text = text.replace("The paired v28 effects", "The paired v29 effects", 1)

pos_fixed = arms[("positive_growth_control_v1", "fixed_founder_v1")]
pos_fiss = arms[("positive_growth_control_v1", "deterministic_dependency_fission_v2")]
interpretation = (
    "The experiments separate **intrinsic demographic tendency** from **realized population growth**. "
    "Even the positive intrinsic schedule does not remain approximately stationary once the dependency-aware "
    "household-fission treatment is enabled: "
    f'mean late realized growth changes from {100*pos_fixed["lateGrowthRatePerYear"]["mean"]:.3f}%/year '
    f'under fixed-founder households to {100*pos_fiss["lateGrowthRatePerYear"]["mean"]:.3f}%/year under '
    f'`deterministic_dependency_fission_v2`, while mate limitation rises from '
    f'{100*pos_fixed["mateLimitationFraction"]["mean"]:.1f}% to '
    f'{100*pos_fiss["mateLimitationFraction"]["mean"]:.1f}% and mean N240 falls from '
    f'{pos_fixed["terminalPopulation"]["mean"]:.1f} to {pos_fiss["terminalPopulation"]["mean"]:.1f}.'
)
text, n = re.subn(
    r"The experiments separate \*\*intrinsic demographic tendency\*\* from \*\*realized population growth\*\*\..*?\n\nThis does not by itself prove",
    interpretation + "\n\nThis does not by itself prove",
    text,
    count=1,
    flags=re.S,
)
if n != 1:
    raise SystemExit("failed to replace #304 interpretation")

lr = result["longRun"]
c = lr["primaryClassificationCounts"]
longrun = (
    f'The v29 long-run analysis still detects multiple stable regimes, with primary classifications '
    f'`drifting={c.get("drifting", 0)}`, `insufficient_data={c.get("insufficient_data", 0)}`, '
    f'`stable={c.get("stable", 0)}`; {lr["stochasticMultiRegimeContextCount"]} treatment contexts meet the '
    "stochastic multi-regime criterion. Environment-dependence and initialization-dependence flags remain false. "
    "These diagnostics reinforce the same design conclusion: there is no defensible universal demographic baseline "
    "hidden in the current synthetic controls."
)
text, n = re.subn(
    r"The v28 long-run analysis still detects multiple stable regimes,.*?current synthetic controls\.",
    longrun,
    text,
    count=1,
    flags=re.S,
)
if n != 1:
    raise SystemExit("failed to replace #304 long-run paragraph")
text = text.replace("rather than being relabelled as v28 output.", "rather than being relabelled as v29 output.", 1)
write(path, text)

path = "docs/research/m8-first-evidence-grounded-benchmark-result.md"
text = read(path)
text = text.replace(
    "## Current regression reference — model semantics v28",
    "## Historical reviewed reference — model semantics v28",
    1,
)
marker = "The current machine-readable reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json`. Earlier exact references remain preserved in Git history.\n\n"
section = "\n".join(
    [
        "## Current regression reference — model semantics v29",
        "",
        "Audit-v4 AV4-005 / #495 removes arbitrary canonical male-person ordering from M2 parentage RNG assignment while preserving locality, age/sex eligibility, uniform reservoir selection and the separate `demography/parentage` RNG stream. Because genealogy can propagate into household, kin and migration histories, the frozen M8.6 experiment was rerun unchanged and reviewed before its reference was replaced.",
        "",
        "Reviewed v29 execution:",
        "",
        "- evidence-generating workflow run: `33813559006`, job `100840645788`;",
        "- evidence-generating production head: `0e69401b82c512c1f66d15303bc98a8dc75da7e5`;",
        "- artifact: `9915797546`;",
        "- artifact SHA-256: `27cf02539a53a4a21e2cd13e223f70ae00166ecc6972441500447d9248f52ef3`;",
        "- aggregate canonical SHA-256: `978ed2342509d9cbca1a647055f1d794ba513bcb1fdaee01fd26f5e6c7ed4b44`;",
        "- model semantics: `anthrosim-model-semantics-v29`;",
        "- subsequent checked-reference verification: applicable-gates run `33815769032` passed both the canonical comparison and tamper rejection.",
        "",
        "All **32/32** declared runs completed and all four arms remained non-degenerate. The overall benchmark class remains **`fragile_spatial_structure`**. The v29 primary results are:",
        "",
        "| Primary metric | v29 result | Strong-vs-flat median absolute paired effect | Strong paired signs (+ / - / 0) |",
        "| --- | --- | ---: | ---: |",
        "| total migration distance | not distinctive | 6.96% | 6 / 2 / 0 |",
        "| cell-time occupied | not distinctive | 2.02% | 3 / 5 / 0 |",
        "| terminal population Herfindahl | **fragile** | **14.17%** | 4 / 4 / 0 |",
        "| terminal largest-cell share | **fragile** | **33.47%** | 5 / 3 / 0 |",
        "",
        "Relative to v28, terminal largest-cell share crosses from robust back to **fragile** under the unchanged predeclared criteria; Herfindahl remains fragile, while migration distance and cell-time occupancy remain not distinctive. This threshold/classification change is retained as a legitimate downstream consequence of corrected genealogy stochastic coupling rather than tuned away. Terrain inputs, evidence identity and `anthrosim-spatial-transform-semantics-v3` are unchanged, so this remains a result about the declared terrain-only null model, not archaeological validation.",
        "",
        "",
    ]
)
if "## Current regression reference — model semantics v29" not in text:
    text = replace_once(text, marker, marker + section, "M8 current-reference marker")
write(path, text)

path = "docs/research/m9-controlled-aggregation-benchmark-result.md"
text = read(path)
text = text.replace(
    "## Model-semantics v28 applicability re-verification — machine reference unchanged",
    "## Historical model-semantics v28 applicability re-verification — machine reference unchanged",
    1,
)
text = text.replace(
    "## Current regression reference — model semantics v27",
    "## Historical reviewed reference — model semantics v27",
    1,
)
marker = "**Current classification:** `capability_distinguished`\n\n"
section = "\n".join(
    [
        "## Current regression reference — model semantics v29",
        "",
        "Audit-v4 AV4-005 / #495 changes which scientifically represented eligible male receives a same-seed M2 parentage realization. The unchanged M9.7 design was therefore rerun and reviewed rather than assuming that disabled permanent migration made genealogy causally irrelevant.",
        "",
        "Reviewed v29 execution:",
        "",
        "- evidence-generating workflow run: `33813559006`, job `100840645688`;",
        "- evidence-generating production head: `0e69401b82c512c1f66d15303bc98a8dc75da7e5`;",
        "- artifact: `9915799402`;",
        "- artifact SHA-256: `8e14622e26728c6e4a300c6c834c6085aa9c2e84013704f237e4aaa4a1221a4c`;",
        "- aggregate canonical SHA-256: `be17795b0ed35aba0c39a6c76b1d45934dd165d75199551464dcbdc589c9294b`;",
        "- reference model semantics: `anthrosim-model-semantics-v29`;",
        "- subsequent checked-reference verification: applicable-gates run `33815769032` passed the canonical comparison, replay/resume checks and tamper rejection.",
        "",
        "The capability conclusion remains **`capability_distinguished`**: all **8/8** paired seeds pass, median focal-person-day difference remains **31 permille**, the maximum remains **36 permille**, median intermittent peak-visitor share remains **432 permille**, and the minimum remains **396 permille**. Exact intermittent replay and active annual checkpoint/resume remain equivalent.",
        "",
        "Unlike the v28 applicability result, the v29 machine reference does change: continuous and intermittent terminal state digests change because parentage/genealogy is authoritative state even when permanent M4 migration is disabled. The paired temporary-mobility capability metrics remain unchanged, so the rebaseline records the corrected complete model state without strengthening the archaeological claim.",
        "",
        "",
    ]
)
if "## Current regression reference — model semantics v29" not in text:
    text = replace_once(text, marker, marker + section, "M9 current-class marker")
write(path, text)

reverify = "\n".join(
    [
        "# Model-semantics v29 reverification",
        "",
        "Audit-v4 AV4-005 / issue #495 removes arbitrary canonical male `PersonId`/record ordering from M2 parentage RNG assignment. Residence-local living male candidates are now ordered by persistent person stochastic-coupling rank before the existing eligibility filter and uniform `demography/parentage` reservoir sample. Locality, age/sex eligibility, stream separation and uniform selection are unchanged.",
        "",
        "Because this changes which represented kin role receives a same-seed parentage realization, the current remediation line advances from `anthrosim-model-semantics-v28` to `anthrosim-model-semantics-v29`; checkpoint schema advances from 16 to 17.",
        "",
        "## Permanent regression",
        "",
        "`crates/anthrosim-core/tests/parentage_label_invariance.rs` covers the original-style 1,000-seed genealogy-preserving male-label swap, a 256-seed three-role cyclic relabel, and a 256-seed two-year propagation check. The pinned Rust 1.97.1 production regression passed all three before final-candidate assembly.",
        "",
        "## Frozen scientific surfaces reviewed",
        "",
        "### Issue #304 demographic confirmation",
        "",
        "The unchanged 3 × 2 × 130 design completed all **780** runs under v29 and all three predeclared precision gates returned `sufficient_stop`. The recommendation remains `no_universal_demographic_baseline`. Reviewed run `33813558679`, job `100840609676`, artifact `9915805924`, SHA-256 `607dbdf2e86db582fe7b519c1bf9ea1ad8d69ba02ffc282f934c4f5d4240d45c`. "
        f'The checked-in current research identity is `{result["researchId"]}`.',
        "",
        "### M8.6 terrain null model",
        "",
        "Reviewed applicable-gates run `33813559006`, job `100840645788`, artifact `9915797546`, SHA-256 `27cf02539a53a4a21e2cd13e223f70ae00166ecc6972441500447d9248f52ef3`. The overall class remains `fragile_spatial_structure`; both terminal Herfindahl and terminal largest-cell share are fragile under v29. Aggregate canonical SHA-256: `978ed2342509d9cbca1a647055f1d794ba513bcb1fdaee01fd26f5e6c7ed4b44`.",
        "",
        "### M9.7 controlled aggregation",
        "",
        "Reviewed applicable-gates run `33813559006`, job `100840645688`, artifact `9915799402`, SHA-256 `8e14622e26728c6e4a300c6c834c6085aa9c2e84013704f237e4aaa4a1221a4c`. All 8/8 paired criteria, exact replay and active checkpoint/resume remain green and the class remains `capability_distinguished`; authoritative terminal state digests change under v29. Aggregate canonical SHA-256: `be17795b0ed35aba0c39a6c76b1d45934dd165d75199551464dcbdc589c9294b`.",
        "",
        "These are reviewed upstream-semantics rebaselines, not new empirical validation. Historical v25–v28 results remain bound to their original semantics in Git history and the living result documents.",
        "",
    ]
)
write("research/general-demography-baseline-v1/model-semantics-v29-reverification.md", reverify)
