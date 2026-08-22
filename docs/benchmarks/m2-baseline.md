# M2 performance and memory baseline

**Recorded:** 2026-08-22  
**Milestone:** M2 — persistent people, households and demography  
**Toolchain:** Rust 1.97.1, locked Cargo dependency graph  
**Environment:** GitHub-hosted Ubuntu 24.04 runner

These measurements are an engineering regression baseline for AnthroSim. GitHub-hosted runner hardware can vary between runs, so the values should be treated as approximate comparative evidence rather than hard cross-machine performance guarantees.

They are **not scientific validation** of the demographic model.

## Synthetic world generation

Criterion benchmark: `generate_synthetic_world_256x256`

- 65,536 cells
- observed interval: **13.821–14.071 ms**
- point estimate: **13.921 ms**

## Persistent population initialization

Criterion benchmark: `population_initialization`

| Founder records | Observed interval | Point estimate |
| ---: | ---: | ---: |
| 10,000 | 273.55–313.63 µs | 288.75 µs |
| 100,000 | 5.0842–5.1649 ms | 5.1194 ms |
| 1,000,000 | 38.644–39.066 ms | 38.866 ms |

These runs initialize packed persistent person state, synthetic co-resident households, founder locations, reproductive-sex state, age/birth-day state, parent placeholders, and the cell occupancy index.

## Dynamic M2 demographic lifecycle

Criterion benchmark: `m2_10k_people_25_year_demography_run`

Configuration:

- 10,000 synthetic founders
- 64×64 synthetic world
- 25 simulated years
- `synthetic_validation_v1` demographic schedule
- 250,000 operational persistent-person-record ceiling
- mortality, fertility, local parent selection, births, deaths, genealogy state and occupancy maintenance enabled

Result:

- observed interval: **16.242–16.838 ms**
- point estimate: **16.523 ms**

This is a throughput baseline for the current M2 implementation, not a claim about future full-model speed. M3+ will add resource, ecology, movement and other work to each simulated period.

## One-million-founder process memory

The release CLI was measured end-to-end with GNU `/usr/bin/time -v`:

```text
anthrosim run
  --years 0
  --world-width 64
  --world-height 64
  --population 1000000
  --max-person-records 1000000
```

Observed:

- wall-clock initialization/run: **0.12 s**
- maximum resident set size: **70,596 kB** (about **68.9 MiB**)
- swaps: **0**

The RSS measurement is intentionally an end-to-end process measurement. It includes the executable/runtime, world state, population state, household state, occupancy structures and other process overhead; it is **not** a measurement of raw person-vector storage alone.

## Regression policy

Future hot-path changes should compare against the relevant benchmark before merge. Small differences on hosted runners should not be over-interpreted. Material regressions should either be corrected or documented with the correctness/scientific capability gained in exchange.

Performance measurements do not substitute for model verification, empirical validation, sensitivity analysis or uncertainty quantification.
