# AnthroSim scientific audit v5 — status ledger

Audit target: immutable AnthroSim `v0.3.5`, tag commit `e7667af52d48a1ffbae2bf7713a2388e65994b42`, model semantics `anthrosim-model-semantics-v33`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v5/README.md`

This is the repository-authoritative compact ledger. Detailed completed-area reports and earlier chronological ledger revisions remain preserved in Git history.

## Current state

| Field | Value |
|---|---|
| Audit generation | v5 / fifth independent scientific audit |
| Immutable discovery target | `v0.3.5` |
| Target tag SHA | `e7667af52d48a1ffbae2bf7713a2388e65994b42` |
| Target software version | `0.3.5` |
| Target model semantics | `anthrosim-model-semantics-v33` |
| Coverage | **7/14 Areas complete — Area H in progress** |
| P0 | none |
| P1 | **3 — AV5-001/#606; AV5-004/#629; AV5-005/#640** |
| P2 | **2 — AV5-002/#617; AV5-003/#627** |
| P3 | none |
| Open Audit-v5 findings | **#606, #617, #627, #629, #640** |
| Convergence | **pending full A–N discovery; non-clean** |
| Repair state | **discovery only; do not repair until A–N discovery completes** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

## Discovery rules

- Immutable `v0.3.5` / v33 remains the scientific discovery target as `main` advances through audit documentation.
- v2/v3/v4 evidence is historical control/hypothesis material only; it does not establish v5 coverage.
- Every Area requires fresh independent falsification evidence.
- Preserve demonstrated defects as issues and in this ledger before repair.
- Evidence-only adversary PRs close unmerged after classification/ledger capture.
- Production remediation starts only after A–N discovery, absent a documented repository-integrity emergency.
- P0/P1 remediation later requires exact-head validation and independent post-merge adversarial re-verification.

## Coverage matrix

| ID | Area | Status | Fresh v5 evidence / disposition |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | #605 demonstrated AV5-001/#606 P1. #608 mortality-cadence stress no finding. #610 M9-return/M4-boundary stress no finding. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | #612 documented same-seed structural-coupling limitation. #614 same-boundary mortality→parentage no finding. #616 demonstrated AV5-002/#617 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via AV5-002** | #619 dependency-safe target overage documented. #620 same-boundary mortality/fission no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | #622 same-cell fission conservation no finding. #623 finite-capacity cadence sensitivity carried to I. #624 fixed-point/newborn boundary no finding. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — non-clean** | #626 demonstrated AV5-003/#627 P2. #628 demonstrated AV5-004/#629 P1 (`256/256` M9 reflection mismatches). AV5-001 also cross-cutting. |
| F | Aggregation and interaction mechanisms | **complete — clean** | #631 household-partition person exposure no finding; #632 birth-during-visit exposure no finding; #633 visitor-resource demand partition no finding. Detailed report: `area-f-2026-09-08.md`. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — non-clean only via AV5-003** | #635 dynamic topology + active-M9 resume exact; #636 declared-founder synthetic knobs inert; #637 future-horizon common prefix exact. Detailed report: `area-g-2026-09-08.md`. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **in progress — non-clean** | #639 demonstrated **AV5-005/#640 P1**: a sequential `mean` plan can stop at `n=2` with nominal 95% interval `[0,0]` although an exact bounded process makes that false-stop event probability 0.81, bounding coverage at 0.19. AV5-001/004 remain cross-cutting. Further independent H control evidence required before closure. |
| I | Sensitivity, uncertainty, convergence, robustness | **incomplete** | AV5-001 cross-cutting; #612 structural-coupling limitation; #623 finite-capacity cadence effect `350→700` harvest to bound explicitly. |
| J | Identifiability, equifinality, calibration, discrimination | **incomplete** | AV5-005 cross-cutting inference/discrimination obligation. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | AV5-003 replay seed-role defect; AV5-005 research-gate/provenance cross-cutting. |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | AV5-003 replay reconstruction defect; AV5-005 statistical-summary cross-cutting. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | AV5-002 documentation/scope obligation around close-kin/null mate eligibility. |
| N | Cross-system integration | **incomplete** | AV5-001 through AV5-005 are early cross-system evidence; explicit N pass still required. |

## Finding register

| Finding | Severity | Area(s) | Issue | Exact discovery evidence | State |
|---|---:|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N | #606 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; key `1→2`, destination divergence `503/1024` | **open; unrepaired** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/M/N | #617 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233` | **open; unrepaired** |
| `AV5-003 — spatial M9 history replay uses process seed instead of population seed` | P2 | E; G/K/L/N | #627 | PR #626, corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100` | **open; unrepaired** |
| `AV5-004 — M9 equal-cost destination choice is not spatial-reflection equivariant` | P1 | E; H/N | #629 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; `256/256` mismatches | **open; unrepaired** |
| `AV5-005 — small-n normal-CLT mean gate can certify severely under-covered intervals` | P1 | H; J/K/L/N | #640 | PR #639, head `97fa8ee6636b89a7bdb3660583a2484216a9ef5f`, run `34179975096`, job `101916912997`; `n=2`, `[0,0]`, `sufficient_stop`, exact false-stop probability `0.81`, maximum overall coverage `0.19` vs nominal `0.95` | **open; unrepaired** |

## Fresh evidence register

### Area A
- #605 / AV5-001 — head `b57ac276e07f89fb3179ad585a586b5be60e150c`; run `34167265827`; job `101880669781`.
- #608 — head `6858b5ee721253c6097faf24b79c0447c12d48c1`; run `34167878417`; job `101882427621`; no finding.
- #610 — corrected head `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`; run `34170168801`; job `101888814237`; no finding.

### Area B
- #612 — head `2018921f188f312567b984b8ad4403e8e00196cd`; run `34170606978`; job `101890040711`; documented coupling limitation.
- #614 — head `828b143adbb7ebbc831dd265eeb3557444e506fd`; run `34170852651`; job `101890718683`; no finding.
- #616 / AV5-002 — head `b130fdac8c2ee421203af8a8e5a044a48de29d26`; run `34171257065`; job `101891843233`.

### Area C
- #619 — head `79ecc7888887906f60ed20995b5a22e1d385d273`; run `34172549189`; job `101895508298`; no finding.
- #620 — corrected head `7ad02415ed72a358381aa1d075771910cf6a0dc8`; run `34172954061`; job `101896675856`; no finding.

### Area D
- #622 — head `276b134f52b9db0b546752072add14045f5a013a`; run `34173500474`; job `101898257594`; no finding.
- #623 — corrected head `e8c844da5b1f41ee4f5158eae0993bc68b95a60d`; run `34174156771`; job `101900133852`; P=1 harvest350 vs P=365 harvest700; declared sensitivity.
- #624 — corrected head `2fe40a8d7541ede93a95f61ebffd76f708f5641a`; run `34174434312`; job `101900925979`; no finding.

### Area E
- #626 / AV5-003 — corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`; run `34175807550`; job `101904842100`.
- #628 / AV5-004 — head `8fe3e5ce9694b23450c57a666d64a95f946fee45`; run `34176415296`; job `101906603997`.

### Area F
- #631 — head `ea5c2b61b493af21208ac0c2c8124f9f5714bb90`; run `34176902246`; job `101907992004`; no finding.
- #632 — head `9ebe76cc97adeea0685f6964f29e45afce0bf96c`; run `34178268167`; job `101911905647`; no finding.
- #633 — head `236dbf28ca0667b7a348b7defad0f0e4707f7f92`; run `34178544437`; job `101912698794`; no finding.

### Area G
- #635 — head `8d88425634048ca73fe88201b5a5e62494840f1e`; run `34179017837`; job `101914085007`; no finding.
- #636 — head `6f641f41c451f18fff473e57e6cea915d98a5739`; run `34179236440`; job `101914732283`; no finding.
- #637 — head `6a83f9819235bb44fe6fa97eb68ef2ac63750302`; run `34179379004`; job `101915149933`; no finding.

### Area H
- **#639 / AV5-005** — head `97fa8ee6636b89a7bdb3660583a2484216a9ef5f`; run `34179975096`; job `101916912997`. Exact diagnostic: `decision=sufficient_stop; n=2; estimate=0.0; interval=[0.0,0.0]; half_width=0.0; false_stop_probability=0.81; max_overall_coverage=0.19; nominal_confidence=0.95`. **P1 finding.**

## Area-G completion assessment

Area G independently covered complex continuation after dynamic topology + active M9, initialization isolation from dormant synthetic-only parameters, and future-horizon common-prefix path independence. AV5-003/#627 remains cross-cutting because explicit spatial population realization is not correctly supplied to M9 history reconstruction.

**Area G is complete, non-clean only via AV5-003/#627.**

## Area-H current assessment

Fresh Area-H discovery has demonstrated AV5-005/#640. Historical #334 (quantile coverage), #410 (independent-arm covariance), #528 (large-integer numeric fidelity) and paired-seed limitation #214 were reviewed as nonduplicates/controls and are not counted as fresh v5 evidence.

Area H remains **in progress**. At least one additional independent fresh control/adversary should exercise another supported inference/RNG surface before completion; do not repair AV5-005 or any other v5 finding during discovery.

## Next action

Continue **Area H — stochasticity, RNG, ensembles and Monte Carlo inference** against immutable v0.3.5/v33. Prioritize a fresh control that is not a replay of #334/#410/#528: e.g. a method-specific probability/Wilson sequential precision construction or an ensemble-order/seed-set invariance probe. Then assess whether H has sufficient fresh breadth to close.
