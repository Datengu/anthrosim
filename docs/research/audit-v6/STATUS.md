# AnthroSim scientific audit v6 — status ledger

Audit target: immutable AnthroSim `v0.3.6`, tag commit `7d5e47309556e458477cd7283230871363b2c89a`, model semantics `anthrosim-model-semantics-v35`.

Protocol: `docs/research/scientific-audit-protocol.md`  
Re-verification addendum: `docs/research/audit-reverification-version-drift.md`  
Charter: `docs/research/audit-v6/README.md`

This is the repository-authoritative compact ledger for Scientific Audit v6. Detailed completed-area reports and executable adversarial evidence should be added under `docs/research/audit-v6/` or preserved through clearly identified evidence branches/PRs while this ledger remains the authoritative handoff state.

## Current state

| Field | Value |
|---|---|
| Audit generation | v6 / sixth independent scientific audit |
| Immutable discovery target | `v0.3.6` |
| Target tag SHA | `7d5e47309556e458477cd7283230871363b2c89a` |
| Target software version | `0.3.6` |
| Target model semantics | `anthrosim-model-semantics-v35` |
| Discovery coverage | **0/14 Areas A–N complete** |
| Discovery result | **not yet determined** |
| Authoritative Audit-v6 findings | **0** |
| Open Audit-v6 findings | **0** |
| Open P0/P1 | **0** |
| Phase | **discovery initialized; no Area completed yet** |
| Production remediation | **prohibited until full A–N discovery completes, except documented repository-integrity emergency** |
| Desired convergence result | **fresh P1-clean pass: no new P0/P1 findings** |
| Empirical readiness | **none implied — framework/software scientific verification only** |

The immutable `v0.3.6` / v35 tag is the discovery baseline even if audit-documentation commits later advance protected `main`. No earlier audit Area or finding closure is inherited as v6 completion evidence.

## Initialization reconstruction — 2026-09-09

Audit-v6 groundwork was initialized only after reconstructing the live repository state:

- protected `main`: `7d5e47309556e458477cd7283230871363b2c89a`;
- immutable `v0.3.6`: `7d5e47309556e458477cd7283230871363b2c89a`;
- software version: `0.3.6`;
- model semantics: `anthrosim-model-semantics-v35`;
- open issues at initialization: **0**;
- open PRs at initialization: **0**;
- pre-existing `audit/v6*` branches: **none**;
- initialization branch: `audit/v6-groundwork`, created directly from immutable `v0.3.6`.

No simulator semantics, scientific configuration, benchmark reference or release identity is changed by audit initialization.

## Discovery phase rules

- Start every Area at zero coverage.
- Use immutable `v0.3.6` / v35 for discovery attribution.
- Prior audits are historical evidence and attack-hypothesis sources only.
- Each Area requires genuinely fresh falsification-oriented evidence; rerunning an earlier adversary alone is insufficient.
- Preserve demonstrated findings before production repair.
- Search open and closed issues/PRs before creating a finding.
- Assign sequential identifiers `AV6-001`, `AV6-002`, ... only after a distinct scientific defect is demonstrated.
- Continue A–N discovery after a finding is recorded; ordinary production remediation begins only after full discovery completes.
- Record exact source SHA, model semantics, experiment configuration, seeds/replicates/horizon and quantitative result for numerical evidence.
- Keep evidence from semantically different source heads explicitly separated.
- Do not treat green CI, release gates, preserved benchmarks or prior re-verification as proof that an Area is clean.
- Do not introduce case-specific evidence, calibration or desired outcomes into this framework audit.

## Discovery coverage matrix

| ID | Area | Status | Fresh v6 emphasis / required direction |
|---|---|---|---|
| A | Authoritative semantics and scheduler behaviour | **not started** | Attack event ordering, simultaneous-process assumptions, hidden iteration priority, boundary timing, scientific-key/canonicalization effects and composition of repaired coupling semantics. |
| B | Demography, fertility, mortality, ageing, population structure | **not started** | Fresh limiting cases and structural controls for fertility/mortality competition, mate limitation, lifecycle timing, extinction/censoring and finite-population effects. |
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
| M | Documentation, TRACE/ODD/ODD+D, claim consistency | **not started** | Compare executable v35 semantics and current artifacts with all living scientific claims, explicit null-model boundaries, release statements and benchmark interpretation. |
| N | Cross-system integration | **not started** | Fresh coupled attacks across demography×households/resources, households×movement, movement/aggregation×resources, initialization×spatial/demography, inference×censoring, sensitivity×hidden config, calibration×identifiability, resume×RNG and observability×interpretation. |

An Area is not complete merely because no issue is found. Its evidence, files/mechanisms inspected, adversarial construction, quantitative result where relevant, neighbouring-system considerations and residual uncertainty must be recorded.

## Finding register

No Audit-v6 finding has been demonstrated yet.

When a finding is established, add one row per underlying scientific defect:

| Finding | Severity | Primary / cross-cutting Areas | Immutable-target evidence | Issue | Discovery state | Later remediation / re-verification |
|---|---:|---|---|---|---|---|
| `AV6-###` | P0/P1/P2/P3 | — | exact SHA, test/experiment, quantitative result | `#…` | demonstrated | pending until discovery completion |

Do not pre-allocate identifiers for hypotheses that have not been demonstrated.

## Area completion record template

For each completed Area, preserve enough information that a new agent can independently understand what was challenged and why the Area was closed:

- Area and date;
- immutable audit target and exact source SHA used;
- model semantics ID;
- live `main` observed at session start;
- overlapping open issues/PRs/branches checked;
- files and mechanisms inspected;
- fresh adversarial hypothesis/construction;
- tests/experiments executed and exact commands/workflows where relevant;
- seed policy, replicate count, horizon and configuration for numerical work;
- quantitative results/effect sizes/precision;
- earlier adversaries rerun as controls, clearly separated from fresh evidence;
- findings created or overlap with existing issues;
- neighbouring/cross-system interactions considered;
- unresolved uncertainties and explicit non-coverage;
- whether evidence must be repeated because live `main` changed;
- recommended next Area.

Detailed reports should be linked from the coverage matrix or ledger once created.

## Finding creation standard

Before creating an Audit-v6 finding issue:

1. demonstrate or otherwise establish the failure on immutable `v0.3.6` / v35, or document why a causally equivalent source is necessary;
2. search open and closed issues/PRs for the same underlying defect;
3. identify the smallest scientific failure rather than an implementation symptom;
4. assign severity by scientific consequence under the reusable protocol;
5. preserve exact reproduction/evidence before repair;
6. create the issue and assign the next sequential `AV6-*` identifier;
7. update this ledger before any production remediation.

The issue must state scientific consequence, affected identity, reproduction/evidence, expected contract, scope/non-goals, acceptance criteria and required tests/experiments.

## Discovery/remediation barrier

Audit-v6 discovery and remediation are deliberately separated.

Until Areas A–N are all complete:

- do not merge production scientific-semantic repairs for Audit-v6 findings;
- do not rewrite the immutable discovery baseline;
- do not close a finding merely because an experimental branch contains a possible fix;
- continue auditing the released target so that later findings are not hidden by mid-audit semantic changes.

After discovery completes, create a remediation plan ordered by severity/dependency. Every P0/P1 repair then requires normal production review/gates plus independent post-merge adversarial re-verification before the finding is considered fully closed.

## Convergence decision

At the end of A–N discovery:

- **P1-clean pass:** no new P0/P1 findings were demonstrated. P2/P3 findings still require explicit disposition, but the framework-convergence gate may be considered satisfied only after the full audit completion criteria and any required documentation reconciliation are met.
- **Non-clean pass:** one or more new P0/P1 findings were demonstrated. Complete discovery, remediate, independently reverify P0/P1 repairs, freeze the repaired line as a new immutable release, and require another fresh audit generation before the framework-convergence gate is satisfied.

Neither outcome constitutes empirical calibration or validation.

## Current handoff

Audit-v6 groundwork is initialized; **no substantive Area discovery has yet been credited**.

Next recommended action:

1. merge the initialization documentation after normal review/CI;
2. reconstruct live repository state from the merged head and immutable `v0.3.6`;
3. begin **Area A — authoritative semantics and scheduler behaviour** from zero coverage;
4. create fresh evidence rather than simply replaying Audit-v5 Area-A tests;
5. update this ledger before handoff.

## Cross-session start instruction

> Read `docs/research/scientific-audit-protocol.md`, `docs/research/audit-v6/README.md`, `docs/research/audit-reverification-version-drift.md`, and `docs/research/audit-v6/STATUS.md`. Verify live `main`, immutable `v0.3.6`, open issues/PRs and overlapping audit work. Continue the next incomplete Audit-v6 Area from first principles using genuinely fresh adversarial/quantitative evidence, preserve demonstrated defects before repair, do not begin production remediation before A–N discovery is complete, and update the repository-authoritative ledger before handoff.
