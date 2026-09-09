# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact handoff state. Detailed area reports and executable evidence are preserved under `docs/research/audit-v6/` and on the referenced evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **13/14 Areas A–N complete** |
| Discovery result | **non-clean: 14 findings — 7 P1, 7 P2; Area N remains** |
| Open Audit-v6 findings | **14** |
| Open P0/P1 | **7 — AV6-001/#687; AV6-004/#707; AV6-006/#711; AV6-009/#726; AV6-010/#729; AV6-012/#737; AV6-013/#741** |
| Phase | **Area N discovery active — cross-system integration** |
| Active ownership | **N — cross-system integration** |
| Production remediation | **prohibited until A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean v6 result is impossible because new P1 findings were demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-only documentation commits advance protected `main`.

## Completed Areas

| Area | Status | Fresh v6 evidence |
|---|---|---|
| A — authoritative semantics and scheduler behaviour | **complete — AV6-001/#687 P1** | #684 clean scheduler-equivalence control; #686/#687 same-day M4-before-newly-due-M9 inversion. `area-a-2026-09-09.md` |
| B — demography, fertility, mortality, ageing and population structure | **complete — AV6-002/#694 P2** | #692 clean chronology/cadence; #693/#694 inconsistent male-parent age-window references. `area-b-2026-09-09.md` |
| C — households, kinship, social links and lifecycle structure | **complete — AV6-003/#699 P2** | #698/#699 external-kin context collapse; #701 clean relabelling control. `area-c-2026-09-09.md` |
| D — resources, condition, subsistence, depletion/recovery | **complete — AV6-004/#707 P1; AV6-005/#708 P2** | scarce-resource reflection and duration-split household-index defects. `area-d-2026-09-09.md` |
| E — spatial landscape, movement, migration, temporary mobility and boundaries | **complete — AV6-006/#711 P1** | unreachable impassable padding changes tied M9 destination; locality controls clean. `area-e-2026-09-09.md` |
| F — aggregation and interaction mechanisms | **complete — AV6-007/#718 P2** | touching half-open visits inflate `peakVisitors`; lifecycle control clean. `area-f-2026-09-09.md` |
| G — initialization, burn-in, path dependence and continuation state | **complete — AV6-008/#721 P2** | incomplete founder genealogy consumed as absence; reproductive-history resume control clean. `area-g-2026-09-09.md` |
| H — stochasticity, RNG, ensembles and Monte Carlo inference | **complete — AV6-009/#726 P1; AV6-010/#729 P1** | sequential Wilson undercoverage; confirmatory sample values not bound to authoritative outputs. `area-h-2026-09-09.md` |
| I — sensitivity, uncertainty, convergence and robustness | **complete — AV6-011/#733 P2** | no-op long-run sensitivity declarations can satisfy coverage; spatial extent late-divergence control clean. `area-i-2026-09-09.md` |
| J — identifiability, equifinality, calibration and discrimination | **complete — AV6-012/#737 P1** | deterministic claim-driving outputs not bound to authoritative executions; equifinality control clean. `area-j-2026-09-09.md` |
| K — experiment orchestration, configuration, provenance and reproducibility | **complete — AV6-013/#741 P1** | finalized-study root verifier accepts semantically forged canonical rows; cross-root transplant control clean. `area-k-2026-09-09.md` |
| L — observability, analysis outputs and statistical summaries | **complete — AV6-014/#745 P2** | drifting initialization families collapse to identical status distributions; exposure/extinction denominator control clean. `area-l-2026-09-09.md` |
| M — documentation, TRACE/ODD/ODD+D and claim consistency | **complete — non-clean via existing AV6-001, AV6-013, AV6-014; no new finding** | #747 executable/documentation adversary; TRACE empirical boundary and benchmark/release wording remain conservative. `area-m-2026-09-09.md` |

## Area M completion

Evidence-only PR #747 compared the living normative claim surface with repository-authoritative v6 evidence.

Final evidence head: `8660d291fcde484774bb18444a0ea47c924bf22c`  
Dedicated run: `34405940821`  
Dedicated job: `102648793144`

The green adversary demonstrated three claim-surface mismatches already owned by existing findings:

```text
long_run_full_distribution_claim_present=true
drifting_initialization_dependence=false
drifting_terminal_mean_ratio=10.0
provenance_fresh_reidentity_rejection_claim_present=true
producer_finalize_rejects=true
root_verifier_accepted=true
m9_before_m4_authoritative_claim_present=true
observed_same_day_order=M4_sequence_1_then_M9_sequence_2
trace_empirical_boundary_conservative=true
area_m_claim_consistency_result=non_clean_via_existing_AV6_001_AV6_013_AV6_014
```

- **AV6-001/#687:** living M9 resource-order documentation says same-day M9 transitions precede M4; the preserved v35 edge case executes M4 sequence 1 then newly-due M9 sequence 2 on day 91.
- **AV6-013/#741:** provenance-v2 documentation says freshly re-identifying a binding over falsified authoritative artifacts cannot make it acceptable; the Area-K producer/verifier adversary demonstrates producer rejection and root-verifier acceptance.
- **AV6-014/#745:** long-run documentation claims initialization/environment dependence compares full normalized outcome distributions; drifting runs currently reach the grouping layer only as `status:drifting`, hiding a persistent 10× initialization separation.

Broader review of README, ODD, ODD+D, scientific-model, TRACE, release/version and benchmark narratives found the high-level boundaries conservative: v0.3.6/v35 identity is explicit, synthetic/capability benchmarks are not promoted to archaeological validation, and TRACE remains **NOT YET EMPIRICALLY RESEARCH-READY**.

No duplicate finding is created merely because living documentation must be synchronized with already-preserved v6 defects.

Detailed evidence: `docs/research/audit-v6/area-m-2026-09-09.md`.

## Area N — active discovery

Primary scope: **cross-system integration**.

Area N starts at zero fresh coverage. It must include genuinely fresh coupled evidence rather than only listing cross-cutting findings. High-value directions include:

- M3 × M9 × M4 same-day resource/presence/residence composition without merely replaying AV6-001;
- initialization × household lifecycle × demography under alternative founder state without duplicating AV6-008;
- stochastic inference × extinction/censoring × downstream identification/robustness without duplicating AV6-009/010/012/014;
- orchestration/provenance × downstream analysis consumers under valid untampered producer output;
- observability × scientific interpretation where individually reasonable subsystem summaries could compose into a misleading conclusion;
- checkpoint/resume × RNG × temporary mobility under an active coupled trajectory.

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Issue | State |
|---|---:|---|---|---|
| AV6-001 | **P1** | A primary; D/E/F/N | #687 | open; remediation deferred |
| AV6-002 | **P2** | B primary; C/G/M/N | #694 | open; remediation deferred |
| AV6-003 | **P2** | C primary; E/N | #699 | open; remediation deferred |
| AV6-004 | **P1** | D primary; E/H/N | #707 | open; remediation deferred |
| AV6-005 | **P2** | D primary; C/E/F/N | #708 | open; remediation deferred |
| AV6-006 | **P1** | E primary; F/H/I/N | #711 | open; remediation deferred |
| AV6-007 | **P2** | F primary; E/L/N | #718 | open; remediation deferred |
| AV6-008 | **P2** | G primary; C/M/N | #721 | open; remediation deferred |
| AV6-009 | **P1** | H primary; K/L/M/N | #726 | open; remediation deferred |
| AV6-010 | **P1** | H primary; K/L/M/N | #729 | open; remediation deferred |
| AV6-011 | **P2** | I primary; K/L/M/N | #733 | open; remediation deferred |
| AV6-012 | **P1** | J primary; K/L/M/N | #737 | open; remediation deferred |
| AV6-013 | **P1** | K primary; J/L/M/N | #741 | open; remediation deferred |
| AV6-014 | **P2** | L primary; G/I/M/N | #745 | open; remediation deferred |

## Discovery/remediation barrier and convergence

Audit v6 is a **non-clean pass**. Required path:

1. finish fresh Area-N discovery against immutable v0.3.6/v35;
2. disposition any additional finding;
3. only then begin remediation by severity/dependency;
4. independently reverify every P0/P1 repair;
5. freeze a new repaired immutable release;
6. require another fresh audit generation before framework convergence can be satisfied.

## Current handoff

Audit-v6 Areas **A–M are complete** once this Area-M completion record reaches protected `main`. Fourteen findings remain deliberately unrepaired.

Next action: execute a genuinely fresh **Area N — cross-system integration** adversary against immutable v0.3.6/v35. Reconstruct live state/overlap before opening evidence and do not repair v6 findings during discovery.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, `docs/research/audit-v6/STATUS.md`, and the latest completed-area report. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the active incomplete Audit-v6 Area from first principles with fresh adversarial evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery completes, and treat the repository ledger as authoritative.
