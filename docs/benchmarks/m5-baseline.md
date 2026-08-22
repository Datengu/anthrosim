# M5 checkpoint persistence baseline

**Milestone:** M5 — events, metrics, checkpoints and causal inspection  
**Purpose:** regression baseline for persistence cost, not a cross-machine performance guarantee  
**Environment:** GitHub-hosted Ubuntu 24.04 runner, Rust 1.97.1  
**Measured branch head:** `179b5d1eb1131e3f24aad2e83bb80329bd54c601`  
**CI run:** 116

M5 introduces deterministic annual-boundary checkpoints containing the dynamic population, resource and migration state, event/metric history and exact positions of all named stochastic streams. Persistence overhead is measured separately from the simulation lifecycle so later schema changes can be evaluated rather than treated as free.

## Checkpoint workload

The benchmark constructs a synthetic-validation run with 10,000 founders, advances it to completed year 10 and serializes the resulting `SimulationCheckpoint` with `serde_json`.

The measured checkpoint JSON size was:

- **2,589,495 bytes** (~2.47 MiB)

Criterion measurements on the hosted runner were:

| Operation | 95% measurement interval | Point estimate |
| --- | ---: | ---: |
| JSON serialize, 10k population at year 10 | 3.1312–3.3677 ms | 3.2422 ms |
| JSON deserialize, same checkpoint | 7.8428–7.9114 ms | 7.8758 ms |

These numbers are an engineering regression baseline only. They depend on runner hardware, operating-system scheduling, population history, event volume and the current human-readable JSON representation. They are not scientific model outputs and should not be used as a portability promise.

## Related final M5 validation

The same final read-only CI run also passed:

- uninterrupted-versus-checkpoint-resumed deterministic equivalence;
- checkpoint JSON round-trip followed by identical final execution;
- authoritative-event / derived-metric reconciliation;
- a real release-CLI `run --checkpoint-year 3` → `resume` smoke test with a complete offline run bundle;
- the existing world, population, migration and full resource–migration–demography regression benchmarks;
- the one-million-founder process-memory regression (78,820 kB maximum RSS on this runner).

The checkpoint benchmark is intentionally retained in CI as `checkpoint_persistence` so future changes to checkpoint content or serialization have an explicit measured cost.