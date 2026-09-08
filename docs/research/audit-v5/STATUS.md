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
| Coverage state | **5/14 Areas complete — Area F next** |
| Current P0 findings | none discovered |
| Current P1 findings | **2 — AV5-001 / #606; AV5-004 / #629** |
| Current P2 findings | **2 — AV5-002 / #617; AV5-003 / #627** |
| Current P3 findings | none discovered |
| Current open Audit-v5 findings | **#606 / AV5-001; #617 / AV5-002; #627 / AV5-003; #629 / AV5-004** |
| Convergence classification | **pending full A–N discovery; non-clean because Areas A/B/E demonstrated AV5-001 (P1), AV5-002 (P2), AV5-003 (P2), and AV5-004 (P1); Areas C and D added no new findings** |
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
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | PR #622 confirmed same-cell household fission preserves the next-period aggregate resource budget exactly (`regenerated/supplied=482`, `unmet=399518` in both arms). PR #623 quantified the explicitly documented finite-capacity timing effect: `P=1` harvested `350` with `650` unmet, while `P=365` harvested `700` with `300` unmet; both arms conserved exactly, so this is a resolution sensitivity for Area I rather than a defect. PR #624 confirmed the v20 fixed-point/newborn boundary exactly: resource `400/350/50`, one day-365 birth, terminal condition-loss remainders `[500,500,0]`. Historical AV2-005/#326 and related M3 repairs were reviewed as controls, not fresh coverage. No new Area-D finding. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — non-clean** | PR #626 demonstrated **AV5-003 / #627 (P2)**: explicit-split spatial synthetic founders use `populationSeed`, but M9 history replay reconstructs them from process `ExperimentConfig.seed`, so valid fixed-environment/fixed-population M9 runs can reject themselves. Its declared-founder inert-population-seed control passed. PR #628 demonstrated **AV5-004 / #629 (P1)**: one-household M9 equal-cost destination selection failed horizontal-reflection equivariance in `256/256` paired seeds while the coupling key remained unchanged. Frozen source/history review covered explicit finite-boundary semantics, scale/readiness contracts, square-cell enforcement, repaired #212 seed-role separation, repaired M4 spatial-isomorphism controls, M9 route-cost/reachability, temporary presence/residence separation, and historical #190/#211/#203/#392/AV4-003/007/009 without counting them as fresh v5 evidence. AV5-001 remains additional cross-cutting E evidence. |
| F | Aggregation and interaction mechanisms | **incomplete — next** | — |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete** | AV5-003 cross-cutting evidence: the explicit spatial population-realization identity is not propagated into M9 history reconstruction. |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 and AV5-004 cross-cutting evidence; PR #612 is a documented same-seed coupling limitation to revisit during independent Area-H inference/coupling audit. |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence only; PR #612 documents why per-agent same-seed invariance cannot be assumed for structural arms. PR #623 adds a large but contract-consistent finite-capacity M3 resolution sensitivity (`350→700` harvested between `P=1` and `P=365`) that Area I must explicitly bound/converge rather than assume negligible. |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | AV5-003 cross-cutting evidence: fixed-environment/fixed-population process-replicate spatial runs with M9 can fail recorded-run integrity because replay consumes the wrong seed role. |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | AV5-003 cross-cutting evidence: M9 event-history replay/validation cannot reconstruct the authoritative synthetic founder state in explicit-split spatial mode. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | AV5-002 includes a documentation/scope ambiguity to revisit explicitly in Area M. |
| N | Cross-system integration | **incomplete** | AV5-001, AV5-002, AV5-003 and AV5-004 are early cross-system evidence; Area N remains incomplete until its explicit pass. |

## Finding register

| Finding | Severity | Area | Issue | Discovery status | Remediation / re-verification status |
|---|---|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N cross-cutting | #606 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #605, exact head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; focal key `1 -> 2`, destination divergence `503/1024` tie seeds | **open and unrepaired during discovery** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/N cross-cutting | #617 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #616, exact head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233`; complete living genealogy produced `Birth(day 365, child 4, female parent 3, male parent 2)` where person 2 is female parent 3’s declared father | **open and unrepaired during discovery** |
| `AV5-003 — spatial M9 history replay uses process seed instead of population seed` | P2 | E; G/K/L/N cross-cutting | #627 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #626, corrected exact head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100`; declared-founder population-seed inertness passed, but explicit-split synthetic-founder + active-M9 execution failed its own history validator because replay reconstructed residences from `ExperimentConfig.seed` instead of the spatial `populationSeed` | **open and unrepaired during discovery** |
| `AV5-004 — M9 equal-cost destination choice is not spatial-reflection equivariant` | P1 | E; H/N cross-cutting | #629 | **demonstrated on frozen v0.3.5/v33 target** — evidence PR #628, exact head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; horizontal-reflection oracle failed in `256/256` paired seeds while authoritative destination coupling key remained `1` in both arms | **open and unrepaired during discovery** |

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
- **Area C is complete under the audit protocol.** It is non-clean only through the already-open cross-cutting AV5-002 / #617; no Area-C-local finding was created.

### 2026-09-08 — Area D pass 1: same-cell fission × aggregate resource conservation

- Area D began from protected `main` `c322842f7f10c8b7201c81661fbb836abfcd78d4`; immutable scientific target remained v0.3.5/v33.
- Historical Area-D/resource findings were searched first. In particular, AV2-005/#326's per-boundary partial-supply ceiling amplification was confirmed historical and repaired by the v20 fixed-point condition remainder; it was not replayed as fresh v5 evidence.
- Evidence-only PR #622 exact head `276b134f52b9db0b546752072add14045f5a013a`, run `34173500474`, job `101898257594`, compared two otherwise-identical two-year four-person same-cell runs: one retained a fixed founder household and one fissioned into two same-cell households after year 1.
- Mortality, fertility, migration, condition response and condition mortality were neutralized. The predeclared invariant allowed household-level sharing outcomes to differ but required the unchanged cell-level resource budget to remain conserved.
- Exact year-2 output was `fixed=(stock_before=0, regenerated=482, need=400000, supplied=482, unmet=399518, stock_after=0); split=(stock_before=0, regenerated=482, need=400000, supplied=482, unmet=399518, stock_after=0)`.
- Full recorded-run invariants passed. **Disposition: no finding.** PR #622 was closed unmerged.

### 2026-09-08 — Area D pass 2: finite storage capacity × M3 settlement timing

- Evidence-only PR #623 compared a one-person, one-cell, one-year capacity-stress fixture at `P=1` versus `P=365`, holding annual need, annual regeneration, initial stock, productivity and all non-resource causal mechanisms fixed. Day-zero stock began exactly at a one-year storage cap.
- The first run `34174073058` / job `101899891114` was a harness-only compile failure caused by naming a non-exported recorded-run type; no scientific assertion executed.
- Corrected exact head `e8c844da5b1f41ee4f5158eae0993bc68b95a60d`, run `34174156771`, job `101900133852`: **success**.
- Exact output was `initial_stock=350; P=1 regenerated=0, harvested=350, unmet=650, final_stock=0; P=365 regenerated=350, harvested=700, unmet=300, final_stock=0`.
- Both arms exactly conserved `initial stock + regeneration = harvest + final stock` and `harvest + unmet = annual need`; full recorded-run invariants passed.
- **Disposition: no Area-D finding.** The magnitude is large, but the v20 resource-time contract explicitly permits finite-capacity partition sensitivity because regeneration clipping occurs before harvest at each settlement. This quantified `350→700` harvest difference is assigned to Area I for explicit convergence/robustness bounding rather than assumed negligible. PR #623 was closed unmerged.

### 2026-09-08 — Area D pass 3: latent M3 deterioration × newborn condition initialization

- Evidence-only PR #624 exercised the documented v20 boundary where a newborn inherits the mother's visible integer condition but not her latent `conditionLossRemainderThousandths`.
- The controlled one-year fixture used two 30-year-old parents, no initial stock, one resource settlement, resource supply `350/400`, zero mortality, certain fertility, no migration, zero recovery, no condition mortality, and a reference-quarter maximum condition loss of one point.
- The first run `34174345793` / job `101900670417` was harness-only: `EventKind::Birth` was destructured with field `child` rather than the actual `person`; compilation stopped before the scientific oracle.
- Corrected exact head `2fe40a8d7541ede93a95f61ebffd76f708f5641a`, run `34174434312`, job `101900925979`: **success**.
- Exact output was `resource=(need=400, supplied=350, unmet=50); births=[(365, PersonId(3), PersonId(1), PersonId(2))]; remainders=[500, 500, 0]`.
- The 875‰ supply fraction creates exactly 500 thousandths latent deterioration in each parent with no whole-point condition loss; the newborn begins at visible maternal condition 1000 and zero latent remainder exactly as declared. Full recorded-run invariants passed.
- **Disposition: no finding.** PR #624 was closed unmerged.

### 2026-09-08 — Area D completion assessment

- Fresh Area-D coverage tested cross-household-topology resource conservation (#622), a strong finite-capacity temporal-resolution stress (#623), and the repaired fixed-point condition/newborn integration boundary (#624).
- Historical controls/source review covered annual demand conservation, seasonal mean-preserving integration, largest-remainder household allocation, M9 duration-aware demand, zero-demand neutrality, fixed-point partial-supply response, condition-mediated mortality timing, initial stock/capacity separation, resource checkpoint accounting and period-level observability.
- No fresh implementation defect was demonstrated. The only large new effect, #623, is explicitly allowed by the declared model and is therefore a sensitivity/convergence obligation for Area I rather than a finding.
- **Area D is complete and clean under the Audit-v5 protocol; no Area-D finding was created.**

### 2026-09-08 — Area E reconstruction and historical-control review

- Area E began from protected `main` `329fcc39b339700dcfc068eb5baac246b0503af2`; immutable scientific target remained v0.3.5/v33.
- Historical spatial findings were reviewed before new evidence. #211 already defines the finite world as an explicit closed graph and adds analysis-domain/buffer/extent-sensitivity machinery; #203 already treats raster scale as an explicit scientific/readiness dimension; #212 introduced separate environment/population/process realization identities; AV4-003/#491 repaired M4 household-label RNG assignment; AV4-007/#500 repaired M9 HouseholdId tie-key dependence; AV4-009/#518 repaired M4 candidate-order/spatial-reflection dependence.
- Frozen v0.3.5 source and permanent controls were inspected for square-cell enforcement on M4 and M9, M4 candidate geometry and simultaneous application, M9 symmetric edge costs and unreachability, finite-boundary declarations/analysis buffers, transformed-world/provenance binding, explicit seed roles, temporary presence versus persistent residence, checkpoint reconstruction, and M4 horizontal/vertical reflection regressions.
- These historical repairs and permanent tests were used only as controls/hypothesis sources, not counted as fresh v5 coverage.

### 2026-09-08 — Area E pass 1: spatial realization seed roles × active seasonality/M9

- Evidence-only PR #626 attacked the repaired #212 seed-role contract under active residual seasonality and configured M9 rather than merely replaying the basic permanent seed-separation unit test.
- The initial head `48a01e93c6e77247bbd5b70b67b48101a73824aa` failed to compile because the evidence harness used the wrong diagnostic formatting type and an incorrect provenance field path. No scientific oracle ran; those errors are not evidence.
- Corrected exact head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100`, pinned Rust 1.97.1, compiled and executed both seed-role tests.
- The declared-founder control passed: with environment/process fixed and only the unused population seed changed `80100→80101`, the complete core causal checkpoint was identical; output was `declared founders: population_seeds=(80100, 80101); core_state=f08df7132f137f8b; events=16`.
- The synthetic-founder explicit-split arm established identical authoritative worlds, initial population digests and residual seasonal fields when only process seed varied, but `SpatialLandscapeSimulation::run_recorded()` then rejected its own valid M9 history with `temporary departure residence does not match replay residence`.
- Frozen-source inspection established the cause: the spatial host initializes synthetic founders with resolved `populationSeed`, while `temporary_history.rs` reconstructs synthetic founders with `RngFactory::new(checkpoint.experiment.seed)`, which is the process seed under explicit split.
- Duplicate analysis distinguished this from AV3-002/#392: #392 repaired declared-founder mode handling before spatial seed-role separation; the new failure is a later composition regression between that replay seam and #212's population/process split.
- **Finding preserved as AV5-003 / #627, P2.** It is fail-closed but blocks the fixed-environment/fixed-population/varying-process spatial-M9 experiment mode that #212 explicitly introduced.

### 2026-09-08 — Area E pass 2: M9 equal-cost tie × horizontal spatial reflection

- Evidence-only PR #628 exact head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`, tested a one-household 3×1 centre-origin M9 tie problem against its exact horizontal reflection over process seeds `1..=256`.
- Both arms kept the same founder/person/household, centre origin, explicit environment/population seeds, M9 trigger, equal movement costs, focal candidate set and authoritative destination coupling key. A non-causal resource field `[900,100,500]` was reflected to `[500,100,900]` so the complete physical spatial state was mirrored while resource demand remained zero.
- Checkout, pinned toolchain and compilation succeeded. The test executed the complete sweep and reported **256/256 reflection mismatches**. Representative pairs were `(seed 1, CellId(1)→CellId(1), key 1)` and `(seed 3, CellId(3)→CellId(3), key 1)`; every reflected run retained the same canonical destination rather than the mirrored physical destination.
- Frozen source explains the behavior: `resolution_for_coupling_key` hashes tie-policy identity, destination tie seed, numeric origin, coupling key and trigger, then chooses `hash % candidates.len()` from the canonically `CellId`-ordered equal-cost candidate vector. The centre origin/key/trigger stay fixed under reflection, so the same vector index is selected rather than transforming with the physical alternative.
- Duplicate analysis distinguished historical #190 (marginal lower-CellId bias), AV4-007/#500 (HouseholdId tie key), AV5-001/#606 (global coupling-rank locality) and AV4-009/#518 (the analogous but separate M4 spatial-reflection defect).
- **Finding preserved as AV5-004 / #629, P1.** An arbitrary row-major spatial representation changes the paired physical M9 outcome under a scientifically isomorphic reflection.

### 2026-09-08 — Area E completion assessment

- Fresh Area-E evidence independently attacked repaired spatial-realization provenance/replay composition (#626) and M9 spatial-isomorphism of equal-cost stochastic destinations (#628). Both demonstrated new defects, AV5-003 and AV5-004.
- Frozen-source/history review additionally covered the principal landscape/grid/boundary/M4/M9 contract surfaces and explicitly separated historical repaired findings from fresh v5 evidence.
- AV5-001/#606 remains cross-cutting Area-E evidence but was not reused as Area-E's fresh attack.
- **Area E is complete under the Audit-v5 protocol, but non-clean:** AV5-003/#627 (P2) and AV5-004/#629 (P1) remain open and intentionally unrepaired during discovery.

## Next action

Begin **Area F — aggregation and interaction mechanisms** from zero independent coverage against immutable `v0.3.5` / v33. Reconstruct live state and historical aggregation/co-presence findings before creating evidence. Prioritize fresh adversaries around occupancy/co-presence definitions, aggregation trigger timing, visitor-versus-resident accounting, duration/person-day weighting, equality/boundary cases, resource feedback, and whether benchmark summaries remain invariant to scientifically irrelevant representation choices. Do **not** repair #606, #617, #627 or #629 during discovery.
