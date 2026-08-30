# Experiment execution in v0.1

This document records the M7.1–M7.6 batch, ensemble, sweep, downstream-analysis and versioned-reproduction contract. It describes experiment orchestration and provenance only: it does not add an anthropological mechanism and does not create a second simulation implementation.

## Purpose

A useful research simulator must support repeated stochastic runs without requiring a person to relaunch each seed by hand, while making it impossible to confuse a retry with a different experiment. M7.1 added deterministic seed ensembles around the existing AnthroSim run lifecycle. M7.2 added immutable experiment identity, explicit per-run lifecycle state, reconciliation after interruption and deterministic retry behaviour. M7.3 added deterministic parameter-grid expansion and explicitly derived analysis tables. M7.4 and M7.5 harden that same path with long-run invariants and performance/memory acceptance. M7.6 adds a reviewable versioned experiment-definition layer for the first documented synthetic resource-variability exercise and records enough provenance for third-party reproduction.

The scientific unit remains an ordinary AnthroSim run. Every child run has its own exact `ExperimentConfig`, authoritative M5 artifacts and final run manifest. The orchestration layer decides which runs were requested, records their immutable identities, tracks execution state and decides whether an exact retry is permitted. Sweep-level analysis consumes those run results; it never becomes authoritative simulation state.

## Seed definitions

An ensemble or sweep has exactly one of these seed definitions:

- an explicit ordered seed set supplied with `--seeds`, for example `--seeds 2,5,11,19`;
- a consecutive range supplied with `--seed-start` and `--seed-count`, for example `--seed-start 100 --seed-count 20`.

Duplicate explicit seeds are rejected because two identical seeds would target the same stable child directory. Zero-length ranges and ranges that overflow `u64` are also rejected.

The same seed definition and shared run controls produce the same ordered plan. AnthroSim intentionally preserves the order supplied by an explicit seed set rather than silently sorting it.

## Immutable experiment manifest

Before any child simulation starts, a fresh ensemble writes `experiment-manifest.json`. This is the authoritative experiment identity for M7.2. It contains:

- an experiment-manifest schema version;
- a deterministic experiment ID;
- the AnthroSim model/package version;
- the build source revision in `gitCommit` when Git metadata or an explicit override is available;
- every planned run ID and stable output path;
- the complete exact `ExperimentConfig` for every child, including its seed.

Ordinary builds from a Git checkout resolve the source revision automatically. A clean tracked tree records the exact commit SHA; staged or unstaged tracked modifications record `<sha>-dirty-<working-tree-digest>`; builds outside Git record `null` rather than fabricating an identity. A controlled build can still set `ANTHROSIM_GIT_COMMIT` explicitly. See [`source-provenance.md`](source-provenance.md) for the full policy.

The experiment ID is a stable FNV-1a fingerprint of the versioned identity payload. It is an identifier and accidental-change detector, not a cryptographic signature. Retry safety does not rely on the fingerprint alone: AnthroSim deserializes the stored manifest and requires exact structural equality with the experiment definition requested by the retry command.

AnthroSim never rewrites `experiment-manifest.json` during retry. Changing a seed, duration, world size, population setting, resource control, migration control, model version, git identity or any other serialized run configuration causes `--retry` to fail before child artifacts are touched.

`ensemble-plan.json` is retained as the concise M7.1 planning view for compatibility. `experiment-manifest.json` is the authoritative M7.2 provenance record.

## Stable output layout

For seed `42`, the child directory remains:

```text
runs/seed-00000000000000000042/
```

Each completed child directory contains the normal M5 completed bundle:

```text
manifest.json
world.json
initial-population.json
events.json
metrics.json
checkpoint.json
```

It also contains the M7.1-compatible positive completion marker:

```text
completion.json
```

`completion.json` is written only after all ordinary M5 child artifacts have been written successfully.

Mutable M7.2 lifecycle state is deliberately kept outside the child bundle:

```text
status/seed-00000000000000000042.json
```

This separation lets an incomplete child directory be discarded and deterministically rebuilt on retry without mutating the immutable experiment definition.

## Run lifecycle

Every planned child has an explicit versioned status record with:

- experiment ID, run ID and seed;
- lifecycle state;
- execution attempt number;
- optional failure/reconciliation message;
- for completed runs, the child manifest path and final state digest.

The lifecycle states are:

- `planned` — requested but not yet started;
- `running` — an attempt has begun;
- `completed` — the completed bundle and its provenance have reconciled successfully;
- `failed` — the last execution attempt returned an error;
- `incomplete` — a prior state or bundle does not prove successful completion and must be reconciled/retried.

A fresh experiment writes all planned status records before executing the first child. Immediately before a child starts, its status moves to `running` and its attempt number increments. It becomes `completed` only after the full child bundle and completion marker have been written. An execution error becomes `failed`, with the error text retained in the status record.

A batch continues to later planned seeds after one child fails, so unattended ensembles do not lose all remaining work because of one failed run. The overall command still exits unsuccessfully if any child remains failed, preventing a partially successful experiment from masquerading as fully successful.

## Retry and reconciliation

Retry uses the same ensemble command and exact original definition plus `--retry`:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4,5 \
  --run-dir runs/resource-baseline-seeds \
  --retry
```

The requested definition must exactly match the stored immutable manifest. AnthroSim then reconciles each run independently.

A run is kept without re-execution only when its ordinary M5 bundle is complete and provenance-valid. Reconciliation requires all six normal completed artifacts plus `completion.json`, then verifies that the child `manifest.json` and final `checkpoint.json` agree with the exact planned `ExperimentConfig`, model identity and final state digest.

A valid completed bundle is retained even if its mutable status was left at `running` by an interruption after the bundle finished. A `completed` status with a missing artifact is downgraded to `incomplete` and rerun. Failed, incomplete and genuinely unfinished runs are retried from the exact planned configuration. Before such a retry, the incomplete child directory is removed so partial artifacts cannot be mixed with the new attempt.

A completed bundle whose serialized provenance conflicts with the immutable experiment is treated as an integrity error, not silently deleted or overwritten.

Re-running `--retry` after every child is already valid is idempotent: completed bundles are kept, attempt numbers do not increase and `experiment-manifest.json` is unchanged.

## Duplicate execution safeguards

M7.2 uses three layers to avoid accidental duplicate or ambiguous execution:

1. A fresh ensemble still refuses a non-empty output root.
2. Retry requires exact equality with the existing immutable experiment manifest.
3. Reconciliation keeps a provenance-valid completed child instead of executing it again.

These rules mean a retry can repair an interrupted experiment without silently changing what experiment was requested or overwriting a valid completed result.

## Parameter sweeps

A sweep is an explicit deterministic Cartesian product over supported experiment controls. In v0.1 the sweepable controls are:

- founder population;
- target household size;
- M3 resource productivity scale;
- M3 resource seasonal-amplitude scale;
- annual food need;
- M4 migration enabled/disabled state;
- M4 local migration radius.

A dimension that is not supplied uses its ordinary base command value. Supplying a dimension preserves the explicit value order. The Cartesian expansion order is fixed by the implementation and therefore produces the same point sequence for the same sweep definition. Duplicate values inside one dimension are rejected because they would create scientifically redundant parameter points with indistinguishable configurations.

For example:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-resource-seasonality-scale-permille 0,1000 \
  --run-dir runs/resource-sweep
```

This expands to four parameter points. Each point is then executed over seeds 1–4, for 16 planned ordinary AnthroSim runs.

AnthroSim deliberately limits planning to 100,000 parameter points so an accidental combinatorial explosion is rejected before expensive execution begins. This is a planning safeguard, not a performance claim.

## Immutable sweep manifest

Before any point experiment starts, a fresh sweep writes `sweep-manifest.json`. It records:

- a sweep-manifest schema version;
- a deterministic sweep ID;
- model/package and build source identity;
- the exact seed definition;
- all base controls;
- every explicitly declared sweep dimension and value order;
- every expanded point ID, stable point path and exact point settings.

The sweep ID fingerprints the complete versioned identity payload. As with experiment IDs, the fingerprint is an identifier rather than a cryptographic signature. Retry safety requires exact structural equality with the stored sweep manifest.

A sweep retry uses the identical command plus `--retry`. Changing a seed, base control, dimension, dimension value order, model version or build git identity causes retry to fail before point execution. The stored `sweep-manifest.json` is never rewritten during retry.

## Sweep point layout and relationship to M7.2

The sweep layer does not invent a new child-run format. Each expanded point is an ordinary M7.2 experiment:

```text
experiments/point-000000/
  experiment-manifest.json
  ensemble-plan.json
  status/
  runs/
```

That means every sweep run retains the same exact per-run `ExperimentConfig`, M5 bundle, completion marker, lifecycle status and retry/reconciliation guarantees as an ordinary M7.2 ensemble.

On retry, a point that already has its experiment manifest is reconciled through M7.2. A point that had never begun can be created fresh from the immutable top-level sweep definition. A non-empty point directory without its expected experiment manifest is not silently repurposed.

## Derived analysis outputs

After point execution AnthroSim writes a separate `analysis/` directory:

```text
analysis/runs.json
analysis/runs.csv
analysis/points.json
analysis/points.csv
analysis/summary.json
```

These files are explicitly labelled `derived`. They are not checkpoints, run manifests, events, metrics or any other authoritative simulation state.

The current run-row and top-level analysis-summary wire contracts use derived-analysis schema v6. The current point-table contract uses schema v7. Run/summary v6 adds the exact origin/destination resource and water-security score totals preserved by the completed run checkpoint so downstream pooled migration-quality statistics do not have to reconstruct events or multiply rounded run means. Point v7 makes migration-quality weighting explicit, retains the existing equal-run-weighted estimand under unambiguous names, and adds exact pooled per-completed-move means plus their move-count and score-total support. Historical v5/v6 artifacts retain their original identities and field meanings.

`runs.json` and `runs.csv` contain one row for **every planned run**, not just scientifically aggregated outcomes. Each row records:

- sweep, point, experiment and run identity;
- seed and exact swept control values;
- current lifecycle state and attempt count;
- relative source status path;
- for completed runs, the source run-manifest path;
- the machine-readable `scientificAggregationStatus`;
- an `operationalCensoringReason` when an engineering stop censors the scientific trajectory;
- achieved `simulatedDays` and exact `endDay` for completed runs;
- selected terminal descriptive values copied from the provenance-valid completed run manifest, including population, occupied-cell, resource-stress and migration outcomes;
- for completed runs, exact `migrationOriginResourceScoreTotal`, `migrationDestinationResourceScoreTotal`, `migrationOriginWaterSecurityScoreTotal`, and `migrationDestinationWaterSecurityScoreTotal` values copied from the authoritative checkpoint's retained M4 accounting state. Non-completed rows retain these as `null`.

Lifecycle completion and scientific aggregation eligibility are deliberately separate concepts. `durationReached` and `populationExtinct` are classified as `eligibleScientificOutcome`: extinction is a genuine modeled outcome and remains in the default point population even though it can end before the requested duration. `personRecordLimitReached` is classified as `operationallyCensored` because `maxPersonRecords` is an engineering/safety ceiling rather than a scientific terminal condition. The censored row and all of its terminal values remain visible for diagnosis and alternative downstream estimands, but it does not enter the default scientific point aggregates.

If a run is failed, incomplete, planned, running or otherwise not completed, that row remains present with `scientificAggregationStatus = notLifecycleComplete` and terminal result fields remain empty. The sweep layer therefore never hides a missing or censored run by simply omitting it from the dataset.

`points.json` and `points.csv` contain one row per parameter point. Every row repeats all seven currently sweepable design variables so the point table remains interpretable when exported separately from `sweep-manifest.json`: founder population (`initialPopulation` / `initial_population`), target household size (`householdSize` / `household_size`), resource productivity scale (`resourceProductivityScalePermille` / `resource_productivity_scale_permille`), resource seasonal-amplitude scale (`resourceSeasonalityScalePermille` / `resource_seasonality_scale_permille`), annual food need (`annualFoodNeed` / `annual_food_need`), migration enabled/disabled control (`disableMigration` / `disable_migration`), and migration radius (`migrationRadius` / `migration_radius`). The immutable sweep manifest remains authoritative for the complete base configuration, explicit dimension declarations, value ordering and full provenance; the repeated point fields make the manipulated design matrix directly usable rather than replacing that manifest.

Point rows also report planned, lifecycle-completed, failed, incomplete and other non-completed counts; explicit counts for duration reached, population extinction and the operational person-record limit; and separate `scientificallyEligibleRuns` and `operationallyCensoredRuns` counts. Default descriptive outputs include final living population, final occupied-cell count, births, deaths, living condition, condition-mediated deaths, unmet resource need, migration moves and migration distance over the scientifically eligible population only. A pooled scientifically-eligible-only migration-distance-per-move value is also emitted where eligible runs contain moves.

Migration quality has **two deliberately distinct point-level estimands**. The `runWeightedMeanOfRunMeanMigration...MoveObservedRunsOnly` fields answer: *what is the average move-observed run's mean migration quality?* Every scientifically eligible run with at least one completed move contributes one equal-weight run-level mean, regardless of whether that run contains 1 move or 1,000; scientifically eligible zero-move runs are outside this conditional run-weighted mean. The `pooledMeanMigration...PerMoveScientificallyEligibleOnly` fields instead answer: *what is the average quality of all completed moves generated by the scientifically eligible runs at this point?* They divide the exact summed checkpoint score total by `migrationMovesCompletedScientificallyEligibleOnly`. Zero-move eligible runs add zero events to that numerator and denominator. When the total eligible move count is zero, both quality estimands remain `null` rather than becoming zero.

The standalone point table also preserves the exact pooled support: `migrationMovesCompletedScientificallyEligibleOnly` plus the four `migration...ScoreTotalScientificallyEligibleOnly` numerator fields. A consumer can therefore reconstruct every pooled migration-quality mean from `points.json` or `points.csv` alone and cannot reasonably infer that the run-weighted and move-weighted values are the same statistic. Unequal move counts can change their numerical value or even reverse a treatment ranking; neither weighting is universally preferred, so both remain explicit rather than one silently replacing the other.

The implementation has regression coverage that derives the set of supported fields from the serialized `SweepDimensions` contract and requires every one to have both a point-level JSON field and a CSV column. Adding a future sweep dimension without adding its point-analysis representation therefore fails the analysis tests instead of silently producing an incomplete design matrix.

The aggregation population is explicit in names such as `meanFinalLivingPopulationScientificallyEligibleOnly`. `sourceScientificallyEligibleRunIds` lists exactly which runs contributed, while `sourceOperationallyCensoredRunIds` identifies lifecycle-completed runs that were preserved but excluded. Thus lowering `maxPersonRecords` cannot silently pull a truncated trajectory into the same scientific mean merely because its artifact bundle completed.

`analysis/summary.json` records run and point row counts, lifecycle completion counts, and the top-level scientifically eligible and operationally censored counts. Its v6 note explicitly states the two migration-quality weighting units so the summary cannot present them as interchangeable.

These are descriptive cumulative summaries, not exposure-normalized rates. `simulatedDays`/`endDay` make different achieved durations explicit for downstream analysis; this repair does not redefine cumulative counts or the estimand used for extinction outcomes.

## Python and R consumption

The CSV artifacts are ordinary rectangular comma-separated tables. No AnthroSim-specific library is required. Typical downstream loading is therefore as simple as:

```python
import pandas as pd
runs = pd.read_csv("runs/resource-sweep/analysis/runs.csv")
points = pd.read_csv("runs/resource-sweep/analysis/points.csv")
```

or in base R:

```r
runs <- read.csv("runs/resource-sweep/analysis/runs.csv")
points <- read.csv("runs/resource-sweep/analysis/points.csv")
```

The JSON equivalents preserve null values and nested source-run ID lists for consumers that prefer structured records.

AnthroSim intentionally does **not** add statistical inference, plotting, notebook machinery, sensitivity-analysis algorithms or a general-purpose statistics dependency to the simulation core. More sophisticated inference belongs in downstream scientific workflows where assumptions and methods can be chosen explicitly.

## Versioned experiment definitions and M7.6 reproduction

M7.6 adds a small adapter for reviewable versioned sweep definitions. The canonical v0.1 exercise is stored at:

```text
experiments/v0.1-resource-variability.json
```

The definition records the synthetic-validation status, research-style question, base controls, exact ordered seed set, factorial dimensions and plain-language interpretation of the varied controls. `scripts/run-versioned-sweep.py` translates that file into the ordinary `anthrosim sweep` CLI and then verifies that the generated immutable `sweep-manifest.json` contains the same seeds, base settings and dimensions. The adapter does not simulate agents and does not aggregate results independently.

Build the release binary normally from a clean Git checkout; the checked-out source revision is embedded automatically. On a POSIX shell:

```text
cargo build --locked --workspace --release
python3 scripts/run-versioned-sweep.py \
  experiments/v0.1-resource-variability.json \
  --binary target/release/anthrosim \
  --run-dir runs/v0.1-resource-variability
```

On PowerShell:

```text
cargo build --locked --workspace --release
python scripts/run-versioned-sweep.py experiments/v0.1-resource-variability.json --binary target/release/anthrosim.exe --run-dir runs/v0.1-resource-variability
```

No manual revision environment variable is required for the ordinary path. A clean tracked checkout records the exact SHA. A tracked modification produces a `-dirty-<working-tree-digest>` source identity and build warning. If Git metadata is unavailable, the build records `gitCommit: null`. `scripts/run-versioned-sweep.py` deliberately refuses both missing identity and automatically detected dirty source state so a formal versioned result cannot silently claim an unreproducible source tree. Controlled build systems may still set a non-empty `ANTHROSIM_GIT_COMMIT` override deliberately. See [`source-provenance.md`](source-provenance.md).

A successful versioned run copies the exact input to `source-definition.json` and writes `reproduction-record.json`. That record contains the SHA-256 of the source definition, model/package version, required build source revision, immutable sweep ID and paths to the authoritative sweep manifest and derived analysis directory. The immutable sweep manifest remains authoritative for the fully expanded point/run identity.

To retry an interrupted reproduction, use the exact same checked-out model revision, definition file, binary identity and output directory plus `--retry`. M7.2/M7.3 reconciliation then preserves valid completed children and retries only unfinished/failed runs; a changed immutable definition is rejected.

The canonical experiment's observed results and interpretation are documented separately in `docs/research/resource-variability-v0.1.md`. That document is explicitly a synthetic-model exercise, not an empirical validation report.

## Relationship to single-run execution

Batch and sweep execution do not fork the simulation logic. `run`, `ensemble`, and every sweep point construct ordinary `ExperimentConfig` values, initialize the same `Simulation`, execute the same `run_recorded()` lifecycle for completed bundles and use the same M5 artifact writer.

This is important scientifically. Experiment orchestration must not create a subtly different model path merely because many configurations or seeds are being run.

Existing `anthrosim run` and `anthrosim resume` behaviour remains unchanged.

## Scientific use of statuses and aggregates

A `completed` lifecycle state proves that the run artifact bundle completed and reconciled; it does **not** by itself prove that the run belongs in a scientific aggregate. `failed`, `incomplete`, `planned` and `running` remain explicit non-results, while completed runs receive a separate scientific aggregation classification.

For the current duration-target sweep estimand, `DurationReached` and `PopulationExtinct` are eligible scientific outcomes. Extinction is retained because it is produced by the model rather than by an operational ceiling. `PersonRecordLimitReached` is operational censoring: the configured persistent-record ceiling may change for engineering reasons, so allowing that truncated trajectory into a point mean would let an arbitrary safety setting change the scientific aggregate.

Run-level derived tables therefore preserve every planned run, including censored completed runs, and expose achieved duration plus machine-readable classification. Point-level descriptive means and pooled migration summaries use only the scientifically eligible runs and publish both contributor counts and exact contributor IDs. Migration-quality summaries additionally publish both run-weighted and pooled per-move estimands together with the exact pooled move count and score totals. A researcher can define a different estimand downstream, but the default AnthroSim sweep output no longer silently equates artifact completion with scientific eligibility or one weighting unit with another.

This is scientific-analysis integrity hardening, not empirical validation. It changes derived-analysis schema and statistical labeling only; it does not change any simulation trajectory, stop behavior, demographic/resource/migration rule, RNG behavior, checkpoint continuation meaning or `MODEL_SEMANTICS_ID`.

## Example fresh ensemble

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 100 \
  --world-width 64 \
  --world-height 64 \
  --population 10000 \
  --seeds 1,2,3,4,5 \
  --run-dir runs/resource-baseline-seeds
```

All non-seed controls are shared across these child runs. Each exact per-run configuration is nevertheless expanded and serialized independently in the immutable experiment manifest, and each child manifest remains the authoritative record of what that run actually simulated.

## Scientific interpretation

Running many seeds or parameter points does not validate the model and does not turn synthetic assumptions into empirical parameters. An ensemble or sweep measures behaviour under declared executable models and configurations. Interpretation still has to respect the provenance and synthetic-validation boundaries documented for demography, resources and migration.

M7 provides reproducible experiment design, explicit failure accounting, clean descriptive outputs, long-run invariant checks, an engineering performance envelope and a versioned reproduction path. It does not establish that the parameter ranges themselves are archaeologically or anthropologically justified.
