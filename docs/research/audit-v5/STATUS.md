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
| Coverage state | **1/14 Areas complete — Area B next** |
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
| B | Demography, fertility, mortality, ageing, population structure | **incomplete — next** | — |
| C | Households, kinship, social links, lifecycle structure | **incomplete** | — |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete** | AV5-001 cross-cutting evidence only; Area E not yet independently audited |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 cross-cutting evidence only; Area H not yet independently audited |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence only; Area I not yet independently audited |
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
- Initial Area-A attack direction: challenge scheduler/order invariance and, independently of v4 replay, attack whether the v26–v33 shared stochastic-coupling/equivalence machinery composes without introducing new arbitrary ordering, tie, or simultaneous-event dependencies across demographic, resource, M4 and M9 mechanisms.

### 2026-09-07 — Area A pass 1: global coupling locality × M9 equal-cost ties

- Reconstructed live state before evidence work: protected `main` `34abc607d3f9dcbc9c9972f790b623fe53251f02`; immutable discovery target remained `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / v33; no open issue/PR overlap existed before the evidence branch.
- Source inspection identified a fresh composition hypothesis: v33 initial stochastic coupling identities are globally unique ordinal ranks over the complete represented founder population, while M9 v31+ hashes the numeric minimum living-person rank as the equal-cost household destination key.
- Evidence-only PR #605 exact head `b57ac276e07f89fb3179ad585a586b5be60e150c` added a dedicated Rust adversary and pinned Rust 1.97.1 workflow; no production model code changed.
- Controlled arms kept the focal founder exactly unchanged at `PersonId(1)` / `HouseholdId(1)` / `CellId(13)` and appended only one older founder in a separate household at an M9-unreachable isolated `CellId(1)`.
- The added founder changed the focal persisted stochastic-coupling rank from `1` to `2` despite being unreachable from the tested M9 component.
- The focal origin had exactly two equal-cost destinations, `CellId(8)` and `CellId(18)`. Over tie seeds `0..=1023`, authoritative `resolution_for_coupling_key` changed the focal destination in **503/1024** cases.
- Dedicated workflow run `34167265827`, job `101880669781`: checkout/toolchain/build succeeded; the test failed only at the intended locality assertion after compiling `anthrosim-core v0.3.5` successfully.
- Duplicate search found no existing issue for M9 global coupling-rank/population-composition locality. Historical AV4-007/#500 is related but distinct: it demonstrated direct `HouseholdId` dependence and was independently reverified as repaired after v31; AV5-001 demonstrates a new nonlocal dependency introduced by the replacement globally ordinal key. Historical #324 concerns household-fission PersonId/cohort sorting and is also distinct.
- Finding preserved as **AV5-001 / #606, P1** before any production repair.

### 2026-09-07 — Area A pass 2: annual background mortality × M3 cadence

- Evidence-only PR #608 exact head `6858b5ee721253c6097faf24b79c0447c12d48c1` attacked update-frequency dependence in the full executed scheduler rather than relying only on the interval-risk helper proof.
- Controlled experiment: 8,192 one-year seeds per arm; one founder; one-cell world; annual background mortality fixed at `500,000/1,000,000` in every age band; fertility, migration, resource need and condition/scarcity mortality disabled; only M3 `periodsPerYear` varied across `1`, `4`, `12`, `365`.
- Predeclared scientific guards: each empirical annual mortality estimate must remain within 47%–53%, and maximum-minus-minimum cadence mortality must remain at or below 4 percentage points. At `n=8192`, true `p=0.5` has standard error about 0.00552, so these thresholds are deliberately conservative against ordinary Monte Carlo noise.
- Dedicated workflow run `34167878417`, job `101882427621`, pinned Rust 1.97.1: **success**. Exact results:
  - 1 period/year: `4163/8192` deaths = `508178` per million = **50.818%**;
  - 4 periods/year: `4082/8192` = `498291` per million = **49.829%**;
  - 12 periods/year: `4141/8192` = `505493` per million = **50.549%**;
  - 365 periods/year: `4197/8192` = `512329` per million = **51.233%**.
- Observed max-minus-min spread: `4197 - 4082 = 115` deaths, `115/8192 = 1.404` percentage points, well inside the predeclared 4-point bound; all individual arms were inside 47%–53%.
- **Disposition: no finding.** This falsifies a material one-year annual-risk shift from M3 partition cadence for the tested constant annual mortality configuration. It does not establish invariance of within-year death timing, age-band-crossing cases, or interactions with coincident mechanisms.
- Source/document review also confirmed that age-specific background mortality uses the declared model-year-start age band and that mortality is intentionally resolved at M3 boundaries before coincident M4 opportunities; those declared semantics are not treated as defects merely because changing M3 cadence can change within-year timing.

### 2026-09-07 — Area A pass 3: M9 return completion × coincident M4 boundary

- Evidence-only PR #610 attacked the uncovered complement of the permanent same-day departure regression: whether a household whose temporary journey **completes exactly on an M4 boundary** becomes M4-eligible immediately or one decision boundary late.
- The scientific construction fixed seed `50_003`, one year, 20 founders in four five-person households, zero fertility and background mortality, zero resource need and condition/scarcity mortality, synthetic M4 with four annual decision boundaries, and a day-0 M9 departure with zero outbound/return travel and `182` visiting days. Therefore every journey completes exactly on the second M4 boundary, day 182.
- Initial dedicated run `34168320489`, job `101883665488`, was **not scientific evidence**: it failed to compile because the first external integration-test harness tried to use crate-private/internal conveniences (`Simulation::new_with_temporary_mobility` and root-level ID imports). No scientific assertion executed. The harness was corrected without changing the predeclared scientific oracle by moving the test into the existing internal M9 integration-test scope.
- Corrected evidence head: `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`.
- Corrected dedicated workflow run `34170168801`, job `101888814237`, pinned Rust 1.97.1: **success**. Exact output:
  - represented households: `4`;
  - baseline M4 evaluations: `16` (`4 × 4` boundaries);
  - active-M9 M4 evaluations: `12` (`4 × 3` eligible boundaries);
  - day-182 journey completions: `4`;
  - last day-182 completion event sequence: `16`;
  - first day-182 permanent-migration event: none for this seed.
- The `12` active-M9 evaluations establish the intended same-day visibility: day 91 excludes all visiting households, while after all four returns complete at day 182 the same households are evaluated by M4 on days 182, 273 and 365. Full recorded-run invariants passed.
- **Disposition: no finding.** The tested return-completion boundary is coherent with the declared M3 → M9 → M4 scheduler semantics.

### 2026-09-07 — Area A completion assessment

- Frozen executable scheduler inspection confirmed that temporary transitions strictly before a fixed boundary are drained first; on the fixed day itself the executed order is elapsed M3 resource/background/condition mortality settlement, then M9 temporary transitions, then M4 permanent migration, followed by annual M2 demography after the subannual loop. Living scientific documentation states the same causal order.
- Frozen competing-mortality implementation uses separate latent condition/background triggers and a symmetric dual-trigger attribution rule. Permanent controls include exact exchange-of-causes/streams attribution symmetry across 10,000 draws and a 100,000-trial frequency check against the independent-union/risk-weighted attribution contract.
- Fresh v5 evidence covered three distinct Area-A attack classes: arbitrary shared coupling/tie identity (#605), operational update-frequency dependence (#608), and a same-day state-boundary composition (#610). Neighbouring M3/M4/M9/M2 interactions and documentation/executable agreement were explicitly inspected.
- **Area A is complete under the audit protocol, but it is not a clean Area:** AV5-001 / #606 is a demonstrated open P1 finding and remains intentionally unrepaired during discovery. The other two fresh adversaries produced quantitative no-finding results. No unresolved Area-A uncertainty currently requires keeping the Area open; cross-cutting consequences of AV5-001 remain assigned to later Areas E/H/I/N for their independent passes.

## Next action

Begin **Area B — demography, fertility, mortality, ageing, and population structure** from zero coverage against immutable `v0.3.5` / v33. Use a genuinely fresh adversarial construction rather than treating v2/v3/v4 demographic regressions as completion evidence. Prioritize limiting cases and population-structure interactions around fertility eligibility/timing, parent availability, mortality/age boundaries, newborn initialization, extinction/censoring, and finite-population behaviour. Do **not** repair #606 during discovery.
