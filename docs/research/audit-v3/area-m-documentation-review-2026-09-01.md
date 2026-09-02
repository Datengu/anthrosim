# Audit v3 Area M review — documentation, TRACE/ODD/ODD+D and claim consistency

Immutable discovery target: `v0.3.3` / `d3b6fc0b0750933b57252c9087513d156d88f218` / `anthrosim-model-semantics-v21`.

Discovery only. No finding is repaired.

## Surfaces inspected

Fresh v3 comparison covered current-facing scientific and release documentation against the frozen executable/result artifacts:

- `docs/scientific-model.md`;
- `docs/research/odd.md`;
- `docs/research/odd-d.md`;
- `docs/research/trace.md`;
- `docs/releases/v0.3.3.md`;
- `docs/research/general-scientific-demographic-baseline-v1.md`;
- `docs/research/household-lifecycle-structural-sensitivity-v2.md` and v2 result;
- frozen `ExperimentConfig` / household-lifecycle configuration;
- frozen general-demography confirmatory definition/result.

Known documentation consequences of AV3-006/#410, AV3-013/#423, AV3-014/#427 and AV3-015/#429 remain open. They are not duplicated merely because release/TRACE prose describes the intended fail-closed precision, evidence-role and exact-source contracts.

## Empirical-versus-synthetic and benchmark interpretation

ODD, ODD+D, TRACE, scientific-model and v0.3.3 release documentation consistently state that:

- current human-decision and household/resource mechanisms are synthetic/unvalidated unless a future study evidence-grounds them;
- M8/M9 benchmark success is capability/mechanism verification, not archaeological validation;
- a green build, deterministic replay or ODD description is not scientific validation;
- no current result validates a reconstruction of human prehistory;
- another genuinely fresh audit of immutable v0.3.3 is required before empirical inference.

No new defect was demonstrated on this claim-boundary surface. The current audit's non-clean result reinforces rather than contradicts the release note's stated condition for proceeding.

## ODD / ODD+D model description

ODD and ODD+D explicitly separate residence from temporary presence, synthetic M4 decision rules from claims about real cognition, demographic mechanics from conscious reproductive choice, and absent learning/social institutions from claims that those processes were historically absent. Scheduler and causal-boundary descriptions broadly track current v21 contracts.

No distinct new ODD+D decision-theory claim defect was demonstrated. The normative scientific-model household-lifecycle identity, however, is stale as described below.

## AV3-016 — stale TRACE-linked demographic baseline result

`docs/research/trace.md` currently directs readers to `general-scientific-demographic-baseline-v1.md` as the concrete #304 structural-sensitivity result. That living result page still reports the historical `deterministic_size_fission_v1` / 64-seed confirmation.

The frozen current confirmatory definition/result instead use dependency-aware `deterministic_dependency_fission_v2` and 130 fresh seeds per arm. For the positive schedule, the stale/current comparison includes:

| Quantity | living narrative | frozen current confirmation |
|---|---:|---:|
| seeds per arm | 64 | 130 |
| fission lifecycle | `deterministic_size_fission_v1` | `deterministic_dependency_fission_v2` |
| fixed late growth | -0.002%/yr | -0.081659%/yr |
| fission late growth | -0.994%/yr | -1.028906%/yr |
| fixed mean N240 | 108.2 | 107.307692 |
| fission mean N240 | 28.9 | 20.923077 |
| fission extinction | 4.7% | 3.0769% |
| fission mate limitation | 38.7% | 43.2889% |

The current simple fission-minus-fixed mean N240 difference is **-86.384615** people rather than the narrative's historical **-79.4**. This is AV3-016/#431 (P2). The broad no-universal-demographic-baseline conclusion may remain supported, but the current-facing evidence/treatment identity is stale.

## AV3-017 — normative scientific-model household identity is stale

`docs/scientific-model.md` identifies itself as a current model-semantics-v21 specification but says that post-audit structural-sensitivity experiments can enable explicit `deterministic_size_fission_v1` lifecycle semantics.

Frozen v21 `config.rs` defines the current explicit lifecycle identity as:

`deterministic_dependency_fission_v2`

and the current v2 lifecycle contract explicitly says this dependency-aware treatment supersedes v1. Since v2 replaced v1's record-order-sensitive partitioning, this is a scientifically meaningful model-form mismatch, not just a renamed label. AV3-017/#432 (P2) records the defect.

## TRACE/release safeguard claims under current findings

TRACE and the v0.3.3 release note describe intended safeguards including exact source provenance, fail-closed stochastic precision and machine-auditable evidence-role separation. Fresh audit findings show defects within those claimed contracts:

- AV3-006/#410: same-seed covariance can be ignored by a precision gate;
- AV3-013/#423: evidence aliases can evade source-level independence checks;
- AV3-014/#427: `gitCommit=null` can collapse source-distinct executables to one accepted research identity.

These are cross-cutting consequences of the underlying implementation/analysis defects, not independently counted documentation defects. The release note also explicitly says v0.3.3 does not establish that no defect remains and requires this fresh audit before empirical work, so no additional contradiction issue is inferred solely from the safeguard summaries.

## Historical versus living documentation boundary

Historical lifecycle-v1 contracts/results are legitimate preserved provenance artifacts when treated as historical. The defects arise where current-facing documents present the old treatment/result as the current v21 model/evidence. The v2 lifecycle contract and v2 structural-sensitivity result correctly identify the replacement treatment, demonstrating that the repository already has an explicit historical/current distinction on that narrower surface.

## Area-M disposition

Area M has fresh coverage of executable/scientific-document consistency, ODD/ODD+D, TRACE, release/version statements, empirical-versus-synthetic boundaries, frozen benchmark interpretation, limitations/null assumptions and stale current-facing statements.

New findings:

- AV3-016/#431 (P2): TRACE-linked demographic baseline page reports superseded v1/64-seed evidence as current.
- AV3-017/#432 (P2): current normative scientific-model specification names superseded household-fission v1 semantics.

Known AV3-006/013/014/015 remain cross-cutting documentation limitations. No repair was made.

Area M is complete with findings open. The final pending discovery surface is Area N — cross-system integration.
