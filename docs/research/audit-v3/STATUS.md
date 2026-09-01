# AnthroSim scientific audit v3 — status ledger

Audit target at initialization: immutable AnthroSim `v0.3.3`, tag commit `358ae93b57a9b8f7053575dc6651aa959de2b4f9`, model semantics `anthrosim-model-semantics-v21`.

Protocol: `docs/research/scientific-audit-protocol.md`

Charter: `docs/research/audit-v3/README.md`

Purpose: durable repository-authoritative state for the third independent/adversarial comprehensive scientific audit and the first full convergence audit of the frozen `v0.3.3` baseline.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v3 / third independent scientific audit |
| Immutable discovery target | `v0.3.3` |
| Target tag SHA | `358ae93b57a9b8f7053575dc6651aa959de2b4f9` |
| Initial protected-main SHA | `358ae93b57a9b8f7053575dc6651aa959de2b4f9` |
| Target software version | `0.3.3` |
| Target model semantics | `anthrosim-model-semantics-v21` |
| Required protected-main contexts at initialization | 24 |
| Open issues at initialization | none |
| Open PRs at initialization | none |
| Overlapping audit work at initialization | none observed |
| Current P0 findings | none discovered yet |
| Current P1 findings | none discovered yet |
| Current P2 findings | none discovered yet |
| Current P3 findings | none discovered yet |
| Coverage state | **A–N pending** |
| Audit-v3 convergence classification | **not yet assessable** |
| Closure state | **in progress — initialization only** |

The immutable `v0.3.3` tag is the initial defect-discovery target. Audit documentation and any future repair commits may advance protected `main`; the ledger must distinguish evidence obtained on the frozen release from evidence obtained on a repaired successor head.

Audit v2 is historical context only. Its green Areas, repaired findings, protected CI and preserved benchmarks do not automatically satisfy any audit-v3 Area.

## Coverage matrix

| ID | Audit area | Status | Fresh v3 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **pending** | — |
| B | Demography, fertility, mortality, ageing, population structure | **pending** | — |
| C | Households, kinship, social links, lifecycle structure | **pending** | — |
| D | Resources, condition, subsistence, depletion/recovery | **pending** | — |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **pending** | — |
| F | Aggregation and interaction mechanisms | **pending** | — |
| G | Initialization, burn-in, path dependence, continuation state | **pending** | — |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **pending** | — |
| I | Sensitivity, uncertainty, convergence, robustness | **pending** | — |
| J | Identifiability, equifinality, calibration, discrimination | **pending** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **pending** | — |
| L | Observability, analysis outputs, statistical summaries | **pending** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **pending** | — |
| N | Cross-system integration | **pending** | — |

## Finding register

No audit-v3 scientific finding has yet been demonstrated.

Finding IDs will use `AV3-###` in discovery order and will record severity, Area(s), exact affected baseline, issue, repair PR, semantics/provenance implications, and independent re-verification state.

| Finding | Severity | Area | Status | Issue | Evidence / repair / re-verification |
|---|---|---|---|---|---|
| — | — | — | — | — | — |

## Convergence accounting

The project objective is to achieve a fresh full-scale audit with **no newly discovered P0/P1 finding**.

Audit-v3 outcome rules:

- If v3 completes Areas A–N without discovering any P0/P1, classify it as a **P1-clean convergence pass**.
- If v3 discovers one or more P0/P1 findings, preserve and repair them normally, but classify v3 as a **non-clean convergence pass** even if every finding is later independently reverified as fixed.
- After a non-clean pass, a new frozen repair baseline and a new audit generation from zero coverage are required before claiming the desired clean convergence signal.
- Do not lower severity to improve the convergence classification.

## Freshness requirements for Area completion

Every Area A–N must record:

- authoritative implementation and documentation inspected;
- exact target/head SHA and model-semantics identity used;
- at least one fresh falsification-oriented question or construction;
- tests/experiments and quantitative results where feasible;
- neighbouring-system interactions considered;
- explicit comparison against any relevant audit-v2 repaired contract without using v2 as sole evidence;
- findings/dispositions and unresolved uncertainties;
- whether evidence must be repeated after a material repair.

## Initial repository verification — 2026-09-01

- `refs/tags/v0.3.3` resolves exactly to `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.
- Protected `main` was the same SHA at audit initialization.
- `crates/anthrosim-core/src/provenance.rs` declares `MODEL_SEMANTICS_ID = "anthrosim-model-semantics-v21"`.
- `v0.3.3` release documentation records software version `0.3.3` and model semantics v21.
- The release-candidate line had 24 required protected-main contexts.
- Exact-SHA release verification had passed protected CI, RustSec, manually dispatched M8.6 and M9.7, and the fail-closed named-release tag workflow before audit-v3 initialization.
- Open issues: none.
- Open pull requests: none.
- No overlapping audit-v3 agent/branch was observed before creating the initialization branch.

These facts establish the audit target and release provenance. They are **not** scientific completion evidence for Areas A–N.

## Session / handoff log

### 2026-09-01 — audit-v3 initialization

- Frozen target established: `v0.3.3` / `358ae93b57a9b8f7053575dc6651aa959de2b4f9` / semantics v21.
- Reusable scientific-audit protocol reviewed; its convergence section already defines a fresh no-new-P0/P1 pass as the useful maturity signal.
- Audit-v3 charter created with stricter independence rules: v2 evidence is historical context, not v3 completion evidence; each Area requires fresh adversarial work.
- Coverage deliberately reset to A–N pending.
- No scientific finding has been claimed during initialization.
- Recommended next work: **Area A — authoritative semantics and scheduler behaviour**, starting from the immutable tag and constructing fresh scheduler/order/boundary adversaries before consulting v2 conclusions as regression context.

## Handoff instruction

Read, in order:

1. `docs/research/scientific-audit-protocol.md`
2. `docs/research/audit-v3/README.md`
3. this `STATUS.md`

Then verify live protected `main`, immutable `v0.3.3`, open issues, open PRs and overlapping work. Continue the next incomplete Area from first principles. Do not mark an Area complete solely because audit v2 or CI previously passed it.
