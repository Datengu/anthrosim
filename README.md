# AnthroSim

AnthroSim is an experimental, reproducible agent-based simulation framework for exploring human demography, resource-constrained movement, kinship, evidence-grounded spatial experiments, and temporary mobility/aggregation under explicit model assumptions.

**This source tree carries AnthroSim software version `0.3.5` at current model semantics v33.** The v0.3.5 release line names and preserves the fully repaired and independently re-verified Audit-v4 state. The immutable `v0.3.4` release remains model semantics v25 and the frozen Audit-v4 discovery target; immutable `v0.3.3` remains model semantics v21.

**Scientific Audit v4 is complete.** It independently audited immutable `v0.3.4`/v25 across Areas A–N, demonstrated 15 findings (13 P1 and 2 P2), and all 15 were repaired and independently re-verified/dispositioned on the living line. The final repaired Audit-v4 line is `anthrosim-model-semantics-v33`; there are no open Audit-v4 findings. See [`docs/research/audit-v4/STATUS.md`](docs/research/audit-v4/STATUS.md).

This is strong framework-verification evidence, not empirical validation or proof of scientific correctness. The preserved M8.6 and M9.7 references are capability/regression baselines under declared assumptions, not archaeological validation. AnthroSim is not a civilisation game and it is not a validated model of human prehistory.

## Core rule

> Model causes, not historical outcomes.

If a pattern appears in AnthroSim, the goal is for it to be explainable from simulated conditions and agent behaviour rather than from a scripted "civilisation" rule.

## Current capabilities

The current v0.3.5/model-semantics-v33 line retains the completed M1–M9 capability set and the scientific/reproducibility hardening added through subsequent audits:

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
- fail-closed identifiability/equifinality and provenance/integrity analysis;
- label/order-neutral scientific stochastic coupling introduced by Audit-v4 repairs for fertility, mortality, parentage, migration, resource remainder ties and M9 equal-cost destination choices;
- M4 spatial-candidate uncertainty/choice coupling that is invariant to arbitrary canonical candidate ordering on the current v33 line.

Culture, language, trade, states, religion, warfare, and AI-controlled agents remain deferred until a research question or validation target justifies adding them. M9 temporary mobility is deliberately a generic null mechanism rather than a cultural, ritual, political or economic motive model.

## Current milestone and audit status

- **M1 — Deterministic synthetic world:** complete.
- **M2 — Persistent people, households, demography and genealogy:** complete.
- **M3 — Renewable resources, household sharing, condition and scarcity survival:** complete; parameters remain an explicit synthetic validation baseline until empirically grounded.
- **M4 — Interpretable local permanent migration:** complete; migration utility weights, information radius and response thresholds remain explicit synthetic validation assumptions.
- **M5 — Events, metrics, checkpoints and causal inspection:** complete.
- **M6 — Local simulation explorer:** complete as a read-only artifact consumer.
- **M7 — Deterministic experiment orchestration, ensembles, retries, sweeps, soak/performance validation and the v0.1 reference experiment:** complete.
- **M8 — Evidence-grounded spatial experiments / v0.2.0:** complete and released.
- **M9 — Temporary mobility and controlled aggregation / v0.3.0:** complete, audited and released.
- **v0.3.1:** released post-M9 scientific/research-readiness hardening.
- **v0.3.2:** released documentation-convergence maintenance baseline at immutable model semantics v19.
- **v0.3.3:** immutable post-Audit-v2 baseline at model semantics v21 and frozen Audit-v3 target.
- **v0.3.4:** immutable post-Audit-v3 convergence baseline at model semantics v25 and frozen Audit-v4 target.
- **v0.3.5:** current post-Audit-v4 repaired convergence baseline at model semantics v33; it packages the fully remediated/re-verified v4 line and subsequent documentation-consistency reconciliation without adding a new roadmap milestone.
- **Scientific Audit v4:** discovery and remediation complete; 15/15 findings repaired and re-verified, with living semantics advanced through v26–v33 where authoritative scientific meaning changed.

No fixed M10 feature list is declared. The next scientific work should remain question-led and must separately establish evidence, calibration/validation, uncertainty, sensitivity, applicability and corroboration appropriate to the intended inference.

M8 keeps evidence separate from model-facing transformations and results. Its terrain reference remains **fragile spatial structure** under the declared terrain-only null model; the current checked-in machine reference is on model semantics v33. See [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md).

M9 does **not** reinterpret M4 migration. Permanent migration changes residence; temporary mobility changes physical presence while preserving residence. Transit deliberately has no authoritative per-day world cell. Resource demand is duration-weighted across journey states, and M9 observability is separate from M8 residence-based spatial observability. The M9.7 benchmark remains `capability_distinguished`; that is capability validation, not archaeological validation. See [`docs/research/temporary-mobility-v1.md`](docs/research/temporary-mobility-v1.md) and [`docs/research/m9-controlled-aggregation-benchmark-result.md`](docs/research/m9-controlled-aggregation-benchmark-result.md).

No historical destination, route, settlement, group or migration outcome is scripted into these loops.

## Running locally

AnthroSim uses the Rust toolchain pinned in `rust-toolchain.toml`. The CLI package contains multiple binaries, so local `cargo run` commands should explicitly select the main `anthrosim` binary.

Builds made inside a Git checkout automatically capture source provenance. A clean tracked tree records the exact commit SHA in `gitCommit`; a tracked dirty tree records a dirty source identity; outside Git, AnthroSim records `gitCommit: null`. See [`docs/source-provenance.md`](docs/source-provenance.md).

A small headless run:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --world-width 64 --world-height 64 --seed 1 --output runs/first-run.json
```

A completed inspectable run bundle:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m6-example
python scripts/serve-explorer.py runs/m6-example
```

A deliberately paused run can be explored and then resumed:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- run --years 25 --population 10000 --seed 1 --run-dir runs/m5-resume --checkpoint-year 10
python scripts/serve-explorer.py runs/m5-resume
cargo run --release -p anthrosim-cli --bin anthrosim -- resume --checkpoint runs/m5-resume/checkpoint.json --run-dir runs/m5-resume
```

`initial-population.json` is the day-zero founder state. A different-directory resume may also retain `resume-start-population.json` as boundary provenance; it must not be treated as the founder population.

The CLI exposes synthetic experiment controls such as `--resource-productivity-scale-permille`, `--resource-seasonality-scale-permille`, `--annual-food-need`, `--migration-radius` and `--disable-migration`. These are model-validation controls, not empirical caloric, palaeoecological or mobility measurements.

## Testing M8 landscape mode

```text
cargo run --release -p anthrosim-cli --bin anthrosim-landscape -- run \
  --landscape examples/landscape-loading/landscape.json \
  --mechanisms examples/landscape-loading/spatial-mechanisms.json \
  --years 25 \
  --population 1000 \
  --seed 1 \
  --run-dir runs/m8-landscape-example
```

The source landscape and transformed authoritative `world.json` remain separate by design. For the full M8.6 definition/result, see [`docs/research/m8-first-evidence-grounded-benchmark.md`](docs/research/m8-first-evidence-grounded-benchmark.md), [`docs/research/m8-first-evidence-grounded-benchmark-result.md`](docs/research/m8-first-evidence-grounded-benchmark-result.md), and `examples/m8-first-evidence-grounded-benchmark/`.

## Deterministic ensembles and sweeps

Launch an explicit seed set:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- ensemble \
  --years 25 \
  --population 10000 \
  --seeds 1,2,3,5,8 \
  --run-dir runs/example-ensemble
```

Or a Cartesian parameter sweep:

```text
cargo run --release -p anthrosim-cli --bin anthrosim -- sweep \
  --years 100 \
  --population 10000 \
  --seeds 1,2,3,4 \
  --sweep-resource-productivity-scale-permille 700,1000 \
  --sweep-resource-seasonality-scale-permille 0,1000 \
  --run-dir runs/resource-sweep
```

Ensemble/sweep definitions and planned run identities are immutable; retry requires the same definition, provenance-valid completed runs are retained, and incomplete/failed children remain explicit. Derived `analysis/` CSV/JSON files are downstream artifacts rather than authoritative state. See [`docs/experiments-v0.1.md`](docs/experiments-v0.1.md).

## Reproducing the v0.1 reference experiment

```text
cargo build --locked --workspace --release
python3 scripts/run-versioned-sweep.py \
  experiments/v0.1-resource-variability.json \
  --binary target/release/anthrosim \
  --run-dir runs/v0.1-resource-variability
```

The launcher records the exact definition digest, model identity and source identity and refuses uncontrolled dirty/missing source provenance. See [`docs/research/resource-variability-v0.1.md`](docs/research/resource-variability-v0.1.md).

## Scientific status

AnthroSim is a **research-oriented simulation framework, not a validated anthropological or archaeological model**. Its synthetic demographic, resource, permanent-migration and temporary-mobility presets remain explicit model assumptions unless a particular experiment supplies and justifies stronger evidence grounding.

Four independent/adversarial comprehensive audit generations have now been completed. Audit v4 is the latest: it challenged immutable v0.3.4/v25, demonstrated 13 P1 and 2 P2 findings, and all 15 were subsequently repaired and re-verified on the living v33 line. This strengthens implementation/convergence evidence, but repeated audits are evidence about verification maturity rather than proof of correctness.

Strong archaeological or anthropological claims still require question-specific problem formulation, evidence-role separation, calibration/validation where appropriate, uncertainty and sensitivity analysis, identifiability/equifinality assessment, comparison against independent observations, discriminating predictions and relevant domain review. See [`docs/research/trace.md`](docs/research/trace.md), [`docs/research/audit-v4/STATUS.md`](docs/research/audit-v4/STATUS.md), and [`docs/roadmap.md`](docs/roadmap.md).

## Contributing and security

Contribution expectations are documented in [`CONTRIBUTING.md`](CONTRIBUTING.md). Please report security-sensitive issues according to [`SECURITY.md`](SECURITY.md) rather than publishing exploit details in a normal issue.

## License

AnthroSim is licensed under the [Apache License 2.0](LICENSE).
