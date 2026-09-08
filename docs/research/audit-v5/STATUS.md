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
| Coverage | **12/14 Areas complete — Area M next** |
| P0 | none |
| P1 | **4 — AV5-001/#606; AV5-004/#629; AV5-005/#640; AV5-007/#651** |
| P2 | **4 — AV5-002/#617; AV5-003/#627; AV5-006/#648; AV5-008/#658** |
| P3 | none |
| Open Audit-v5 findings | **#606, #617, #627, #629, #640, #648, #651, #658** |
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
| A | Authoritative semantics and scheduler behaviour | **complete — non-clean** | #605 demonstrated AV5-001/#606 P1. #608 and #610 no new finding. |
| B | Demography, fertility, mortality, ageing, population structure | **complete — non-clean** | #612 documented coupling limitation; #614 no finding; #616 demonstrated AV5-002/#617 P2. |
| C | Households, kinship, social links, lifecycle structure | **complete — non-clean only via AV5-002** | #619 and #620 no new finding. |
| D | Resources, condition, subsistence, depletion/recovery | **complete — clean** | #622 and #624 no finding; #623 cadence sensitivity bounded in I. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **complete — non-clean** | #626 demonstrated AV5-003/#627 P2. #628 demonstrated AV5-004/#629 P1. AV5-001 also cross-cutting. |
| F | Aggregation and interaction mechanisms | **complete — clean** | #631, #632 and #633 no finding. Detailed report: `area-f-2026-09-08.md`. |
| G | Initialization, burn-in, path dependence, continuation state | **complete — non-clean only via AV5-003** | #635, #636 and #637 no new finding. Detailed report: `area-g-2026-09-08.md`. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **complete — non-clean** | #639 demonstrated AV5-005/#640 P1; #642 matched Wilson control no finding. Detailed report: `area-h-2026-09-08.md`. |
| I | Sensitivity, uncertainty, convergence, robustness | **complete — non-clean via cross-cutting findings** | #644 cadence refinement and #645 analysis-end sensitivity produced no new finding. Detailed report: `area-i-2026-09-08.md`. |
| J | Identifiability, equifinality, calibration, discrimination | **complete — non-clean** | #647 demonstrated AV5-006/#648 P2; #650 demonstrated AV5-007/#651 P1; #653 valid discriminator control no finding. Detailed report: `area-j-2026-09-08.md`. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **complete — non-clean via cross-cutting findings** | #655 operational relocation/retry preserved identity; no new finding. Detailed report: `area-k-2026-09-08.md`. |
| L | Observability, analysis outputs, statistical summaries | **complete — non-clean** | #657 demonstrated **AV5-008/#658 P2**: terminal survivor-conditioned condition can pass using an early-window survival/population observable. AV5-003, AV5-005, AV5-006 and AV5-007 remain cross-cutting. Detailed report: `area-l-2026-09-08.md`. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **incomplete — next** | AV5-002 documentation/scope obligation; AV5-006 threshold-domain contract; AV5-008 analysis-window semantic obligation are cross-cutting. |
| N | Cross-system integration | **incomplete** | AV5-001 through AV5-008 are early cross-system evidence; explicit N pass still required. |

## Finding register

| Finding | Severity | Area(s) | Issue | Exact discovery evidence | State |
|---|---:|---|---|---|---|
| `AV5-001 — global coupling-rank renumbering lets an isolated founder change an unchanged focal household's M9 equal-cost destination` | P1 | A; E/H/I/N | #606 | PR #605, head `b57ac276e07f89fb3179ad585a586b5be60e150c`, run `34167265827`, job `101880669781`; destination divergence `503/1024` | **open; unrepaired** |
| `AV5-002 — parentage ignores declared close kin and permits first-degree mating` | P2 | B; C/M/N | #617 | PR #616, head `b130fdac8c2ee421203af8a8e5a044a48de29d26`, run `34171257065`, job `101891843233` | **open; unrepaired** |
| `AV5-003 — spatial M9 history replay uses process seed instead of population seed` | P2 | E; G/K/L/N | #627 | PR #626, head `48e28987453cc20fc0cb116d595b7d64cf4ee5f1`, run `34175807550`, job `101904842100` | **open; unrepaired** |
| `AV5-004 — M9 equal-cost destination choice is not spatial-reflection equivariant` | P1 | E; H/N | #629 | PR #628, head `8fe3e5ce9694b23450c57a666d64a95f946fee45`, run `34176415296`, job `101906603997`; `256/256` mismatches | **open; unrepaired** |
| `AV5-005 — small-n normal-CLT mean gate can certify severely under-covered intervals` | P1 | H; J/K/L/N | #640 | PR #639, head `97fa8ee6636b89a7bdb3660583a2484216a9ef5f`, run `34179975096`, job `101916912997`; false-stop probability `0.81`, maximum coverage `0.19` vs nominal `0.95` | **open; unrepaired** |
| `AV5-006 — negative held-out discrimination tolerance can label overlapping structural envelopes as discriminating` | P2 | J; L/M/N | #648 | PR #647, head `5e57dd95bf268c7bb13bf27d711bd2ad1a414e0f`, run `34183461176`, job `101927040185`; zero-gap overlap accepted with tolerance `-1` | **open; unrepaired** |
| `AV5-007 — binary64 parameter-coordinate collapse can falsely certify a wide compatible region as identified` | P1 | J; I/L/N | #651 | PR #650, head `6147ae898f959aec9068f0ae2e3d654a2509afca`, run `34183710162`, job `101927759593`; exact width `0.5`, reported width `0.0` | **open; unrepaired** |
| `AV5-008 — survivor-conditioning gate accepts mismatched survival analysis window` | P2 | L; M/N | #658 | PR #657, head `58f7b6bd5dd7d20330804e6091ce7b3e65c8a4ab`, run `34187243024`, job `101937940280`; matched control valid and terminal/early mismatch also `valid=True` with `failures=[]` | **open; unrepaired** |

## Fresh evidence register

### Area A
- #605 / AV5-001 — run `34167265827`, job `101880669781`.
- #608 — run `34167878417`, job `101882427621`; no finding.
- #610 — corrected run `34170168801`, job `101888814237`; no finding.

### Area B
- #612 — run `34170606978`, job `101890040711`; documented coupling limitation.
- #614 — run `34170852651`, job `101890718683`; no finding.
- #616 / AV5-002 — run `34171257065`, job `101891843233`.

### Area C
- #619 — run `34172549189`, job `101895508298`; no finding.
- #620 — corrected run `34172954061`, job `101896675856`; no finding.

### Area D
- #622 — run `34173500474`, job `101898257594`; no finding.
- #623 — corrected run `34174156771`, job `101900133852`; declared cadence sensitivity.
- #624 — corrected run `34174434312`, job `101900925979`; no finding.

### Area E
- #626 / AV5-003 — corrected run `34175807550`, job `101904842100`.
- #628 / AV5-004 — run `34176415296`, job `101906603997`.

### Area F
- #631 — run `34176902246`, job `101907992004`; no finding.
- #632 — run `34178268167`, job `101911905647`; no finding.
- #633 — run `34178544437`, job `101912698794`; no finding.

### Area G
- #635 — run `34179017837`, job `101914085007`; no finding.
- #636 — run `34179236440`, job `101914732283`; no finding.
- #637 — run `34179379004`, job `101915149933`; no finding.

### Area H
- #639 / AV5-005 — run `34179975096`, job `101916912997`; P1 finding.
- #642 — run `34180240625`, job `101917689688`; no finding.

### Area I
- #644 — corrected run `34180737754`, job `101919165133`; no finding.
- #645 — run `34180965342`, job `101919828088`; no finding.

### Area J
- #647 / AV5-006 — run `34183461176`, job `101927040185`; P2 finding.
- #650 / AV5-007 — run `34183710162`, job `101927759593`; P1 finding.
- #653 — corrected run `34184015481`, job `101928620097`; no finding.

### Area K
- #655 — run `34186026881`, job `101934406468`; no new finding.

### Area L
- **#657 / AV5-008** — exact head `58f7b6bd5dd7d20330804e6091ce7b3e65c8a4ab`; dedicated run `34187243024`; job `101937940280`; `matched_window_valid=True; mismatched_window_valid=True; mismatched_failures=[]`; intentional scientific-oracle failure. **P2 finding.** Evidence PR closed unmerged.

## Area-L completion assessment

Fresh Area-L discovery reviewed authoritative/derived observability, missing/undefined/censored semantics, analysis-window binding, aggregation and denominator/weighting semantics, survivor conditioning, statistical precision/fidelity, downstream fail-closed behavior and provenance compatibility. Historical audit evidence was used only to avoid duplicate attacks.

The fresh survivor-window adversary demonstrated AV5-008/#658 (P2): the survivor-conditioning gate can accept a survival/population observable from an unrelated analysis window. Existing AV5-003/#627, AV5-005/#640, AV5-006/#648 and AV5-007/#651 remain directly cross-cutting Area-L defects.

**Area L is complete — non-clean.** Evidence PR #657 is closed unmerged. No production remediation was performed. Detailed report: `docs/research/audit-v5/area-l-2026-09-08.md`.

## Next action

Begin **Area M — documentation, TRACE/ODD/ODD+D and claim consistency** against immutable `v0.3.5`/v33 from zero v5 Area-M coverage. Reconcile current/frozen scientific claims against executable semantics and already-preserved findings, with special attention to AV5-002 close-kin eligibility, AV5-006 threshold-domain semantics and AV5-008 analysis-window obligations. Do not begin remediation while Area N remains incomplete.