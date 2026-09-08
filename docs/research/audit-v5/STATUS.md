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
| Coverage | **7/14 Areas complete — Area H next** |
| P0 | none |
| P1 | **2 — AV5-001/#606; AV5-004/#629** |
| P2 | **2 — AV5-002/#617; AV5-003/#627** |
| P3 | none |
| Open Audit-v5 findings | **#606, #617, #627, #629** |
| Convergence | **pending full A–N discovery; non-clean** |
| Repair state | **discovery only; do not repair until A–N discovery completes** |
| Empirical readiness | **none implied — this audit is framework/software scientific verification, not case-study archaeological validation** |

## Discovery rules

- Immutable `v0.3.5` / v33 is the scientific discovery target even as `main` advances through audit documentation.
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
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | #612 documented same-seed structural-coupling limitation, no defect. #614 same-boundary mortality→parentage no finding. #616 demonstrated AV5-002/#617 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via AV5-002** | #619 dependency-safe fission target overage is documented. #620 same-boundary mortality/fission no finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | #622 same-cell fission resource conservation no finding. #623 quantified finite-capacity cadence sensitivity for Area I. #624 fixed-point/newborn boundary no finding. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — non-clean** | #626 demonstrated AV5-003/#627 P2. #628 demonstrated AV5-004/#629 P1 with 256/256 M9 reflection mismatches. AV5-001 also cross-cutting. |
| F | Aggregation and interaction mechanisms | **complete — clean** | #631 household-partition person exposure no finding; #632 birth-during-visit half-open exposure no finding; #633 visitor-resource demand partition no finding. Detailed report: `area-f-2026-09-08.md`. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — non-clean only via AV5-003** | #635 post-fission + 3 active-M9-journey resume matched uninterrupted exactly. #636 declared-founder synthetic-only knobs remained inert through 5 births. #637 2-year vs 3-year common-prefix trajectory matched exactly at day 730. AV5-003 remains the cross-cutting initialization/reconstruction defect. Detailed report: `area-g-2026-09-08.md`. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **incomplete — next** | AV5-001 and AV5-004 cross-cutting. #612 documents that structural arms do not promise per-agent common-random-number coupling. Fresh H evidence required. |
| I | Sensitivity, uncertainty, convergence, robustness | **incomplete** | AV5-001 cross-cutting; #612 structural-coupling limitation; #623 finite-capacity cadence effect `350→700` harvest to bound explicitly. |
| J | Identifiability, equifinality, calibration, discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | AV5-003 cross-cutting: explicit-split process replicates with M9 can fail integrity replay. |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | AV5-003 cross-cutting: M9 replay reconstructs the wrong synthetic founder state under explicit split. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | AV5-002 documentation/scope obligation around close-kin/null mate eligibility. |
| N | Cross-system integration | **incomplete** | AV5-001 through AV5-004 are early cross-system evidence; explicit Area-N pass still required. |

## Finding register

| Finding | Severity | Area(s) | Issue | Exact discovery evidence | State |
|---|---:|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N | #606 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; key `1→2`, destination divergence `503/1024` | **open; unrepaired** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/M/N | #617 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233` | **open; unrepaired** |
| `AV5-003 — spatial M9 history replay uses process seed instead of population seed` | P2 | E; G/K/L/N | #627 | PR #626, corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100` | **open; unrepaired** |
| `AV5-004 — M9 equal-cost destination choice is not spatial-reflection equivariant` | P1 | E; H/N | #629 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; `256/256` mismatches | **open; unrepaired** |

## Fresh evidence register

### Area A
- #605 / AV5-001 — head `b57ac276e07f89fb3179ad585a586b5be60e150c`; run `34167265827`; job `101880669781`.
- #608 — head `6858b5ee721253c6097faf24b79c0447c12d48c1`; run `34167878417`; job `101882427621`; death risks 50.818%, 49.829%, 50.549%, 51.233%; no finding.
- #610 — corrected head `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`; run `34170168801`; job `101888814237`; no finding.

### Area B
- #612 — head `2018921f188f312567b984b8ad4403e8e00196cd`; run `34170606978`; job `101890040711`; 523/1024 focal outcomes changed; documented coupling limitation, no defect.
- #614 — head `828b143adbb7ebbc831dd265eeb3557444e506fd`; run `34170852651`; job `101890718683`; no finding.
- #616 / AV5-002 — head `b130fdac8c2ee421203af8a8e5a044a48de29d26`; run `34171257065`; job `101891843233`.

### Area C
- #619 — head `79ecc7888887906f60ed20995b5a22e1d385d273`; run `34172549189`; job `101895508298`; no finding.
- #620 — corrected head `7ad02415ed72a358381aa1d075771910cf6a0dc8`; run `34172954061`; job `101896675856`; no finding.

### Area D
- #622 — head `276b134f52b9db0b546752072add14045f5a013a`; run `34173500474`; job `101898257594`; no finding.
- #623 — corrected head `e8c844da5b1f41ee4f5158eae0993bc68b95a60d`; run `34174156771`; job `101900133852`; P=1 harvest350 vs P=365 harvest700; declared sensitivity, carry to I.
- #624 — corrected head `2fe40a8d7541ede93a95f61ebffd76f708f5641a`; run `34174434312`; job `101900925979`; no finding.

### Area E
- #626 / AV5-003 — corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`; run `34175807550`; job `101904842100`.
- #628 / AV5-004 — head `8fe3e5ce9694b23450c57a666d64a95f946fee45`; run `34176415296`; job `101906603997`.

### Area F
- #631 — head `ea5c2b61b493af21208ac0c2c8124f9f5714bb90`; run `34176902246`; job `101907992004`; unified `1/120/30/4`, split `4/120/120/4`; no finding.
- #632 — head `9ebe76cc97adeea0685f6964f29e45afce0bf96c`; run `34178268167`; job `101911905647`; visitor person-days `120→166`, exactly 46 newborn post-birth days; no finding.
- #633 — head `236dbf28ca0667b7a348b7defad0f0e4707f7f92`; run `34178544437`; job `101912698794`; total/home/visitor demand `1460/1340/120` both arms; no finding.

### Area G
- #635 — head `8d88425634048ca73fe88201b5a5e62494840f1e`; run `34179017837`; job `101914085007`; day730 has 3 fissioned households + 3 active journeys; resumed final state `315745b06d57359e` exactly matched uninterrupted; no finding.
- #636 — head `6f641f41c451f18fff473e57e6cea915d98a5739`; run `34179236440`; job `101914732283`; declared-founder synthetic-only knobs `(1,120,0‰)` vs `(99,1,1000‰)` remained inert through 5 births; final state `ab87f9c225835bf6`; no finding.
- #637 — head `6a83f9819235bb44fe6fa97eb68ef2ac63750302`; run `34179379004`; job `101915149933`; 2y terminal vs 3y checkpoint common prefix identical at day730; state `7bae68fb701c0cc5`, 109 people, 10 events, 2 metrics; no finding.

## Area-G completion assessment

Fresh Area-G evidence covers complex continuation after dynamic topology + active M9, initialization isolation from dormant synthetic-only parameters, and future-horizon common-prefix path independence. Frozen-source/history review covered continuation identity, founder reconstruction, initial-condition/path-dependence contracts and the explicit rule that elapsed time alone does not establish burn-in/equilibrium.

No new Area-G defect was demonstrated. However AV5-003/#627 remains directly cross-cutting because the explicit spatial population realization is not correctly supplied to M9 history reconstruction.

**Area G is complete, non-clean only via AV5-003/#627.**

## Next action

Begin **Area H — stochasticity, RNG, ensembles and Monte Carlo inference** from zero independent coverage against immutable v0.3.5/v33.

Prioritize fresh attacks on:

1. independence/keying of scientific RNG decisions and hidden shared-stream coupling;
2. ensemble seed semantics and reproducibility across ordering/parallelization surfaces;
3. paired-seed interpretation and whether analyses overclaim common-random-number coupling;
4. seed-set/sample-size/statistical summaries and Monte Carlo uncertainty;
5. stochastic tie/choice exchangeability beyond the already-preserved AV5-001 and AV5-004 defects.

Do not repair #606, #617, #627 or #629 during discovery.
