# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact ledger for Scientific Audit v6. Detailed completed-area reports and executable adversarial evidence are preserved under `docs/research/audit-v6/` and through clearly identified evidence PRs/branches.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target software version | `0.3.6` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **1/14 Areas A–N complete** |
| Discovery result | **non-clean convergence pass already established by AV6-001 P1; discovery continues through B–N** |
| Authoritative Audit-v6 findings | **1** |
| Open Audit-v6 findings | **1** |
| Open P0/P1 | **1** |
| Phase | **Area A complete; Area B is next from zero coverage** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Convergence status | **P1-clean result is no longer possible for v6 because a new P1 was demonstrated** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag remains the discovery baseline even as audit-documentation commits advance protected `main`. No earlier audit Area or finding closure is inherited as v6 completion evidence.

## Initialization reconstruction — 2026-09-09

Audit-v6 groundwork began only after reconstructing live repository state:

- protected `main`: `7d5e47309556e458477cd7283230871363b2c89a`;
- immutable `v0.3.6`: same SHA;
- software version: `0.3.6`;
- model semantics: `anthrosim-model-semantics-v35`;
- open issues: **0**;
- open PRs: **0**;
- pre-existing `audit/v6*` branches: **none**;
- initialization branch: `audit/v6-groundwork`, created directly from immutable `v0.3.6`.

No simulator semantics, scientific configuration, benchmark reference or release identity changed during audit initialization.

## Area A — completed 2026-09-09

Completion report: `docs/research/audit-v6/area-a-2026-09-09.md`

Area A began from zero after a fresh live-state reconstruction. At substantive start:

- protected `main`: `3824babf16eb04cb0797cd8bef3f336961fd6a69`;
- immutable target: `v0.3.6` / `7d5e47309556e458477cd7283230871363b2c89a` / v35;
- open issues: **0**;
- open PRs: **0**;
- no overlapping substantive Area-A owner was present.

### Fresh evidence #684 — no finding

Evidence-only PR **#684**, exact head `d70cf3de2a8c0b851c72554f6da7aab054512543`, attacked duplicated core/spatial scheduler implementations with an exact identity spatial movement transform.

- four process seeds;
- M3/M4 cadence pairs `(1,1)`, `(4,4)`, `(12,4)`, `(4,12)`, `(365,365)`;
- three-year horizon;
- **20** complete authoritative core-checkpoint comparisons.

All comparisons were exactly equal and exact-head CI passed. This is fresh v6 Area-A no-finding evidence. PR #684 was closed unmerged as designed.

### Scheduler/source review

Frozen-source and living-contract inspection established the intended fixed-boundary order as:

1. M3 resource/background-mortality settlement where due;
2. M9 temporary-mobility transitions due on that day;
3. M4 permanent migration where due;
4. annual M2 demography where applicable;
5. optional household-lifecycle processing after M2.

The existing ordinary annual same-day integration test agrees with this contract.

### Fresh evidence #686 — AV6-001 demonstrated

Evidence-only PR **#686**, scientific evidence head `bf94ed046df9d53889d1eacf43c57b52812bc1a7`, attacks an exact `TargetArrivalDay` × M4 boundary case.

Controlled construction:

- one declared household starts at an M9-unreachable Cell 1;
- first M4 boundary is day **91**;
- M4 is deterministically forced to relocate Cell 1 → Cell 2 on day 91;
- Cell 2 → focal Cell 3 has exactly **9** outbound travel days;
- target arrival is day **100**;
- therefore post-M4 reconsideration makes the newly required M9 departure exactly day **91**.

Central CI run `34301664858`, `Quality and tests` job `102309697902` reached the scientific oracle cleanly:

- format passed;
- Clippy passed;
- all **284** pre-existing core unit tests passed;
- dedicated adversary failed with authoritative event order:

```text
M4 HouseholdMigration       day 91, sequence 1
M9 TemporaryJourneyDeparted day 91, sequence 2
```

The event-driven scheduler therefore re-enters M9 from the post-M4 residence on the same authoritative day after the declared M9 phase has already passed.

Historical issue #197 / PR #295 was checked and is related but distinct: it repaired dynamic reconsideration when a residence change creates a valid **future** departure, not a departure newly equal to the already-entered M4 boundary day.

This failure is preserved as **AV6-001 / #687 / P1**. No production repair is authorized during discovery.

### Additional Area-A hypothesis disposition

Exact demographic-duplicate households deliberately share the v35 M9 `m9/household-local-demographic-equivalence-v1` ambiguity key. Because this is explicitly documented in the v34/v35 equal-cost/local-coupling contract, it was not relabelled as an Area-A implementation defect without contrary evidence. Its joint-correlation/interpretation consequences remain a fresh later surface for H/F/L.

An additional pre-scheduling tie hypothesis was also rejected: exact equal-cost M9 destinations share accumulated travel cost and therefore travel duration, so household-specific exact-cost tie selection does not itself make the precomputed target-arrival departure date inconsistent.

### Area-A completion

Area A is **complete — AV6-001 P1 open**. Neighbouring M3/M4/M9/M2, resource-ledger, aggregation, core/spatial parity and continuation implications were considered. Residual non-coverage is recorded in the completion report. Audit-documentation advances to `main` do not require repeating immutable-target evidence because no simulator-semantic commit intervened.

## Discovery phase rules

- Start every Area at zero coverage.
- Use immutable `v0.3.6` / v35 for discovery attribution.
- Prior audits are historical evidence and attack-hypothesis sources only.
- Each Area requires genuinely fresh falsification-oriented evidence; rerunning an earlier adversary alone is insufficient.
- Preserve demonstrated findings before production repair.
- Search open and closed issues/PRs before creating a finding.
- Assign sequential identifiers `AV6-001`, `AV6-002`, ... only after a distinct scientific defect is demonstrated.
- Continue A–N discovery after findings are recorded; production remediation begins only after full discovery completes.
- Keep numerical evidence from semantically different source heads explicitly separated.
- Do not treat green CI, release gates, preserved benchmarks or prior re-verification as proof that an Area is clean.
- Do not introduce case-specific evidence, calibration or desired outcomes into this framework audit.

## Discovery coverage matrix

| ID | Area | Status | Fresh v6 evidence / required direction |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **complete — AV6-001 P1 open** | `area-a-2026-09-09.md`; #684 exact core/spatial identity adversary passed; #686 demonstrated same-day target-arrival/M4 scheduler inversion; #687 open. |
| B | Demography, fertility, mortality, ageing, population structure | **not started — next** | Fresh limiting cases and structural controls for fertility/mortality competition, mate limitation, lifecycle timing, extinction/censoring and finite-population effects. |
| C | Households, kinship, social links, lifecycle structure | **not started** | Challenge formation/fission/parentage lifecycle invariants, relationship-order dependence, household equivalence/locality and downstream demographic/mobility coupling. |
| D | Resources, condition, subsistence, depletion/recovery | **not started** | Probe depletion/replenishment cadence, allocation order/ties, realized-vs-nominal costs, initialization dependence and resource coupling under structural perturbation. |
| E | Spatial landscape, movement, migration, temporary mobility, boundaries | **not started** | Fresh symmetry/reflection/rotation where supported, boundary, unreachable/equal-cost, transformed-input, representation-equivalence and local-coupling attacks beyond prior regressions. |
| F | Aggregation and interaction mechanisms | **not started** | Challenge trigger/timing semantics, temporary concentration versus relocation, interaction opportunity accounting, recovery/crowding effects and mechanism distinguishability. |
| G | Initialization, burn-in, path dependence, continuation state | **not started** | Alternative founder/resource/spatial/network starts, transient-versus-stationary claims, checkpoint continuation equivalence and path-dependence attacks. |
| H | Stochasticity, RNG, ensembles, Monte Carlo inference | **not started** | Challenge seed/stream identity, draw ordering, coupling locality/invariance, rare events, stopping rules, replicate sufficiency, censoring and precision claims. |
| I | Sensitivity, uncertainty, convergence, robustness | **not started** | Fresh parameter/structure/horizon/resolution/initialization/replicate sensitivity and interaction-effect tests; expose hidden fixed configuration. |
| J | Identifiability, equifinality, calibration, discrimination | **not started** | Challenge exact/approximate coordinate handling, compatible regions, compensation, structural equifinality, held-out discrimination and tolerance/boundary behaviour. |
| K | Experiment orchestration, configuration, provenance, reproducibility | **not started** | Attack default resolution, sweep generation, retry/resume/crash recovery, duplicate/partial handling, source/config identity, artifact integrity and replay equivalence. |
| L | Observability, analysis outputs, statistical summaries | **not started** | Challenge denominators, weighting, censoring/missingness, survival conditioning, time windows, multimodality, incompatible-run mixing and mechanism observability. |
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Compare executable v35 semantics and current artifacts with living scientific claims, explicit null-model boundaries, release statements and benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across demography×households/resources, households×movement, movement/aggregation×resources, initialization×spatial/demography, inference×censoring, sensitivity×hidden config, calibration×identifiability, resume×RNG and observability×interpretation. |

An Area is not complete merely because no issue is found. Evidence, mechanisms inspected, adversarial construction, quantitative result where relevant, neighbouring-system considerations and residual uncertainty must be recorded.

## Finding register

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | Discovery state | Later remediation / re-verification |
|---|---:|---|---|---|---|---|
| `AV6-001` | **P1** | A primary; E/F and D/N cross-cutting | `v0.3.6` / `7d5e473...`; evidence PR #686 head `bf94ed0...`; CI `34301664858` job `102309697902`; day-91 M4 sequence 1 then M9 sequence 2 | **#687** | **demonstrated; open** | **deferred until A–N discovery completes** |

## Discovery/remediation barrier

Until Areas A–N are all complete:

- do not merge production scientific-semantic repairs for Audit-v6 findings;
- do not rewrite the immutable discovery baseline;
- do not close a finding merely because an experimental branch contains a possible fix;
- continue auditing the released target so later findings are not hidden by mid-audit semantic changes.

After discovery completes, create a remediation plan ordered by severity/dependency. Every P0/P1 repair then requires normal production review/gates plus independent post-merge adversarial re-verification before the finding is considered fully closed.

## Convergence decision

Audit v6 can no longer be a P1-clean pass because `AV6-001` is a new P1 finding. The required path is now:

1. complete discovery through Areas B–N against immutable `v0.3.6` / v35;
2. disposition every additional finding;
3. after discovery, remediate findings in severity/dependency order;
4. independently reverify every P0/P1 repair;
5. freeze the repaired line as a new immutable release;
6. require another fresh audit generation before the framework-convergence gate can be satisfied.

Neither this non-clean result nor later remediation constitutes empirical calibration or validation.

## Current handoff

Audit-v6 **Area A is complete** against immutable `v0.3.6` / v35 with **AV6-001 / #687 P1 open and deliberately unrepaired**.

Next recommended action:

1. merge this Area-A documentation disposition after normal documentation/CI gates;
2. close evidence-only PR #686 unmerged after the ledger is authoritative on `main`;
3. reconstruct live state again;
4. begin **Area B — demography, fertility, mortality, ageing and population structure** from zero coverage;
5. continue to preserve any new findings before repair.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.