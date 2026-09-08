# AnthroSim scientific audit v5 — status ledger

Audit target: immutable AnthroSim `v0.3.5`, tag commit `e7667af52d48a1ffbae2bf7713a2388e65994b42`, model semantics `anthrosim-model-semantics-v33`.

Protocol: `docs/research/scientific-audit-protocol.md`

Re-verification addendum: `docs/research/audit-reverification-version-drift.md`

Charter: `docs/research/audit-v5/README.md`

Purpose: durable repository-authoritative state for the fifth independent/adversarial comprehensive scientific audit and the fresh post-Audit-v4 convergence pass.

> Ledger maintenance note (2026-09-08): after Area F this file was compacted from the earlier chronological session-log form into the evidence register below. Exact earlier narrative remains preserved in Git history, while this revision retains the authoritative target, area disposition, finding identities, exact fresh-evidence heads/runs/jobs, cross-cutting obligations and next action.

## Current baseline and state

| Field | Value |
|---|---|
| Audit generation | v5 / fifth independent scientific audit |
| Immutable discovery target | `v0.3.5` |
| Target tag SHA | `e7667af52d48a1ffbae2bf7713a2388e65994b42` |
| Target software version | `0.3.5` |
| Target model semantics | `anthrosim-model-semantics-v33` |
| Coverage state | **6/14 Areas complete — Area G next** |
| Current P0 findings | none discovered |
| Current P1 findings | **2 — AV5-001 / #606; AV5-004 / #629** |
| Current P2 findings | **2 — AV5-002 / #617; AV5-003 / #627** |
| Current P3 findings | none discovered |
| Current open Audit-v5 findings | **#606 / AV5-001; #617 / AV5-002; #627 / AV5-003; #629 / AV5-004** |
| Convergence classification | **pending full A–N discovery; non-clean because Areas A/B/E demonstrated four findings; Areas C/D/F added no new findings** |
| Repair state | **discovery only; do not repair v5 findings until A–N discovery completes** |
| Empirical readiness implication | **none — Audit v5 does not establish empirical validity or archaeological research readiness for a specific case** |

## Initialization reconstruction

At Audit-v5 initialization on 2026-09-07:

- protected `main` was exactly `e7667af52d48a1ffbae2bf7713a2388e65994b42`;
- immutable tag `v0.3.5` resolved to the same exact commit;
- executable model semantics at that tag were `anthrosim-model-semantics-v33`;
- there were 0 open pull requests and 0 open issues;
- no overlapping active Audit-v5 work existed;
- Scientific Audit v4 was complete and historical, with all 15 frozen-target findings repaired and independently re-verified/dispositioned on the living line before v0.3.5 was frozen.

Living `main` may advance only through audit documentation/evidence bookkeeping during discovery. Scientific evidence continues to interrogate immutable `v0.3.5` / v33 or source proven causally identical to it.

## Discovery rules

- The immutable `v0.3.5` tag is the scientific discovery target.
- Audit v2/v3/v4 evidence is historical context and regression-hypothesis material only.
- Each Area starts incomplete and requires fresh evidence.
- Existing regressions/release checks may be reused as controls, but cannot by themselves establish v5 coverage.
- Preserve demonstrated defects in issues and this ledger before repair.
- Use sequential finding identifiers `AV5-001`, `AV5-002`, and so on.
- Continue later Areas against frozen v0.3.5/v33 even after findings are discovered.
- Production repair is deferred until A–N discovery is complete, absent an explicitly documented repository-integrity emergency.
- Evidence-only adversary PRs close unmerged after classification/ledger capture unless a separate production decision promotes a test into the permanent suite.
- P0/P1 remediation later requires exact-head validation and independent post-merge adversarial re-verification.
- Version-drift re-verification follows `docs/research/audit-reverification-version-drift.md`.

## Coverage matrix

| ID | Audit area | Status | Fresh v5 evidence / findings |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | #605 demonstrated AV5-001/#606 P1. #608 falsified a material annual-background-mortality risk shift across 1/4/12/365 M3 cadences. #610 confirmed same-day M9 return completion becomes immediately M4-visible. Frozen scheduler review confirmed M3 → M9 → M4 → annual-M2 ordering. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | #612 quantified a documented same-seed structural-coupling limitation (no defect). #614 confirmed survival-conditioned parentage at the exact M3/M2 boundary. #616 demonstrated AV5-002/#617 P2: complete declared genealogy does not constrain mate eligibility, permitting first-degree parentage. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via cross-cutting AV5-002** | #619 confirmed dependency safety legitimately overrides target household size. #620 confirmed same-boundary mortality is respected by dependency-aware fission. No new Area-C finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | #622 confirmed same-cell fission preserves aggregate resource budget. #623 quantified a large but declared finite-capacity cadence sensitivity for later Area I. #624 confirmed fixed-point condition/newborn boundary semantics. No new Area-D finding. |
| E | Spatial landscape, movement, migration, temporary mobility, and boundaries | **complete — non-clean** | #626 demonstrated AV5-003/#627 P2: spatial explicit-split synthetic M9 replay uses process seed instead of population seed. #628 demonstrated AV5-004/#629 P1: M9 equal-cost destination choice failed horizontal-reflection equivariance in 256/256 paired seeds. AV5-001 is additional cross-cutting E evidence. |
| F | Aggregation and interaction mechanisms | **complete — clean** | #631 confirmed person-level aggregation exposure is invariant to one-vs-four household partition when the same four people make the same visit. #632 confirmed a day-365 birth during an active visit adds exactly post-birth visitor exposure (120→166 person-days; peak 2→3). #633 confirmed identical positive aggregation-driven resource demand under the same partition transformation (total 1460, home 1340, visitor 120 in both arms). See `area-f-2026-09-08.md`. |
| G | Initialization, burn-in, path dependence, continuation state | **incomplete — next** | AV5-003 cross-cutting evidence: explicit spatial population-realization identity is not propagated into M9 history reconstruction. Fresh independent Area-G evidence still required. |
| H | Stochasticity, RNG, ensembles, and Monte Carlo inference | **incomplete** | AV5-001 and AV5-004 cross-cutting evidence; #612 documents a legitimate same-seed structural-coupling limitation to revisit in inference/coupling analysis. |
| I | Sensitivity, uncertainty, convergence, and robustness | **incomplete** | AV5-001 cross-cutting evidence; #612 documents why structural arms cannot assume per-agent common-random-number invariance. #623 quantified `350→700` harvested between P=1 and P=365 under a declared finite-capacity timing contract. |
| J | Identifiability, equifinality, calibration, and discrimination | **incomplete** | — |
| K | Experiment orchestration, configuration, provenance, reproducibility | **incomplete** | AV5-003 cross-cutting: fixed-environment/fixed-population process-replicate spatial runs with M9 can fail recorded-run integrity because replay consumes the wrong seed role. |
| L | Observability, analysis outputs, statistical summaries | **incomplete** | AV5-003 cross-cutting: M9 history replay cannot reconstruct the authoritative synthetic founder state in explicit-split spatial mode. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete** | AV5-002 includes a documentation/scope obligation around close-kin parentage and null mate eligibility. |
| N | Cross-system integration | **incomplete** | AV5-001, AV5-002, AV5-003 and AV5-004 are early cross-system evidence; explicit Area-N pass still required. |

## Finding register

| Finding | Severity | Primary / cross-cutting area | Issue | Discovery evidence | Remediation state |
|---|---:|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N | #606 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; coupling key `1→2`, destination divergence `503/1024` tie seeds | **open; unrepaired during discovery** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/M/N | #617 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233`; day-365 birth used the female parent's declared father as male parent | **open; unrepaired during discovery** |
| `AV5-003 — spatial M9 history replay uses process seed instead of population seed` | P2 | E; G/K/L/N | #627 | PR #626, corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100`; valid explicit-split synthetic-founder + active-M9 run failed replay residence validation | **open; unrepaired during discovery** |
| `AV5-004 — M9 equal-cost destination choice is not spatial-reflection equivariant` | P1 | E; H/N | #629 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; reflection oracle failed `256/256` paired seeds with coupling key fixed at `1` | **open; unrepaired during discovery** |

## Fresh evidence register

### Area A

- **#605 / AV5-001** — isolated-founder coupling locality × M9 equal-cost tie; head `b57ac276e07f89fb3179ad585a586b5be60e150c`; run `34167265827`; job `101880669781`; P1 finding.
- **#608** — annual background mortality × 1/4/12/365 M3 cadences; head `6858b5ee721253c6097faf24b79c0447c12d48c1`; run `34167878417`; job `101882427621`; death risks 50.818%, 49.829%, 50.549%, 51.233%; no finding.
- **#610** — M9 return completion exactly on coincident M4 boundary; corrected head `03176cf04f2f79e3cf944279e3b09dc5d0ae8aec`; run `34170168801`; job `101888814237`; no finding.

### Area B

- **#612** — remote fertility candidate × focal same-seed realization; head `2018921f188f312567b984b8ad4403e8e00196cd`; run `34170606978`; job `101890040711`; `523/1024` focal outcomes changed, classified as documented coupling limitation rather than defect.
- **#614** — certain day-365 male mortality × same-day parentage; head `828b143adbb7ebbc831dd265eeb3557444e506fd`; run `34170852651`; job `101890718683`; 64/64 surviving controls birthed, 64/64 certain-death arms had zero births; no finding.
- **#616 / AV5-002** — complete genealogy × close-kin parentage; head `b130fdac8c2ee421203af8a8e5a044a48de29d26`; run `34171257065`; job `101891843233`; P2 finding.

### Area C

- **#619** — dependency-safe fission × target-size overage; head `79ecc7888887906f60ed20995b5a22e1d385d273`; run `34172549189`; job `101895508298`; documented target subordinate to dependency safety; no finding.
- **#620** — same-boundary mortality × dependency-aware fission; corrected head `7ad02415ed72a358381aa1d075771910cf6a0dc8`; run `34172954061`; job `101896675856`; no finding.

### Area D

- **#622** — same-cell household fission × aggregate resource conservation; head `276b134f52b9db0b546752072add14045f5a013a`; run `34173500474`; job `101898257594`; identical year-2 aggregate budget; no finding.
- **#623** — finite storage capacity × M3 settlement cadence; corrected head `e8c844da5b1f41ee4f5158eae0993bc68b95a60d`; run `34174156771`; job `101900133852`; P=1 harvested 350 vs P=365 harvested 700; declared sensitivity, no Area-D finding; carry to Area I.
- **#624** — latent M3 deterioration × newborn condition initialization; corrected head `2fe40a8d7541ede93a95f61ebffd76f708f5641a`; run `34174434312`; job `101900925979`; resource `400/350/50`, birth day365, remainders `[500,500,0]`; no finding.

### Area E

- **#626 / AV5-003** — explicit spatial seed roles × active seasonality/M9; corrected head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`; run `34175807550`; job `101904842100`; P2 finding.
- **#628 / AV5-004** — M9 equal-cost destination × horizontal reflection; head `8fe3e5ce9694b23450c57a666d64a95f946fee45`; run `34176415296`; job `101906603997`; `256/256` reflection mismatches; P1 finding.

### Area F

- **#631** — same four visitors represented by one household vs four; head `ea5c2b61b493af21208ac0c2c8124f9f5714bb90`; run `34176902246`; job `101907992004`; unified `1 journey / 120 visitor-person-days / 30 visitor-household-days / peak 4`, split `4 / 120 / 120 / 4`; no finding.
- **#632** — birth during active visit; head `9ebe76cc97adeea0685f6964f29e45afce0bf96c`; run `34178268167`; job `101911905647`; arrival 351, birth 365, return departure 411, visitor person-days `120→166`, exactly 46 newborn post-birth days, peak `2→3`; no finding.
- **#633** — household partition × positive visitor resource demand; head `236dbf28ca0667b7a348b7defad0f0e4707f7f92`; run `34178544437`; job `101912698794`; both arms `total_need=1460`, `home_need=1340`, `visitor_need=120`; no finding.
- Detailed Area-F assessment: `docs/research/audit-v5/area-f-2026-09-08.md`.

## Area-F completion assessment

Area F independently tested:

1. person-level aggregation accounting under different household partition;
2. living visitor headcount and half-open exposure when demography changes during a visit;
3. aggregation-driven physical resource-demand attribution under different household claim topology.

Frozen-source/history review additionally covered temporary-presence lifecycle, duration ledgers, current v31 controlled-aggregation machine reference, resource feedback, and the explicit model boundary that M9 does not silently create travel purpose, social motive, encounter networks or mating/social-interaction rules.

**Area F is complete and clean.** No new finding was demonstrated.

## Next action

Begin **Area G — initialization, burn-in, path dependence and continuation state** from zero independent coverage against immutable `v0.3.5` / v33.

Before new evidence:

1. reconstruct live `main`, open PRs/findings and overlapping branches;
2. review historical founder/init/burn-in/checkpoint findings and permanent controls;
3. treat AV5-003/#627 as cross-cutting hypothesis material, not Area-G completion;
4. prioritize fresh attacks on initialization transients, declared-vs-synthetic founder reconstruction, continuation/checkpoint state, household/M9 state across resume, and whether scientifically inert realization/config fields remain inert;
5. do **not** repair #606, #617, #627 or #629 during discovery.
