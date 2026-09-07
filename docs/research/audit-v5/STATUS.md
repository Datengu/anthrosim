# AnthroSim scientific audit v5 — status ledger

Audit target: immutable AnthroSim `v0.3.5`, tag commit `e7667af52d48a1ffbae2bf7713a2388e65994b42`, model semantics `anthrosim-model-semantics-v33`.

Protocol: `docs/research/scientific-audit-protocol.md`

Re-verification addendum: `docs/research/audit-reverification-version-drift.md`

Charter: `docs/research/audit-v5/README.md`

Purpose: durable repository-authoritative state for the fifth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v4 convergence pass.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v5 / fifth independent scientific audit |
| Immutable discovery target | `v0.3.5` |
| Target tag SHA | `e7667af52d48a1ffbae2bf7713a2388e65994b42` |
| Target software version | `0.3.5` |
| Target model semantics | `anthrosim-model-semantics-v33` |
| Coverage state | **1/14 Areas complete — Area B in progress** |
| Current P0 findings | none discovered |
| Current P1 findings | **1 — AV5-001 / #606** |
| Current P2 findings | none discovered |
| Current P3 findings | none discovered |
| Current open Audit-v5 findings | **#606 / AV5-001** |
| Convergence classification | **pending full A–N discovery; non-clean because Area A demonstrated AV5-001 (P1)** |
| Repair state | **discovery only; do not repair v5 findings until A–N discovery completes** |
| Empirical readiness implication | **none — Audit v5 does not establish empirical validity or archaeological research readiness for a specific case** |

## Initialization reconstruction

At Audit-v5 initialization on 2026-09-07:

- protected `main` was exactly `e7667af52d48a1ffbae2bf7713a2388e65994b42`;
- immutable tag `v0.3.5` resolved to the same exact commit;
- executable model semantics at that tag were `anthrosim-model-semantics-v33`;
- there were **0 open pull requests** and **0 open issues**;
- no overlapping active Audit-v5 work existed;
- two unprotected historical merged branches remained (`docs/post-v034-documentation-audit-pass-2` and `release/v0.3.5-prep`), but neither represented active or overlapping audit work;
- Scientific Audit v4 was complete and historical, with all 15 frozen-target findings repaired and independently re-verified/dispositioned on the living line before v0.3.5 was frozen.

The repository and this ledger remain authoritative if any of the live-state facts above change after initialization.

## Discovery rules

- The immutable `v0.3.5` tag is the scientific discovery target.
- Audit v2/v3/v4 evidence is historical context and regression-hypothesis material only.
- Each Area starts incomplete and requires fresh evidence.
- Existing v4 regressions and release checks may be reused as controls, but cannot by themselves establish v5 coverage.
- Preserve demonstrated defects in issues and this ledger before repair.
- Use sequential finding identifiers `AV5-001`, `AV5-002`, and so on.
- Continue later Areas against the frozen v0.3.5/v33 baseline even after findings are discovered.
- Production repair is deferred until A–N discovery is complete, absent an explicitly documented repository-integrity emergency.
- Evidence-only adversary PRs should be closed unmerged after classification unless an ordinary production decision separately promotes a test into the permanent regression suite.
- When later remediation advances living software/model/schema/documentation identity, re-verification follows `docs/research/audit-reverification-version-drift.md` rather than falsifying the historical baseline.

## Coverage matrix

| ID | Audit area | Status | Fresh v5 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | PR #605 demonstrated **AV5-001 / #606 (P1)**: isolated-founder/global-coupling locality failure in M9 equal-cost ties. PR #608 quantitatively falsified a material annual-background-mortality risk shift across 1/4/12/365 M3 cadences. PR #610 quantitatively confirmed that an M9 return completed exactly on an M4 boundary becomes immediately M4-visible. Frozen-source review also confirmed explicit M3 → M9 → M4 → annual-M2 fixed-day ordering and symmetric competing-risk attribution. |
| B | Demography, fertility, mortality, ageing, population structure | **in progress** | PR #612 quantified focal fertility stream displacement under an added separate-cell reproductive pair (`523/1024` same-seed focal outcomes changed), but **no finding**: the documented Monte Carlo contract explicitly does not promise per-agent common-random-number coupling across structurally different arms. Fresh promised-invariant attacks still required. |
| C | Households, kinship, social links, lifecycle structure | **incomplete** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete** | AV5-001 cross-cutting evidence only; Area E not yet independently audited |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 is a documented same-seed coupling limitation to revisit when Area H independently audits inference/coupling semantics. |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 documents why per-agent same-seed invariance cannot be assumed for structural arms. |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete** | AV5-001 is an early cross-system composition finding; Area N remains incomplete until the explicit integration pass |

## Finding register

| Finding | Severity | Area | Issue | Discovery status | Remediation / re-verification status |
|---|---|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N cross-cutting | #606 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #605, exact head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; focal key `1 -> 2`, destination divergence `503/1024` tie seeds | **open and unrepaired during discovery** |

## Session log

### 2026-09-07 — Audit v5 initialization

- Frozen target selected: `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / semantics v33.
- Confirmed `main` and `v0.3.5` resolve to the same exact commit at initialization.
- Reusable scientific audit protocol and post-discovery version-drift addendum reviewed.
- Audit-v4 charter and final ledger reviewed as historical context only.
- Audit-v5 coverage initialized at zero; no prior coverage inherited.
- No open issue/PR overlap found at initialization.
- Recommended first substantive area: **Area A — authoritative semantics and scheduler behaviour**.

### 2026-09-07 — Area A pass 1: global coupling locality × M9 equal-cost ties

- Evidence-only PR #605 exact head `b57ac276e07f89fb3179ad585a586b5be60e150c` demonstrated that an M9-unreachable isolated founder changed the unchanged focal persisted stochastic-coupling rank from `1` to `2` and changed the focal equal-cost destination in **503/1024** tie seeds.
- Dedicated workflow run `34167265827`, job `101880669781` reached the intended scientific assertion after successful build.
- Duplicate search distinguished this from historical AV4-007/#500 and #324.
- Finding preserved as **AV5-001 / #606, P1** before any production repair.

### 2026-09-07 — Area A pass 2: annual background mortality × M3 cadence

- Evidence-only PR #608 exact head `6858b5ee721253c6097faf24b79c0447c12d48c1` ran 8,192 one-year seeds per arm at 1/4/12/365 M3 periods/year with annual background mortality `500,000/1,000,000` and other causal mechanisms neutralized.
- Dedicated run `34167878417`, job `101882427621`: deaths were `4163`, `4082`, `4141`, `4197`; empirical annual risks **50.818%**, **49.829%**, **50.549%**, **51.233%**; max spread **1.404 percentage points**.
- **Disposition: no finding** for material one-year annual-risk shift under the tested constant-hazard configuration.

### 2026-09-07 — Area A pass 3: M9 return completion × coincident M4 boundary

- Evidence-only PR #610 tested a day-182 return completion exactly coincident with the second M4 boundary.
- Initial run `34168320489` / job `101883665488` was a harness-only compile failure and never reached the scientific oracle.
- Corrected evidence head `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`, run `34170168801`, job `101888814237`: **success**. Four households produced baseline M4 evaluations `16`, active-M9 evaluations `12`, and four day-182 journey completions.
- **Disposition: no finding**; completed returns are immediately visible to coincident M4 as documented.

### 2026-09-07 — Area A completion assessment

- Frozen executable scheduler and living documentation agree on fixed-day order: elapsed M3 settlement, M9 temporary transitions, M4 permanent migration, then annual M2 after the subannual loop.
- Frozen competing-mortality implementation has separate latent triggers, symmetric dual-trigger attribution, exact cause-swap controls and large-sample union-risk checks.
- Fresh v5 evidence covered arbitrary shared coupling/tie identity, update-frequency dependence and same-day state-boundary composition.
- **Area A complete, non-clean:** AV5-001 / #606 remains an open unrepaired P1; PR #608 and #610 were quantitative no-finding results.

### 2026-09-07 — Area B pass 1: remote fertility candidate × focal same-seed realization

- Reconstructed state after Area-A closure: protected `main` `180ec0626de705a84a35d0b436ef1a9e42a48e7f`; no open PRs before the evidence branch; immutable target remained `v0.3.5` / `e7667af...` / v33. Changes on living `main` since the tag were Audit-v5 documentation only, so executable production semantics remained the frozen v33 target.
- Source inspection confirmed that M2 freezes eligible females, orders them by persisted stochastic-coupling rank, and consumes one shared sequential `demography/fertility` stream. Parentage occupancy remains residence/cell-local.
- Evidence-only PR #612 exact head `2018921f188f312567b984b8ad4403e8e00196cd` kept focal `PersonId(1)` female and `PersonId(2)` male unchanged at household 1 / cell 2 / age 30 / condition 1000. The augmented arm appended only an older 40-year-old female+male pair in household 2 / cell 1. Migration, resource demand and all mortality were disabled; annual fertility was fixed at 0.5.
- Dedicated workflow run `34170606978`, job `101890040711`, pinned Rust 1.97.1 compiled `anthrosim-core v0.3.5` successfully and reached the intended assertion. Exact result: **523/1024** identical-seed focal female birth outcomes differed after the remote pair was added. First divergences included seeds `1,5,6,7,10,12,15,16,17,20,21,24`.
- The observed effect is consistent with the added eligible female consuming an earlier draw from the shared sequential fertility stream and shifting the focal female to a later stream position.
- **Disposition: no defect / documented coupling limitation, not AV5-002.** AnthroSim's current Monte Carlo contract explicitly states that `paired_mean_difference` is replicate-level seed pairing when justified and **does not claim per-agent common-random-number counterfactual coupling or alter simulator RNG semantics**. Therefore the test's stronger focal-agent locality assertion is not an existing model contract. The result is scientifically useful because it quantifies that limitation and prevents same-seed structural comparisons from being misread as focal-agent counterfactual identity.
- This is also not a replay or regression of AV4-001/#486. That historical P1 concerned pure relabelling of a scientifically identical founder state; v33 preserves the corresponding label-invariance regression. PR #612 changes the represented scientific population by adding an eligible remote reproductive pair and therefore tests a different, explicitly unpromised coupling property.
- Area B remains **in progress**. The next adversary must test a promised demographic invariant rather than strengthening the stochastic coupling contract implicitly.

## Next action

Continue **Area B** against immutable `v0.3.5` / v33 with a fresh promised-invariant attack. Priority: construct a one-cell/local-pair case where an eligible male is subject to certain background mortality exactly at the day-365 M3/M2 boundary and verify that the death occurs before fertility/parentage, preventing that dead male from authorizing a same-day birth; include a matched surviving-male control. Continue with ageing/newborn/finite-population limiting cases as needed before Area-B closure. Do **not** repair #606 during discovery.
