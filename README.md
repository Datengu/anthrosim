# AnthroSim

AnthroSim is an experimental, reproducible agent-based simulation framework for exploring human demography, resource-constrained movement, kinship, evidence-grounded spatial experiments, and temporary mobility/aggregation under explicit model assumptions.

**v0.3.2 is the latest released baseline.** It is a documentation-convergence patch over the v0.3.1 scientific-audit implementation: the simulation semantics and scientific results are unchanged, while the living README, roadmap, scientific specification, ODD/ODD+D and TRACE dossier are synchronized with the actual post-audit state. v0.3.1 remains an immutable historical release whose code and verification were valid, but several living documents still described the earlier v0.3.0/pre-audit state when that tag was created.

The preserved M8.6 and M9.7 references are capability/regression baselines under declared assumptions, not archaeological validation. AnthroSim is not a civilisation game and it is not a validated model of human prehistory.

## Core rule

> Model causes, not historical outcomes.

If a pattern appears in AnthroSim, the goal is for it to be explainable from simulated conditions and agent behaviour rather than from a scripted "civilisation" rule.

## Current capabilities

AnthroSim v0.3.2 includes:

- deterministic synthetic spatial environments;
- persistent individuals, genealogy and households;
- birth, ageing, condition, death and renewable-resource scarcity;
- local interpretable **permanent** household migration;
- deterministic experiment configuration, replay and immutable provenance;
- versioned authoritative events, derived metric snapshots and resumable checkpoints;
- self-contained offline run bundles and a read-only local explorer;
- deterministic ensemble execution, retries and parameter sweeps;
- long-run invariant, performance, memory and cross-platform determinism gates;
- versioned normalized landscape inputs with explicit provenance;
- deterministic landscape-to-model transformations for movement cost, water access and resource opportunity;
- evidence catalogues tied to experiment identity and validated parameter paths;
- residence-based spatial observability for landscape-bound runs;
- explicit persistent residence versus temporary physical-presence state;
- identity-bearing focal regions and deterministic temporary travel cost/duration;
- outbound transit, visiting, return transit and journey-completion lifecycle state;
- duration-aware resource demand during temporary mobility;
- temporary-presence observability with resident, visitor, transit, person-day, peak, journey-duration and catchment measures;
- an evidence-grounded M8 terrain null-model benchmark and a controlled synthetic M9 aggregation benchmark;
- complete scientific-configuration exposure for reproducible ensemble/sensitivity experiments;
- stochastic replicate-sufficiency/Monte Carlo precision gates for quantitative conclusions;
- long-run drift/regime diagnostics and explicit equilibrium-claim safeguards;
- structural household/demographic sensitivity support, including the finding that no universal demographic baseline is currently justified;
- fail-closed identifiability/equifinality analysis that preserves acceptable regions and held-out discriminating predictions.

Culture, language, trade, states, religion, warfare, and AI-controlled agents remain deferred until a research question or validation target justifies adding them. M9 temporary mobility is deliberately a generic null mechanism rather than a cultural, ritual, political or economic motive model.

## Current milestone status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local permanent migration:** complete; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.
- **M5 — Events, metrics, checkpoints and causal inspection:** complete.
- **M6 — Local simulation explorer:** complete as a read-only artifact consumer.
- **M7 — Deterministic experiment orchestration, ensembles, retries, sweeps, soak/performance validation and the v0.1 reference experiment:** complete.
- **M8 — Evidence-grounded spatial experiments / v0.2.0:** complete and released. M8 adds normalized landscape inputs, explicit deterministic spatial transformations, landscape/evidence identity, residence-based spatial observability and the first evidence-grounded terrain null-model benchmark.
- **M9 — Temporary mobility and controlled aggregation / v0.3.0:** complete, audited and released. M9 adds persistent-residence/current-presence separation, identity-bearing focal regions, deterministic multi-day temporary journeys, duration-aware resource accounting, M9 observability and a predeclared continuous-residence versus intermittent-aggregation benchmark.
- **v0.3.1 post-M9 scientific hardening:** released. The first major audit backlog is resolved and its analysis/inference safeguards are preserved in that immutable release.
- **v0.3.2 documentation convergence:** released as a maintenance patch that synchronizes living scientific/project documentation with the already-implemented v0.3.1 state; no model semantics or frozen scientific results change.
- **Next validation phase:** further independent/adversarial audit convergence, followed by question-led model interrogation. No fixed M10 feature list is declared.

M1–M4 establish the baseline permanent-residence response loop: local supply and competition affect condition/survival, and surviving pressured households can make bounded permanent relocations. M5–M7 make that loop inspectable, resumable and reproducibly orchestrated without introducing a second simulation engine.

M8 keeps evidence separate from model-facing transformations and results. Its first terrain reference exercise found **fragile spatial structure** under the declared terrain-only null model: terrain materially perturbed some spatial outcomes, but direction was not stable across seeds. See [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md).

M9 does **not** reinterpret M4 migration. Permanent migration changes residence; temporary mobility changes physical presence while preserving residence. A temporary journey can be at residence, in outbound transit, visiting a focal region, in return transit, then complete. Transit deliberately has no authoritative per-day world cell. Resource demand is duration-weighted across those states, and M9 observability is separate from M8 residence-based spatial observability. The controlled M9.7 benchmark distinguished intermittent aggregation from continuous residence under its frozen synthetic assumptions; that is capability validation, not archaeological validation. See [`docs/research/temporary-mobility-v1.md`](docs/research/temporary-mobility-v1.md), [`docs/research/temporary-mobility-observability-v1.md`](docs/research/temporary-mobility-observability-v1.md), [`docs/research/m9-controlled-aggregation-benchmark-v1.md`](docs/research/m9-controlled-aggregation-benchmark-v1.md), and [`docs/research/m9-controlled-aggregation-benchmark-result.md`](docs/research/m9-controlled-aggregation-benchmark-result.md).

No historical destination, route, settlement, group or migration outcome is scripted into these loops.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. The CLI package contains multiple binaries, so local `cargo run` commands should explicitly select the main `anthrosim` binary.

Builds made inside a Git checkout automatically capture source provenance; no manual environment variable is required. A clean tracked tree records the exact commit SHA in `gitCommit`. A staged or unstaged tracked modification records `<sha>-dirty-<working-tree-digest>` and emits a build warning. Outside a Git checkout AnthroSim does not invent a revision and records `gitCommit: null`. Controlled build environments may still supply `ANTHROSIM_GIT_COMMIT` explicitly. See [`docs/source-provenance.md`](docs/source-provenance.md).

From the repository root, a small headless run can be executed with:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

For causal inspection, write a controlled completed run bundle instead:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m6-example
```

A completed directory contains `manifest.json`, `world.json`, `initial-population.json`, `events.json`, `metrics.json` and a final `checkpoint.json`, so analysis does not require a live database.

Open that run in the explorer with:

```text
python scripts/serve-explorer.py runs/m6-example
```

The server binds to `127.0.0.1:8765` by default and opens the local browser. It serves only fixed explorer assets and expected run artifacts, exposes no directory listing or write API, and rejects POST/PUT/DELETE. Use `--no-browser` if you do not want it to open a browser automatically.

A deliberately paused run can be explored **before** resuming it:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m5-resume --checkpoint-year 10
python scripts/serve-explorer.py runs/m5-resume
```

Resume later with:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- resume --checkpoint runs/m5-resume/checkpoint.json --run-dir runs/m5-resume
```

`initial-population.json` is always the day-zero founder state used by the explorer for full-history reconstruction. If a checkpoint is resumed into a **different** output directory, the completed bundle also retains `resume-start-population.json` as the population at the resume boundary. AnthroSim deterministically reconstructs/writes the true original `initial-population.json` into that new bundle; `resume-start-population.json` is boundary provenance and must never be treated as the founders.

The CLI exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--resource-seasonality-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

## Testing M8 landscape mode

A committed generic M8 landscape/mechanism fixture can be run directly from the repository:

```text
cargo run --release -p anthrosim-cli --bin anthrosim-landscape -- run \
  --landscape examples/landscape-loading/landscape.json \
  --mechanisms examples/landscape-loading/spatial-mechanisms.json \
  --years 25 \
  --population 1000 \
  --seed 1 \
  --run-dir runs/m8-landscape-example
```

This produces the ordinary run artifacts plus the preserved normalized landscape, spatial transformation configuration and landscape/spatial wrapper provenance. The source landscape and transformed authoritative `world.json` remain separate by design.

For the full M8.6 public benchmark definition and result, see [`docs/research/m8-first-evidence-grounded-benchmark.md`](docs/research/m8-first-evidence-grounded-benchmark.md), [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md), and `examples/m8-first-evidence-grounded-benchmark/`.

## Running deterministic ensembles

Launch an explicit deterministic seed set unattended:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 25 \
  --population 10000 \
  --seeds 1,2,3,5,8 \
  --run-dir runs/example-ensemble
```

Or use a consecutive seed range:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 25 \
  --population 10000 \
  --seed-start 100 \
  --seed-count 20 \
  --run-dir runs/example-range
```

A fresh ensemble writes `experiment-manifest.json` before child execution. That immutable, versioned manifest records the model identity and complete exact `ExperimentConfig` for every planned seed. `ensemble-plan.json` remains as the concise M7.1 planning view. Mutable per-run lifecycle records are written separately under `status/`.

Each seed receives its own stable directory such as `runs/seed-00000000000000000100/`, containing the ordinary completed run artifacts plus `completion.json`. The positive completion marker is written only after the child bundle succeeds. A run status becomes `completed` only when that bundle reconciles with the exact immutable experiment definition.

If execution is interrupted or one run fails, rerun the **same command and exact configuration** with `--retry`. Retry first requires exact equality with the stored immutable experiment manifest. It keeps provenance-valid completed runs without executing them again, reconciles interrupted/missing bundles as incomplete, and reruns only planned, failed or incomplete children. Partial child directories are removed before a retry attempt so old and new artifacts cannot be mixed. A completed bundle with conflicting provenance is treated as an integrity error instead of being silently overwritten.

The batch continues to later seeds after an individual child fails, but the overall command still exits unsuccessfully while any child is unsuccessful. Downstream analysis should treat only `completed` status records with provenance-valid bundles as successful results.

## Running deterministic parameter sweeps

M7 adds an explicit Cartesian parameter-grid layer. For example, this compares two M3 productivity settings and two seasonal-amplitude settings over the same four seeds:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-resource-seasonality-scale-permille 0,1000 \
  --run-dir runs/resource-sweep
```

Supported sweep dimensions include founder population, target household size, M3 productivity scale, M3 seasonal-amplitude scale, annual food need, migration enabled/disabled, migration radius and the M8 spatial execution path. A control that is not explicitly swept uses its ordinary base command value.

A fresh sweep writes immutable `sweep-manifest.json` before point execution. It records the exact base settings, declared dimension values, seed definition, model identity and every expanded parameter point. Each point then lives under `experiments/point-XXXXXX/` as a normal M7 experiment, with its own immutable `experiment-manifest.json`, status files, retries and completed bundles. Retrying a sweep requires the exact same definition plus `--retry`; a changed grid, seed set or base control is rejected before child execution.

The sweep root also contains a deliberately separate `analysis/` directory:

```text
analysis/runs.json
analysis/runs.csv
analysis/points.json
analysis/points.csv
analysis/summary.json
```

These are **derived analysis artifacts**, not authoritative simulation state. Failed, incomplete, planned or otherwise non-completed runs remain explicit in the run table and point status counts; they are never silently folded into means.

The CSV files are intentionally ordinary rectangular tables with no special Rust tooling required. Python `pandas.read_csv(...)`, base R `read.csv(...)`, or equivalent tools can consume them directly.

## Reproducing the v0.1 reference experiment

The canonical source definition is [`experiments/v0.1-resource-variability.json`](experiments/v0.1-resource-variability.json). Build normally from a clean Git checkout; the source revision is captured automatically:

```text
cargo build --locked --workspace --release
python3 scripts/run-versioned-sweep.py \
  experiments/v0.1-resource-variability.json \
  --binary target/release/anthrosim \
  --run-dir runs/v0.1-resource-variability
```

The launcher copies the exact definition, records its SHA-256, verifies the immutable sweep manifest against the requested seeds/settings/dimensions, and writes a reproduction record containing the model and source identity. It refuses a missing source identity or an automatically detected `-dirty` tracked tree. Controlled build systems can still provide `ANTHROSIM_GIT_COMMIT` explicitly. The full contract and Windows/PowerShell equivalent are in [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md), with the build-time source policy in [`docs/source-provenance.md`](docs/source-provenance.md).

See [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md) for the M7 provenance/retry/sweep contract, [`docs/research/resource-variability-v0.1.md`](docs/research/resource-variability-v0.1.md) for the synthetic reference experiment, [`docs/research/spatial-mechanisms-v1.md`](docs/research/spatial-mechanisms-v1.md) for M8 transformation semantics, [`docs/research/spatial-observability-v1.md`](docs/research/spatial-observability-v1.md) for residence-based M8 observability, [`docs/research/temporary-mobility-v1.md`](docs/research/temporary-mobility-v1.md) for the M9 residence/presence contract, [`docs/research/m9-temporary-travel-semantics-v1.md`](docs/research/m9-temporary-travel-semantics-v1.md) and [`docs/research/m9-duration-aware-resource-semantics-v1.md`](docs/research/m9-duration-aware-resource-semantics-v1.md) for travel/resource semantics, [`docs/research/temporary-mobility-observability-v1.md`](docs/research/temporary-mobility-observability-v1.md) for M9 physical-presence observability, and [`docs/roadmap.md`](docs/roadmap.md) for the question-led development direction after M9.

## Scientific status

AnthroSim is a **research-oriented simulation framework, not a validated anthropological or archaeological model**. Its synthetic demographic, resource, permanent-migration and temporary-mobility presets remain explicit model assumptions unless a particular experiment supplies and justifies stronger evidence grounding.

Released v0.2.0 demonstrated that the deterministic experiment engine can bind provenance-tracked real-world-derived spatial evidence, transform it through declared assumptions and analyse residence-based spatial outcomes reproducibly. Released v0.3.0 preserved that spatial path and added a separate software/model capability: persistent residence and temporary physical presence can generate distinguishable aggregation histories while preserving deterministic replay, checkpoint/resume and explicit resource accounting. Released v0.3.1 preserves those capabilities while adding post-audit scientific safeguards around configuration completeness, stochastic precision, long-run claims, structural sensitivity and identifiability/equifinality. Released v0.3.2 changes no simulation semantics; it corrects documentation drift so the living project documentation consistently describes that already-implemented scientific state.

Neither result makes a run a reconstruction of a real past population. Strong archaeological or anthropological claims still require question-specific evidence, calibration/validation where appropriate, uncertainty and sensitivity analysis, comparison against independent observations, discriminating predictions and domain review.

The first major post-M9 audit backlog is complete, but one completed audit is not evidence of scientific convergence. The next framework-level step is another genuinely independent/adversarial audit pass; after convergence improves, focused model interrogation and study-specific comparisons should identify which missing mechanism, competing assumption, comparison method or observation layer is scientifically useful next. See [`docs/roadmap.md`](docs/roadmap.md).

## Contributing and security

Contribution expectations are documented in [`CONTRIBUTING.md`](CONTRIBUTING.md). Please report security-sensitive issues according to [`SECURITY.md`](SECURITY.md) rather than publishing exploit details in a normal issue.

## License

AnthroSim is licensed under the [Apache License 2.0](LICENSE).
