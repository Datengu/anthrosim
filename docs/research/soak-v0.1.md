# M7.4 long-run soak and invariant hardening

M7.4 is an engineering-hardening milestone. It does **not** calibrate AnthroSim, validate its anthropological assumptions, or make the `synthetic_validation_v1` parameters empirical. Its purpose is narrower: run the executable model for long durations and difficult edge cases, then fail loudly if authoritative state, provenance or accounting stops reconciling.

## Cross-artifact invariant validator

`SimulationCheckpoint::validate_invariants()` and `RecordedRun::validate_invariants()` provide one explicit correctness boundary for soak tests and downstream verification code.

The validator reconstructs the deterministic world from the stored experiment seed/config and checks the following layers together:

- checkpoint/model/config schema identity and simulation-time bounds;
- deterministic world digest reconstruction;
- persistent population column lengths and population accounting;
- living-person, household-location and occupancy-index consistency;
- genealogy validity, including parent existence, parent sex, parent age ordering and no births after parent death;
- condition bounds and person-record ceiling enforcement;
- renewable-resource checkpoint validity and stock conservation (`initial + regenerated = harvested + final stock`);
- resource-period counts against elapsed scheduled boundaries;
- migration counter ordering, decision-boundary limits and disabled-migration zero-activity rules;
- migration Manhattan-distance accounting, directional-step reconciliation, score ceilings and bounded local decision traces;
- authoritative event sequence/provenance/time ordering;
- birth/death/migration event references against persistent state;
- authoritative event totals against population, scarcity-death and migration summaries;
- terminal derived-metric reconciliation with authoritative subsystem summaries and the final state digest;
- final state-digest reconstruction from world, population, resources, migration and time;
- for completed runs, manifest/final-checkpoint equality, artifact-schema identity, statistics and stop-reason semantics.

The validator is intentionally strict. M7.4 does not weaken an invariant to make a soak pass. A failure means either the simulator has exposed a correctness defect or the invariant itself has made an invalid claim and must be justified explicitly before changing it.

## Automated soak scenarios

The normal workspace test suite now contains long-run/adversarial M7.4 coverage.

`crates/anthrosim-core/tests/m7_4_soak.rs` includes:

1. **1,000-year stable-state soak** — 64 persistent founders, no demographic events and no resource pressure. This deliberately exercises 4,000 resource/migration boundaries while the authoritative population should remain unchanged.
2. **120-year dynamic checkpoint/resume soak** — non-zero deterministic births and deaths with an annual checkpoint at year 60. The resumed run must be byte-for-structure identical to uninterrupted execution at the final manifest/checkpoint boundary.
3. **Adversarial resource-pressure seeds** — four deterministic seeds under low productivity, high need and strong scarcity effects. Early extinction, duration completion or a record-limit stop are all valid terminal outcomes, but each run must remain invariant-valid and replay identically for the same seed/config.
4. **Explicit terminal-state cases** — guaranteed demographic extinction and guaranteed person-record-limit pressure are tested as deterministic, named stop conditions rather than generic failures.
5. **Tampering detection** — a cross-artifact migration accounting mutation must be rejected by the invariant validator.

`crates/anthrosim-cli/tests/m7_4_ensemble_soak.rs` additionally launches a three-seed, 150-year run through the real `anthrosim ensemble` command. Every M7.2 child must finish with `completed` lifecycle status and its ordinary M5 manifest/checkpoint pair must pass the M7.4 invariant validator. This ensures the soak path covers experiment orchestration rather than only direct core calls.

These tests are intentionally small enough to remain part of ordinary CI. They are correctness soaks, not M7.5 performance acceptance tests.

## Running the automated soaks directly

From the repository root:

```text
cargo test --locked -p anthrosim-core --test m7_4_soak
cargo test --locked -p anthrosim-cli --test m7_4_ensemble_soak
```

The ordinary workspace CI command also includes them:

```text
cargo test --locked --workspace --all-targets --all-features
```

## Deeper local experiment soaks

M7.1–M7.3 can be used for larger unattended exploratory soaks without adding another simulation path. For example:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 1000 \
  --world-width 32 \
  --world-height 32 \
  --population 1000 \
  --max-person-records 250000 \
  --seed-start 10000 \
  --seed-count 20 \
  --run-dir runs/soak-1000y
```

Or deliberately combine stressful resource/migration settings through a sweep:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- sweep \
  --years 500 \
  --world-width 32 \
  --world-height 32 \
  --population 1000 \
  --max-person-records 250000 \
  --seeds 20001,20002,20003,20004 \
  --sweep-resource-productivity-scale-permille 100,300,700,1000 \
  --sweep-annual-food-need 50,100,200 \
  --sweep-migration-radius 1,3,8 \
  --run-dir runs/adversarial-soak
```

These larger commands inherit M7.2 immutable experiment identity, lifecycle/retry semantics and M7.3 traceable derived tables. A terminal `PopulationExtinct` or `PersonRecordLimitReached` manifest is a model/operational outcome, not an orchestration failure; a failed/incomplete M7 lifecycle state remains a failed/incomplete result.

## Checkpoint/resume boundary

The resumable checkpoint contract remains an **annual completed boundary**. M7.4 explicitly compares a long supported annual checkpoint/resume path against uninterrupted execution. Final completed-run checkpoints may also describe an early terminal state reached between annual boundaries (for example resource-driven extinction); such final artifacts remain valid evidence of the completed run but are not automatically a supported resume point.

Checkpoint schema v3 serializes an optional `terminalStopReason`. Ordinary annual checkpoints carry no terminal reason and remain resumable exactly as before. If `checkpoint_at_year(...)` reaches `PersonRecordLimitReached` exactly on the requested annual boundary, the returned checkpoint is explicitly marked `PersonRecordLimitReached` instead of looking like an ordinary resumable checkpoint. `Simulation::from_checkpoint(...)` validates that marker against the authoritative state, and subsequent execution honors it without advancing the model: `run_recorded()` reproduces the same terminal boundary and a request for a later checkpoint target fails because the terminal state cannot progress to that target.

Completed-run checkpoints also carry the manifest's terminal reason. This makes record-limit termination self-describing without trying to infer it from `personRecords == maxPersonRecords`, which is not by itself proof that the model terminated there. Non-annual terminal completed-run checkpoints remain final evidence rather than supported resume points because the loader still enforces the annual checkpoint boundary.

## Remaining limits after M7.4

Passing M7.4 means the covered configurations survived the stated correctness checks. It does not prove that every possible configuration is safe, that there are no undiscovered overflows at much larger scales, or that long runs are fast enough for research workloads.

In particular:

- the full combinatorial parameter space is not exhaustively enumerated;
- CI soaks are deliberately bounded so they remain practical on every pull request;
- M7.5 owns explicit performance and memory acceptance benchmarking;
- M7.6 owns the first named resource-variability experiment and v0.1 closure;
- anthropological plausibility, calibration and empirical validation remain separate scientific work.

Any correctness failure found by a larger soak should be reduced to a deterministic seed/config and added to this automated regression set before the defect is considered closed.
