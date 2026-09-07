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
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | PR #605 demonstrated **AV5-001 / #606 (P1)**. PR #608 falsified a material annual-background-mortality risk shift across 1/4/12/365 M3 cadences. PR #610 confirmed that an M9 return completed exactly on an M4 boundary becomes immediately M4-visible. Frozen-source review confirmed M3 → M9 → M4 → annual-M2 fixed-day ordering and symmetric competing-risk attribution. |
| B | Demography, fertility, mortality, ageing, population structure | **in progress** | PR #612 quantified a documented same-seed structural-coupling limitation (`523/1024` focal outcomes changed; no defect). PR #614 confirmed survival-conditioned parentage at the exact day-365 M3/M2 boundary: `64/64` surviving-male controls produced one birth, while `64/64` certain-male-death arms produced one day-365 death and zero births. Fresh ageing/newborn or finite-population limiting-case coverage still required. |
| C | Households, kinship, social links, lifecycle structure | **incomplete** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete** | AV5-001 cross-cutting evidence only; Area E not yet independently audited |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 is a documented same-seed coupling limitation to revisit during independent Area-H inference/coupling audit. |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 documents why per-agent same-seed invariance cannot be assumed for structural arms. |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | — |
| N | Cross-system integration | **incomplete** | AV5-001 is early cross-system composition evidence; Area N remains incomplete until its explicit pass. |

## Finding register

| Finding | Severity | Area | Issue | Discovery status | Remediation / re-verification status |
|---|---|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N cross-cutting | #606 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #605, exact head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; focal key `1 -> 2`, destination divergence `503/1024` tie seeds | **open and unrepaired during discovery** |

## Session log

### 2026-09-07 — Audit v5 initialization

- Frozen target selected: `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / semantics v33.
- Confirmed `main` and `v0.3.5` resolved to the same exact commit at initialization.
- Reusable scientific audit protocol and post-discovery version-drift addendum reviewed.
- Audit-v4 charter and final ledger reviewed as historical context only; no v4 coverage inherited.
- No open issue/PR overlap found at initialization.

### 2026-09-07 — Area A pass 1: global coupling locality × M9 equal-cost ties

- Evidence-only PR #605 exact head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`, demonstrated that an M9-unreachable isolated founder changed the unchanged focal coupling rank `1 -> 2` and changed its equal-cost destination in **503/1024** tie seeds.
- Duplicate search distinguished this from historical AV4-007/#500 and #324.
- Preserved as **AV5-001 / #606, P1** before repair.

### 2026-09-07 — Area A pass 2: annual background mortality × M3 cadence

- PR #608 head `6858b5ee721253c6097faf24b79c0447c12d48c1`, run `34167878417`, job `101882427621`, ran 8,192 one-year seeds per arm at 1/4/12/365 M3 periods/year with annual background mortality `500,000/1,000,000` and other mechanisms neutralized.
- Deaths: `4163`, `4082`, `4141`, `4197`; risks **50.818%**, **49.829%**, **50.549%**, **51.233%**; max spread **1.404 percentage points**.
- **Disposition: no finding.**

### 2026-09-07 — Area A pass 3: M9 return completion × coincident M4 boundary

- PR #610 tested day-182 return completion exactly coincident with the second M4 boundary.
- Initial run `34168320489` / job `101883665488` was a harness-only compile failure; no scientific assertion executed.
- Corrected head `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`, run `34170168801`, job `101888814237`: success. Four households gave baseline M4 evaluations `16`, active-M9 evaluations `12`, and four day-182 journey completions.
- **Disposition: no finding.**

### 2026-09-07 — Area A completion assessment

- Frozen executable scheduler and living documentation agree on fixed-day order: elapsed M3 settlement, M9 temporary transitions, M4 permanent migration, then annual M2 after the subannual loop.
- Frozen competing-mortality implementation has separate latent triggers, symmetric dual-trigger attribution, exact cause-swap controls and large-sample union-risk checks.
- **Area A complete, non-clean:** AV5-001 / #606 remains open/unrepaired; PR #608 and #610 were quantitative no-findings.

### 2026-09-07 — Area B pass 1: remote fertility candidate × focal same-seed realization

- After Area-A closure, protected `main` was `180ec0626de705a84a35d0b436ef1a9e42a48e7f`; immutable target remained v0.3.5/v33 and living changes since the tag were Audit-v5 documentation only.
- PR #612 head `2018921f188f312567b984b8ad4403e8e00196cd` kept focal female `PersonId(1)` and male `PersonId(2)` unchanged in household 1 / cell 2 and appended only an older separate-cell female+male pair in household 2 / cell 1. Migration, resource demand and mortality were disabled; annual fertility was 0.5.
- Run `34170606978`, job `101890040711`, compiled successfully and reached its oracle: **523/1024** identical-seed focal female birth outcomes changed after the remote pair was added.
- **Disposition: no defect / documented coupling limitation, not AV5-002.** The Monte Carlo contract explicitly says same-seed paired comparisons do not claim per-agent common-random-number counterfactual coupling. This is distinct from repaired AV4-001/#486 pure relabelling invariance.

### 2026-09-07 — Area B pass 2: certain day-365 male mortality × same-day parentage

- Evidence-only PR #614 exact head `828b143adbb7ebbc831dd265eeb3557444e506fd` tested a promised survival-conditioning invariant at the exact coincident M3/M2 boundary.
- Controlled state: one 30-year-old female plus one 40-year-old male in the same household/cell; fertility `1,000,000/1,000,000`; M3 cadence `1/year`; zero resource need and condition/scarcity mortality; migration disabled; one-year horizon.
- Matched control set all background mortality to zero. The mortality arm set age `<35` mortality to zero and age `>=35` annual background mortality to `1,000,000/1,000,000`, forcing the male to die exactly at the day-365 M3 mortality boundary before annual M2 fertility/parentage.
- Dedicated run `34170852651`, job `101890718683`, pinned Rust 1.97.1: **success** after clean checkout/build. Exact output: `64/64 control seeds: one focal birth and zero male deaths; 64/64 mortality-arm seeds: one day-365 male demographic death and zero births`.
- Full recorded-run invariants passed in both arms for all 64 seeds.
- **Disposition: no finding.** A male killed at the coincident mortality boundary is not retained in parentage occupancy and cannot authorize a same-day birth; the matched surviving-male control deterministically produces the expected birth.
- Area B remains **in progress** pending one additional fresh limiting-case attack around ageing/newborn exposure or another demographic edge not already covered by historical v4 regressions.

## Next action

Continue **Area B** against immutable `v0.3.5` / v33. Before creating the next evidence branch, search permanent and historical tests for overlap. Prefer a fresh age/newborn limiting case that exercises the full scheduler rather than a helper-only proof: for example, a model-born child whose first-year age-0 mortality risk is zero but whose second-year interval-start age enters a certain-mortality band, verifying exactly one full age-0 year of survival followed by death at the correct second-year M3 boundary. If that case is already permanently covered, choose another promised demographic boundary such as exact fertility-age entry/exit or birth-spacing threshold. Do **not** repair #606 during discovery.
