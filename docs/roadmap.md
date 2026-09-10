# AnthroSim roadmap

## Purpose

AnthroSim's roadmap is driven by research capability rather than feature count. The project should add new mechanisms or infrastructure when they are needed to ask a clearer question, distinguish competing explanations, expose uncertainty, or make an experiment more reproducible and inspectable.

**Current framework line:** software version `v0.3.6` / current model semantics v36. Immutable `v0.3.6` remains the v35 Scientific-Audit-v6 discovery/release baseline; immutable `v0.3.5` remains the v33 Audit-v5 discovery/release baseline; immutable `v0.3.4` remains the historical v25 Audit-v4 target. Living v36 repairs the Audit-v6 same-day M9/M4 scheduler seam without rewriting any immutable release or audit target.

The long-term direction remains the one described in `docs/vision.md`: build enough defensible lower-level rules that history-like structure can emerge without scripting historical outcomes.

This roadmap is intentionally case-study-neutral. Public core documentation should describe reusable scientific capabilities and validation boundaries without depending on any particular site, locality, dataset or project-specific research question. Specific case studies, when published, should live in explicitly scoped research artifacts rather than define the core engine roadmap.

## Completed baseline: M1–M7

The first seven milestones established the software and experiment-engine baseline:

- deterministic synthetic spatial environments;
- persistent people, households, demography and genealogy;
- renewable resources, condition and scarcity response;
- bounded, interpretable household migration;
- authoritative events, derived metrics and resumable checkpoints;
- a read-only explorer separated from authoritative simulation state;
- deterministic ensembles and parameter sweeps;
- immutable experiment provenance and explicit retry/failure semantics;
- long-run invariant, performance, memory and cross-platform determinism gates;
- machine-readable evidence provenance for evidence-grounded parameters and external inputs.

This baseline can answer questions of the form “what does this declared model do when assumption X changes?” It is not an empirically validated model of a real past population or landscape.

## Development rule after M7

Later work should normally follow this sequence:

1. **State a research question or methodological target.**
2. **Define the simplest relevant hypotheses or null model.**
3. **Define observable outputs before implementation.**
4. **Identify the minimum missing capability.**
5. **Record assumptions, evidence, units, transformations and uncertainty.**
6. **Run ensembles and sensitivity/uncertainty analysis.**
7. **Treat negative results as information.**
8. **Let results shape the next milestone.**

This keeps AnthroSim from becoming either a feature-accumulation project or a historical reconstruction engine whose desired outcome is embedded in its rules.

## M8 — Evidence-grounded spatial experiments

**Status:** complete. M8.0–M8.6 establish a generic evidence-grounded spatial experiment path; they do not establish archaeological validation.

M8 allows controlled, reproducible experiments on declared spatial evidence while preserving a strict separation between authoritative simulation semantics and external GIS/scientific tooling. Raw GIS data should normally be prepared with mature external tools and converted into a versioned AnthroSim landscape bundle; AnthroSim owns validation, deterministic model-facing transformation, provenance and causal observability.

Implemented M8 capability includes:

- a frozen spatial research/null-model contract;
- normalized landscape input with CRS/extent/resolution/nodata/layer/evidence identity;
- reproducible external preprocessing;
- deterministic landscape loading;
- explicit movement-cost, water-access and resource-opportunity transformations;
- residence-based spatial observability and read-only explorer support;
- the M8.6 paired terrain benchmark.

### Current M8.6 result

The current checked-in M8.6 machine reference is `examples/m8-first-evidence-grounded-benchmark/reference-result.json` under **model semantics v33**. All four terrain arms remain non-degenerate and the overall benchmark classification remains **`fragile_spatial_structure`**. The current v33 reference has no robust primary metric; terminal largest-cell share is fragile, while migration distance, cell-time occupancy and terminal Herfindahl are not distinctive under the predeclared criteria.

Earlier v26–v32 reviewed executions remain important historical sensitivity evidence and are preserved in [`research/m8-first-evidence-grounded-benchmark-result.md`](research/m8-first-evidence-grounded-benchmark-result.md). A changed regression reference after an upstream causal repair is not calibration; it records the behavior of the unchanged benchmark under the corrected complete model.

M8 still does not reconstruct a historical community, script known locations/routes, replace GIS tooling, collapse palaeoenvironmental uncertainty, or establish archaeological validity.

## M9 — Temporary mobility and aggregation experiments

**Status:** complete. M9.0–M9.7 establish generic temporary mobility and controlled aggregation capability.

M9 keeps persistent residence distinct from physical presence. A household may undertake deterministic outbound transit, visit a declared focal region, return, and restore at-residence presence without redefining M4 permanent migration. Transit has timing/resource semantics but deliberately has no authoritative per-day world cell.

Implemented M9 capability includes:

- frozen residence/presence semantics;
- identity-bearing focal-region binding;
- deterministic multi-day journey lifecycle;
- declared travel-cost/duration/reachability semantics;
- duration-aware resource accounting;
- separate temporary-presence observability;
- experiment/ensemble/sweep/checkpoint integration;
- the M9.7 continuous-residence versus intermittent-aggregation benchmark.

The M9.7 benchmark remains **`capability_distinguished`**. Its checked-in machine reference remains on model semantics v35 after the Audit-v5 M9 coupling repairs and reviewed reference rebaseline; earlier v31–v34 references remain historical evidence. Living v36 changes scheduler phase re-entry semantics rather than the frozen reference artifact. That benchmark is a capability/regression result, not evidence for a real social motive or archaeological interpretation. See [`research/m9-controlled-aggregation-benchmark-result.md`](research/m9-controlled-aggregation-benchmark-result.md).

M9 does not by itself add trade, ritual, feasting, religion, politics, warfare, livestock, settlement institutions, archaeological preservation/detection or empirical calibration.

## Release and audit history

Milestone identity, software release identity, model-semantics identity and Git source identity remain separate.

- **v0.2.0 / M8:** preserved evidence-grounded spatial capability baseline.
- **v0.3.0 / M9:** preserved temporary-mobility/aggregation capability baseline.
- **v0.3.1:** post-M9 scientific hardening and analysis/inference safeguards.
- **v0.3.2:** documentation-convergence maintenance release, immutable model semantics v19.
- **v0.3.3:** post-Audit-v2 convergence release, immutable model semantics v21; frozen Audit-v3 target.
- **v0.3.4:** post-Audit-v3 convergence release, immutable model semantics v25; frozen Audit-v4 target.
- **v0.3.5:** post-Audit-v4 repaired convergence release, immutable model semantics v33; frozen Audit-v5 target.
- **v0.3.6:** post-Audit-v5 repaired baseline, immutable model semantics v35; frozen Audit-v6 target, no new roadmap milestone and not a P1-clean convergence result.

Scientific Audit v3 challenged immutable v0.3.3/v21, demonstrated 17 findings and produced the repaired v25 line later frozen as v0.3.4.

Scientific Audit v4 then restarted Areas A–N from zero against immutable v0.3.4/v25. It demonstrated **13 P1 and 2 P2 findings**. Post-discovery remediation repaired and independently re-verified/dispositioned all 15 findings. Authoritative repairs advanced the living development line through **model semantics v26–v33** where continuation/scientific meaning changed. The repository-authoritative record is [`research/audit-v4/STATUS.md`](research/audit-v4/STATUS.md).

Audit v4 is complete and historical. Scientific Audit v5 then restarted all Areas A–N from zero against immutable `v0.3.5`/v33. It demonstrated **8 findings: 4 P1 and 4 P2**; all eight were repaired/dispositioned and closed, with independent post-merge adversarial re-verification for every P1. Causal repairs advanced living semantics through v34 to v35 and the repaired line was frozen as v0.3.6.

Scientific Audit v6 then independently restarted Areas A–N against immutable `v0.3.6`/v35 and demonstrated **14 findings: 7 P1 and 7 P2**. Discovery is complete and is therefore another **non-clean convergence pass**. Controlled remediation is in progress; authoritative current state is tracked in [`research/audit-v6/STATUS.md`](research/audit-v6/STATUS.md). Living semantics advance only when a repair changes authoritative scientific meaning.

## Convergence before the first empirical study

No fixed M10 feature list is declared, and no empirical/site-specific question should begin while Audit-v6 findings remain unresolved. The v0.3.6/v35 discovery result is non-clean, so all findings must be dispositioned, P1 repairs independently reverified, the fully repaired line frozen as a new immutable baseline, and another genuinely fresh audit generation completed. Only after a P1-clean convergence pass should the project move into its first empirical research question.

Once that convergence gate is satisfied, a defensible question-led sequence is:

1. define the inferential or methodological question;
2. identify the smallest relevant existing model/null model;
3. define observables and predeclared comparison criteria;
4. evaluate whether the converged current mechanisms are sufficient to run the comparison without inventing a missing causal process;
5. add only capabilities demonstrated to be necessary;
6. calibrate/parameterise only with declared evidence roles;
7. run stochastic precision, sensitivity, structural sensitivity, resolution/boundary and initialization checks as relevant;
8. assess identifiability/equifinality;
9. compare against appropriate empirical patterns and held-out corroboration where possible;
10. obtain relevant domain review before strong historical claims.

The general demographic-baseline study illustrates this rule. It found no defensible universal demographic default: realized growth changes strongly with household lifecycle and mate limitation. Future studies must declare and justify demographic schedule and household structure rather than treating one synthetic preset as a stationarity guarantee.

Candidate future directions remain valid only when justified by experimental need, including:

- evidence-grounded or alternative demographic/household initialization;
- settlement formation and persistence mechanisms;
- livestock or managed-herd behaviour;
- richer kinship/social-interaction mechanisms;
- collective labour and construction costs;
- exchange or cultural-transmission models;
- archaeological observation models that transform simulated past behaviour into material remains, preservation, detection and sampling;
- calibration and comparison against independent evidence;
- reproduction of published models on an independent implementation for validation.

## Scientific interpretation boundary

A progression from synthetic worlds to real spatial data increases empirical relevance but does not automatically increase explanatory validity.

At every stage, AnthroSim should distinguish:

1. source observations or reconstructed inputs;
2. transformations from evidence into model inputs;
3. authoritative simulated past state and behaviour;
4. derived metrics and classifications;
5. archaeological observation/preservation/detection processes where modelled;
6. downstream interpretation.

Strong archaeological or anthropological claims require question-specific validation, sensitivity and uncertainty analysis, comparison with relevant evidence and domain review. The software should make those steps possible and auditable rather than imply that simulation alone supplies the answer.
