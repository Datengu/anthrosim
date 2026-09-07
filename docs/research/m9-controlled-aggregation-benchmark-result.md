# M9.7 controlled aggregation benchmark result

**Benchmark:** `m9_7_controlled_continuous_vs_intermittent_v1`  
**Predeclared contract commit:** `pre-sanitisation-ref-omitted-after-2026-09-02-privacy-rewrite`  
**Scientific status:** synthetic capability/verification benchmark, not archaeological validation  
**Current classification:** `capability_distinguished`

The checked-in machine-readable reference is `examples/m9-controlled-aggregation-benchmark/reference-result.json`. Earlier exact references and detailed per-generation review narrative remain preserved in Git history and the corresponding audit/evidence records.

**Current machine-readable reference: `anthrosim-model-semantics-v31`.**

## Current regression reference — model semantics v31

Audit-v4 AV4-007 / #500 removes canonical `HouseholdId` from M9 equal-cost destination stochastic coupling. Authoritative M9 tied destinations now use household scientific coupling derived from living-member stochastic-coupling identity. The frozen M9.7 design was rerun/reviewed and the checked-in machine reference advanced to v31.

Current v31 reference provenance:

- workflow run: `33829179759`;
- artifact: `9921177425`;
- artifact SHA-256: `d8efe2d815dee1757beedd7f3b389f0f068e233dbc75d60793be7c06aebeb08a`;
- reviewed branch head: `394f5514a02686e40d8dc0ef4b298b4be38fac49`;
- aggregate canonical SHA-256: `9e51677a44ef87115414a5424b22579880e7e392ce6e45f8df9a271ace37b29a`;
- model semantics: `anthrosim-model-semantics-v31`.

The predeclared capability conclusion remains **`capability_distinguished`**:

- all **8/8** paired seeds pass the declared paired criteria;
- median total focal-person-day difference remains **31 permille**;
- maximum paired total focal-person-day difference remains **36 permille**;
- median intermittent peak-visitor share remains **432 permille**;
- minimum intermittent peak-visitor share remains **396 permille**;
- continuous controls have no temporary journeys/visitor presence;
- intermittent treatments produce the declared temporary-presence signal;
- permanent M4 migration and condition-mediated mortality remain absent in the preserved benchmark design;
- deterministic replay/checkpoint-resume equivalence remains a separate workflow-gated requirement.

The current reference is v31 rather than current framework v33 because a benchmark reference records the latest **causally applicable reviewed execution**, not merely the newest global model-semantics label. Audit-v4 v32 scarce-resource remainder coupling and v33 M4 spatial-candidate coupling do not justify relabelling an unchanged M9.7 reference when those repaired pathways are absent/inapplicable under this benchmark's frozen configuration.

## Historical Audit-v4 applicability/rebaseline record

The benchmark was rerun or explicitly classified for relevant upstream repairs during Audit v4:

- **v26 / AV4-001:** fertility coupling repair changed complete authoritative population/genealogy state while the paired temporary-mobility capability metrics remained within the declared passing region.
- **v27 / AV4-002:** background-mortality coupling repair changed population-dependent occupancy/journey outcomes; the benchmark remained `capability_distinguished` and its checked reference was reviewed/rebaselined.
- **v28 / AV4-003:** permanent M4 migration RNG scheduling changed, but every M9.7 arm records zero permanent migrations, so the repair was classified as causally inapplicable and did not require a machine-reference relabel.
- **v29 / AV4-005:** parentage coupling repair changed authoritative genealogy/state even with permanent migration disabled, so the machine reference was rebaselined while paired capability metrics remained effectively unchanged.
- **v30 / AV4-006:** condition-mediated mortality coupling was reverified; the preserved M9.7 reference matched and no rebaseline was required because the benchmark records no condition-mediated deaths.
- **v31 / AV4-007:** M9 equal-cost destination coupling is directly relevant to the temporary-mobility mechanism; the current checked reference records the reviewed v31 execution.
- **v32 / AV4-008 and v33 / AV4-009:** later resource-remainder/M4-candidate repairs are not reasons to relabel this benchmark absent causal applicability; their global current-state verification is recorded elsewhere.

Exact historical run IDs, hashes, per-seed values and comparisons remain available in Git history and Audit-v4 issue/PR evidence. Older versions must not be cited as the current checked-in machine reference.

## What the current result establishes

Under one controlled synthetic design, AnthroSim can represent two regimes with very similar aggregate focal-region use but materially different temporal occupancy structure, preserve that distinction through authoritative state/events and checkpoint/resume, and expose it reproducibly through downstream observability and ensemble machinery.

The checked-in v31 reference establishes only that this frozen **software/model capability benchmark** remains distinguished after the relevant repaired M9 tie-coupling semantics. It does not establish invariance to all demographic, household, resource, condition or mobility alternatives.

## What this result does not establish

The benchmark is not evidence that intermittent aggregation, continuous residence or any social motive explains a real archaeological site. The focal region, schedule, travel model and population are synthetic validation inputs. Archaeological interpretation would require question-specific evidence, uncertainty propagation, structural sensitivity, calibration/corroboration separation, identifiability/equifinality analysis and domain review.

Reference maintenance after a declared causal/model-semantics change is reproducibility work, not empirical calibration. A later semantics generation should replace this reference only after causal applicability is established and the unchanged benchmark execution/result is independently reviewed.
