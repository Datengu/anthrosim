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
| Coverage state | **3/14 Areas complete — Area D next** |
| Current P0 findings | none discovered |
| Current P1 findings | **1 — AV5-001 / #606** |
| Current P2 findings | **1 — AV5-002 / #617** |
| Current P3 findings | none discovered |
| Current open Audit-v5 findings | **#606 / AV5-001; #617 / AV5-002** |
| Convergence classification | **pending full A–N discovery; non-clean because Area A demonstrated AV5-001 (P1) and Area B demonstrated AV5-002 (P2)** |
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
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | PR #612 quantified a documented same-seed structural-coupling limitation (`523/1024` focal outcomes changed; no defect). PR #614 confirmed survival-conditioned parentage at the exact day-365 M3/M2 boundary: `64/64` surviving-male controls produced one birth, while `64/64` certain-male-death arms produced one day-365 death and zero births. PR #616 demonstrated **AV5-002 / #617 (P2)**: even complete declared genealogy does not constrain M2 mate eligibility, permitting a daughter’s own father to be selected as the male parent of her child. Historical age/newborn/birth-spacing boundary tests were reviewed as controls rather than counted as fresh v5 evidence. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via cross-cutting AV5-002** | PR #619 quantified the documented dependency-safety rule: a dependent can leave a household at size 3 under target `maxLivingMembers=2` when its only living parent is in the full group; **no defect** because the authoritative contract explicitly makes target size subordinate to dependency safety. PR #620 confirmed same-boundary mortality is respected by dependency-aware fission: a mother killed at day 365 is not retained as an anchor, and the child is reassigned with the surviving father. Source/control review covered relationship-role refinement, reciprocal first-degree kin semantics, fission-event membership/household-age observability, temporary-mobility topology extension and checkpoint determinism. No new Area-C finding; AV5-002 remains a cross-cutting kinship limitation. |
| D | Resources, condition, subsistence, depletion/recovery | **incomplete — next** | — |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **incomplete** | AV5-001 cross-cutting evidence only; Area E not yet independently audited |
| F | Aggregation and interaction mechanisms | **incomplete** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | — |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 is a documented same-seed coupling limitation to revisit during independent Area-H inference/coupling audit. |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 documents why per-agent same-seed invariance cannot be assumed for structural arms. |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | — |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | — |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | AV5-002 includes a documentation/scope ambiguity to revisit explicitly in Area M. |
| N | Cross-system integration | **incomplete** | AV5-001 and AV5-002 are early cross-system evidence; Area N remains incomplete until its explicit pass. |

## Finding register

| Finding | Severity | Area | Issue | Discovery status | Remediation / re-verification status |
|---|---|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N cross-cutting | #606 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #605, exact head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; focal key `1 -> 2`, destination divergence `503/1024` tie seeds | **open and unrepaired during discovery** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/N cross-cutting | #617 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #616, exact head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233`; complete living genealogy produced `Birth(day 365, child 4, female parent 3, male parent 2)` where person 2 is female parent 3’s declared father | **open and unrepaired during discovery** |

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
- Matched control set all background mortality to zero. The mortality arm forced the male to die exactly at the day-365 M3 mortality boundary before annual M2 fertility/parentage.
- Dedicated run `34170852651`, job `101890718683`, pinned Rust 1.97.1: **success**. Exact output: `64/64 control seeds: one focal birth and zero male deaths; 64/64 mortality-arm seeds: one day-365 male demographic death and zero births`.
- Full recorded-run invariants passed in both arms for all 64 seeds.
- **Disposition: no finding.** A male killed at the coincident mortality boundary is not retained in parentage occupancy and cannot authorize a same-day birth.

### 2026-09-07 — Area B pass 3: complete genealogy × close-kin parentage

- Before constructing new evidence, permanent/historical demographic controls were searched. Age-0 and second-year mortality exposure, founder/fertility age-band boundaries, male parent-age boundaries, annual birth-spacing normalization and same-day relocation parentage locality were already covered by prior model-contract tests and therefore were not repackaged as fresh v5 evidence.
- Frozen-source inspection showed that M2 male-parent eligibility checks only living state, male reproductive sex, configured age range and demographic exposure residence. It does not consult female/male-parent genealogy links, including when `FounderGenealogyStatus::CompleteLivingDirectParents` is declared.
- Duplicate searches for incest, consanguinity, close-kin exclusion, relatedness-based mate eligibility and inbreeding found no existing issue. Historical #188 concerns downstream M4 kin-proxy bias, and AV4-005/#495 concerns arbitrary parentage RNG assignment; neither defines mate-relatedness eligibility.
- Evidence-only PR #616 exact head `b130fdac8c2ee421203af8a8e5a044a48de29d26` declared one complete three-person living genealogy in one cell: 40-year-old mother `PersonId(1)`, 40-year-old father `PersonId(2)`, and their 20-year-old daughter `PersonId(3)`. Fertility was certain only for ages 18–24, making the daughter the only fertile female; mortality, resource pressure and migration were disabled; the father was the only residence-local age-eligible male.
- Dedicated workflow run `34171257065`, job `101891843233`, pinned Rust 1.97.1: **success**. The authoritative event was `births=[(365, PersonId(4), PersonId(3), PersonId(2))]`.
- Thus the daughter’s declared father was selected as the male parent of her child, and the resulting child’s maternal grandfather is also its male parent. Full recorded-run invariants accepted the state.
- **Finding preserved as AV5-002 / #617, P2.** The implementation matches the documented minimal residence-local uniform-male rule, so this is not classified as a P1 implementation violation. It is a material model-scope/interpretation limitation because close biological kin are not excluded, the absence is not explicitly warned about, and generated genealogy can propagate into later kin-sensitive mechanisms.

### 2026-09-07 — Area B completion assessment

- Fresh v5 Area-B evidence covered a stochastic structural-composition limitation (#612), a promised mortality→parentage survival boundary (#614), and a population-structure/genealogy limiting case (#616).
- Historical permanent controls were inspected for age transitions, newborn exposure, fertility/male age boundaries, birth-spacing normalization and migration-locality interactions, but were not counted as fresh v5 completion evidence.
- The direct mortality→parentage invariant passed cleanly, while the close-kin limiting case demonstrated AV5-002 / #617.
- **Area B is complete under the audit protocol, but not clean:** AV5-002 / #617 remains open and intentionally unrepaired during discovery. Its kinship/integration consequences are assigned to Areas C and N; its documentation-scope consequences are assigned to Area M.

### 2026-09-08 — Area C pass 1: dependency-safe fission × target-size overage

- Area C began from protected `main` `a19ef66241d52425eb86ec263f2a02a016c03fff`; immutable scientific target remained v0.3.5/v33, with living changes since the tag limited to Audit-v5 documentation.
- Historical issue search separated the fresh hypothesis from #324 (PersonId/cohort slicing) and #399 (relationship-role tie-breaking). Frozen source showed dependents preferentially follow groups containing living parents and use remaining target capacity only as a secondary criterion.
- Evidence-only PR #619 exact head `79ecc7888887906f60ed20995b5a22e1d385d273`, run `34172549189`, job `101895508298`, constructed three independent-age adults plus one 10-year-old dependent with `maxLivingMembers=2`; the dependent’s only living parent was deliberately in the already-full anchor group.
- Full recorded-run invariants passed. Exact post-fission observability was `sizes=[(1,1),(3,1)]`, largest living household `3`, configured target `2`.
- The evidence test’s stronger absolute-ceiling assertion failed, but authoritative `household-lifecycle-structural-sensitivity-v2.md` explicitly defines `maxLivingMembers` as a target subordinate to dependency safety and permits a group to remain above target rather than violate the dependency rule.
- **Disposition: no finding / documented limiting case.** PR #619 was closed unmerged after classification.

### 2026-09-08 — Area C pass 2: same-boundary mortality × dependency-aware fission

- Evidence-only PR #620 attacked whether a parent killed earlier on the annual boundary could remain a dependency anchor for the same day’s fission.
- The first fixture accidentally set the unrelated intended survivor to age 60 under certain mortality at age 45+, so both that anchor and the 50-year-old mother died and no fission was possible. That initial red was a fixture error and is not scientific evidence.
- The unrelated anchor was corrected to age 44 without changing the hypothesis. A subsequent central-CI red was `rustfmt` import ordering only and likewise did not reach scientific model logic.
- Corrected exact evidence head `7ad02415ed72a358381aa1d075771910cf6a0dc8`; dedicated run `34172954061`, job `101896675856`, pinned Rust 1.97.1: **success**.
- Exact event evidence: `deaths=[(365, PersonId(2))]; fissions=[(365, HouseholdId(1), HouseholdId(2), [PersonId(3), PersonId(4)])]`.
- Thus the mother killed on day 365 was excluded from dependency anchoring, and the dependent child was reassigned with the surviving father in the daughter household. Full recorded-run invariants passed.
- **Disposition: no finding.** PR #620 was closed unmerged after classification.

### 2026-09-08 — Area C completion assessment

- Fresh Area-C evidence covered both a structural limiting case (#619) and a same-boundary mortality/lifecycle composition (#620).
- Frozen-source review confirmed the v33 dependency-aware treatment uses independent-age anchors, living-parent-aware dependent assignment, relationship-role refinement before PersonId tie-breaking, and explicit source/daughter household event membership.
- Existing permanent controls were inspected for reciprocal direct-parent kin semantics, fission observability/creation-day reconstruction, temporary-mobility household-topology extension, deterministic replay and checkpoint continuation; they were treated as controls rather than fresh v5 evidence.
- AV5-002/#617 remains scientifically relevant to Area C because generated genealogy can contain first-degree mating and subsequently feed kin-sensitive mechanisms, but Area C found no additional lifecycle/kinship implementation defect beyond that already-preserved cross-cutting limitation.
- **Area C is complete under the audit protocol.** It is non-clean only through the already-open cross-cutting AV5-002 / #617; no AV5-003 was created.

## Next action

Begin **Area D — resources, condition, subsistence, depletion/recovery** from zero independent coverage against immutable `v0.3.5` / v33. Reconstruct live state and historical resource/condition findings before creating evidence. Prioritize fresh limiting cases around conservation/accounting, depletion/replenishment cadence, household competition/order independence, condition recovery/loss bounds, initial stock assumptions, and interactions among M3 resources, temporary presence and mortality. Do **not** repair #606 or #617 during discovery.
