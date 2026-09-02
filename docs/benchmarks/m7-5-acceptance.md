# M7.5 performance and memory acceptance baseline

**Milestone:** M7.5 — performance and memory acceptance benchmarking  
**Purpose:** v0.1 engineering acceptance and regression protection, not scientific validation  
**Environment:** GitHub-hosted Ubuntu 24.04, x86-64 Azure runner, 2 logical CPUs, Rust 1.97.1  
**Initial measured PR head:** `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`  
**Initial measurement CI run:** 169

M7.5 turns AnthroSim's earlier micro/regression benchmarks into an explicit end-to-end v0.1 performance envelope. The requirement being tested is the v0.1 specification's baseline that a 10,000-person simulation can execute for thousands of simulated years on ordinary CPU hardware without requiring a database, GPU, distributed execution or a nondeterministic fast path.

This document records engineering behaviour only. It does **not** imply that `synthetic_validation_v1` is calibrated, anthropologically valid, or representative of any real prehistoric population.

## Canonical acceptance workload

The automated acceptance workload uses:

- 10,000 synthetic founders;
- 64 × 64 synthetic world;
- 2,000 requested simulated years;
- seed `1,847,291`;
- 1,000,000 persistent-person-record operational ceiling;
- default `synthetic_validation_v1` demography;
- default `synthetic_validation_v1` resources;
- default enabled M4 migration with radius 3;
- the release `anthrosim run` CLI path with manifest output.

The manifest-only CLI form is **not** a stripped simulator. `Simulation::run()` delegates to `run_recorded()`, so M5 authoritative events and annual/terminal metrics are retained during execution even though only the final manifest is written to disk. The benchmark therefore exercises the normal observable v0.1 simulation lifecycle rather than a special performance-only path.

A performance pass requires `StopReason::DurationReached` and exactly 730,000 simulated days. Extinction or the person-record ceiling cannot be counted as a successful 2,000-year benchmark merely because the process exited normally.

## Initial hosted-runner result

The first full measurement on CI run 169 produced:

| Measurement | Result |
| --- | ---: |
| Requested / completed years | 2,000 / 2,000 |
| Wall time | **72.91 s** |
| CPU time | **72.90 s** |
| Simulated-years throughput | **27.43 years/s** |
| Authoritative events | **841,101** |
| Authoritative-event throughput | **11,536 events/s** |
| Metric snapshots | 2,000 |
| Maximum resident set size | **134.23 MiB** |
| Final persistent person records | 318,524 |
| Final living population | 3,761 |
| Approx. whole-process RSS / persistent record | ~442 bytes |
| Final deterministic state digest | `15479739076504566673` |

The per-record RSS figure is deliberately labelled approximate. Whole-process RSS includes the executable/runtime, world and resource state, household/migration scratch storage, accumulated M5 event history, metrics and allocator overhead. It is not the packed person-vector size.

## CI acceptance envelope

Hosted runners vary in CPU allocation, scheduling and memory behaviour. M7.5 therefore does not gate against the exact point measurement above. The automated end-to-end gate uses deliberately broader limits:

- **at least 20 simulated years/second**;
- **at most 100 seconds wall time** for the canonical 2,000-year run;
- **at most 256 MiB maximum RSS**;
- the run must reach the complete requested 2,000-year duration.

Those limits are strict enough to detect a material regression from the measured baseline while leaving reasonable headroom for normal GitHub-hosted-runner noise. They are not a contractual performance guarantee for arbitrary machines or future model versions.

The existing one-million-founder initialization/process-memory check is also now an acceptance gate rather than log-only measurement. It retains the release `--years 0` scenario and fails above **160 MiB maximum RSS**. Earlier M4/M5 hosted-runner observations were approximately 77 MiB, so the ceiling allows substantial runner noise while still catching major packed-state/migration-scratch regressions.

## Reproducing the end-to-end measurement

Build the pinned release binary, then run:

```text
cargo build --locked --workspace --release
python3 scripts/benchmark-performance-acceptance.py \
  --binary target/release/anthrosim \
  --report /tmp/anthrosim-m7-5-performance.json \
  --enforce
```

The JSON report records:

- exact benchmark configuration;
- platform, architecture and logical CPU count;
- model/source identity when available;
- stop reason and final deterministic digest;
- wall and CPU time;
- simulated years/second;
- authoritative events/second;
- maximum RSS;
- final persistent/living population counts;
- each acceptance check and its threshold.

Arguments can be overridden for exploratory local profiling, but the default values define the v0.1 acceptance profile used by CI.

## Relationship to the existing benchmark suite

M7.5 keeps the existing Criterion regressions because they localise changes better than one end-to-end number:

- synthetic world generation;
- 10k / 100k / 1m founder initialization;
- bounded M4 migration candidate discovery;
- 10k-person 25-year resource/migration/demography lifecycle;
- M5 checkpoint JSON serialization/deserialization.

The new acceptance profile answers a different question: **can the complete current v0.1 executable sustain the intended research-scale duration and population baseline within a practical CPU/memory envelope?**

Both layers matter. Criterion is useful for locating regressions; the M7.5 process benchmark verifies that local measurements still compose into a practical complete run.

## Optimization decision

No hot-path optimization was made for M7.5.

The measured canonical workload completed successfully on a 2-vCPU hosted runner in about 73 seconds with about 134 MiB peak RSS. That already satisfies the v0.1 requirement. Changing data structures, RNG ordering, event semantics, numerical representation, migration logic, or introducing parallel/GPU execution solely to improve an already-acceptable number would create scientific and deterministic risk without a demonstrated need.

Future optimization should begin with a new measured bottleneck or a stricter research workload. Before/after evidence must accompany any such change, and deterministic state/output semantics remain non-negotiable unless separately versioned and justified.

## Interpretation and limits

M7.5 supports these engineering claims for v0.1:

- 10,000-founder, multi-thousand-year execution is practical on modest CPU hardware;
- the full event/metric-observable path, not a stripped simulation path, meets that target;
- representative long-run process memory remains well below 256 MiB in the canonical scenario;
- one-million-founder initialization remains within a separately gated compact-memory envelope;
- no GPU, distributed system, database hot loop or nondeterministic shortcut is required for the v0.1 target.

It does **not** establish:

- performance for every parameter combination or population trajectory;
- a guarantee that all 10,000 individuals remain alive throughout the run (10,000 is the founder/baseline functional target, and demography/resources remain active);
- cross-platform identical wall-clock performance;
- empirical realism or anthropological correctness;
- a promise that future richer cultural/ecological models will fit the same envelope.

M7.6 owns the first named resource-variability experiment and v0.1 closure. M7.5's job is only to establish that the current deterministic engine is fast and compact enough to support that next scientific-validation step without performance shortcuts.
