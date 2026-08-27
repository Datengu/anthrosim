# Evidence-closure spatial composition v1

Status: implementation contract for issue #181.

This document extends the evidence-closure policy without changing simulation trajectories. It
closes a gap between the core `ExperimentConfig` assessment and evidence claims carried by declared
founder state, M9 inputs and spatial execution.

## Meaning of `closed`

`closed` does **not** mean every model assumption is empirical. It means every assumption that
claims `empirical_direct`, `empirical_derived` or `evidence_informed` provenance is backed by the
required stable evidence, while any remaining assumptions are explicitly synthetic/null-model.

This distinction is necessary for the M8.6 terrain benchmark: its terrain input is genuinely
source- and content-bound, while the terrain-to-movement mapping is deliberately an uncalibrated
sensitivity assumption. The benchmark can therefore close the evidence claim it actually makes
without falsely promoting the movement-cost transform to an empirical law.

## Core closure v2

The core assessment now covers all provenance-bearing inputs represented by the current experiment
schema:

- demography;
- resources;
- migration;
- declared founder state;
- the M9 temporary-travel model.

Declared founder state uses `founderPopulation.contentDigest64` as a whole-object evidence address.
The digest binds household membership, people, reproductive state, genealogy completeness,
locations and the declared provenance without depending on array positions.

The M9 travel model requires support for its two scientifically substantive scalar assumptions:
travel capacity per day and the maximum traversable movement cost. An explicitly synthetic travel
model remains a null-model assumption and makes no empirical claim.

M9 v1's trigger schedule is deliberately outside that empirical claim set. The frozen M9 v1
contract defines temporary mobility as synthetic/null-model semantics and the trigger schedule as
an exogenous experimental condition rather than a historical timing claim. If a later M9 schema
allows empirical/evidence-informed schedule provenance, that schedule must gain a stable
content-bound evidence identity and be added to the closure policy rather than inheriting `closed`
status from the v1 null-model contract.

A landscape-mask M9 focal region is itself an external-input evidence claim. Consequently a run
with otherwise synthetic core mechanisms can no longer be labelled `not_applicable_synthetic` when
its M9 region claims an evidence-bound landscape source.

## Referenced external inputs only

Closure assesses external inputs that are actually referenced by the run. Merely storing an
unrelated catalogue entry does not make that external input a causal assumption of the run and does
not block closure. The authoritative boundary is therefore the causal input set of the executed run,
not the full inventory of everything stored in its catalogue or landscape bundle.

Every referenced external input must have:

- a catalogue entry;
- an evidence record;
- a non-empty content digest;
- a reproducible source identity (`persistentId` or `datasetVersion`);
- an explicit transformation when the record claims `empirical_derived` provenance.

## Spatial composition

`assess_spatial_evidence_closure` composes the core assessment with spatial evidence claims that can
causally affect the run:

1. source layers consumed by configured spatial transforms;
2. the landscape-mask layer consumed by M9, when present;
3. transforms that explicitly declare an `evidenceId`.

Unused auxiliary layers are not treated as run assumptions merely because they happen to be present
in the same bundle.

A used layer with no `evidenceInputId` remains synthetic. A transform with no `evidenceId` remains
synthetic. This is deliberate: absence of an empirical claim must never be converted into one by
the readiness system.

## Preserved provenance

Core run manifests preserve the v2 core assessment. Spatial run manifests v2 additionally preserve
the composed spatial assessment. Validation recomputes both from the exact experiment, landscape
and spatial-mechanism configuration, so downstream tooling cannot relabel a spatial run as
`closed` without detection.

This is provenance integrity only. Neither assessment participates in checkpoint continuation state
or changes population, resource, migration, M9 or RNG semantics.

## Positive fixture

The committed `examples/m8-first-evidence-grounded-benchmark` terrain fixture is the positive
spatial closure fixture. Its Mapzen/Skadi-derived terrain layer has a pinned SHA-256 external-input
identity, reproducible source identity and explicit derivation. Its movement-cost transform remains
explicitly uncalibrated/synthetic. The composed assessment therefore closes the terrain evidence
claim while preserving the benchmark's documented null-model interpretation.

Passing this gate still does not establish historical correctness, archaeological validity or that
the tested synthetic transformation is an empirically calibrated model of human movement.
