# M4 performance baseline

**Milestone:** M4 — interpretable local migration  
**Purpose:** regression baseline, not a scientific validation target  
**Environment:** GitHub-hosted Ubuntu 24.04 runner, x86-64, Rust 1.97.1

These measurements record the approximate cost of the M4 implementation on the hosted CI environment used during milestone development. They are useful for detecting large regressions within comparable CI runs; they are **not** hard cross-machine performance guarantees and do not validate any anthropological assumption.

## Full M4 lifecycle

Criterion benchmark:

`m4_10k_people_25_year_resource_migration_demography_run`

Configuration:

- 10,000 synthetic founders;
- 64 × 64 synthetic world;
- 25 simulated years;
- default `synthetic_validation_v1` resource model;
- default `synthetic_validation_v1` migration model;
- quarterly resource and migration boundaries;
- annual M2 demographic boundary.

Measured time:

```text
[72.043 ms 73.669 ms 75.534 ms]
```

The middle Criterion estimate was approximately **73.7 ms** for the complete 25-year run on this runner. M3's corresponding resource-demographic baseline was roughly 40 ms on a comparable hosted runner, so M4's bounded household decision and relocation work increases the target workload substantially but remains in the tens-of-milliseconds range.

The comparison should be treated as approximate because hosted-runner noise and hardware allocation vary between CI jobs.

## Bounded candidate discovery

Criterion benchmark:

`m4_candidate_lookup_radius_3`

Configuration:

- 128 × 128 world;
- interior origin cell;
- Manhattan candidate radius 3;
- at most 24 destination cells are returned.

Measured time:

```text
[70.963 ns 73.316 ns 76.299 ns]
```

The benchmark exists to guard the architectural property that candidate discovery is bounded by the configured information radius rather than by total world area. It does **not** benchmark full utility evaluation or household relocation.

## Existing initialization baselines on the M4 branch

The same CI run measured synthetic founder initialization at:

```text
10,000 founders:    [259.68 µs 285.18 µs 314.78 µs]
100,000 founders:   [5.3887 ms 5.4155 ms 5.4461 ms]
1,000,000 founders: [39.899 ms 40.155 ms 40.523 ms]
```

World generation for a 256 × 256 synthetic world measured:

```text
[15.606 ms 16.039 ms 16.551 ms]
```

These measurements are retained primarily to detect whether M4 accidentally regresses unrelated initialization/world-generation paths.

## Million-founder process memory check

A release build initialized 1,000,000 founders in a 64 × 64 world with `--years 0` and a one-million-record ceiling.

Observed process result:

```text
elapsed wall time: 0.14 s
maximum resident set size: 78,656 kB (~76.8 MiB)
```

This is whole-process RSS, not a direct measurement of authoritative person vectors alone. It includes the executable/runtime, world/resource state and M4's household/cell migration scratch arrays.

For comparison, the M3 process-level million-founder check was about 70.5 MB on a hosted runner. The additional M4 memory is expected because migration keeps reusable per-household and per-cell scratch arrays, but the design remains compact and avoids per-person heap-allocated decision objects.

## Interpretation

The performance evidence supports these engineering claims for M4:

- the 10,000-person v0.1 target remains comfortably runnable on a normal CPU;
- bounded radius-three destination discovery is extremely small compared with the full lifecycle cost;
- migration work does not require a global all-cell destination search for each household;
- one-million-founder initialization remains feasible within modest process memory;
- future optimizations should be driven by measured regression or larger experiment requirements rather than premature unsafe/SIMD/GPU complexity.

It does **not** establish empirical realism, a scientifically correct temporal resolution, or a universal scalability guarantee for future richer AnthroSim models.
