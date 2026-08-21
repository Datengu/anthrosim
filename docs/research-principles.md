# Research principles

AnthroSim may become a research instrument. The repository should therefore adopt research-software discipline before any claim of scientific validity is made.

## 1. Separate implementation from scientific meaning

An optimisation may change *how* a model runs without changing *what* it means. A model revision changes scientific meaning. The two should be distinguished in code review, versioning, and experiment provenance.

## 2. Reproducibility is default behaviour

Every run must identify at minimum:

- model/software version;
- experiment schema version;
- complete configuration;
- master seed and deterministic stream scheme;
- initial conditions;
- stop condition;
- output schema versions.

A supported build, configuration, and seed should reproduce the same deterministic result within a declared determinism boundary.

## 3. Assumptions are data

No important constant should exist only because it "felt right" during implementation. Placeholder assumptions are allowed in early versions, but they must be named, configurable where useful, documented, and marked as placeholders.

## 4. Validation is question-specific

There is no single "is the simulation realistic?" test. Validation should be tied to intended use and observable patterns. Examples include demographic rates, settlement distributions, mobility ranges, population persistence, and spatial clustering.

## 5. Sensitivity analysis precedes strong claims

If a conclusion vanishes under small plausible changes in arbitrary parameters, that uncertainty is part of the result. Batch experiments and parameter sweeps are first-class requirements.

## 6. Ground truth and interpretation remain separate

The authoritative simulation records what occurred inside the model. Metrics and classifications are derived from that state. Narrative or AI-assisted explanations, if introduced later, must never replace the underlying evidence and should expose provenance.

## 7. Negative and boring results are valid

The engine must not bias toward dramatic outcomes merely to make runs entertaining. Population stagnation, repeated extinction, failed migration, or absence of higher-order institutions may be legitimate model outcomes.

## 8. Preserve falsifiability

Where practical, each behavioural subsystem should specify directional expectations and cases that would indicate a defect or implausible assumption.

## 9. Prefer ensembles over anecdotes

A single artificial history can be interesting but is weak evidence. Research conclusions should normally emerge from ensembles of runs, controlled comparisons, uncertainty estimates, and documented analysis.

## 10. External review is a goal

Before AnthroSim is represented as a credible anthropological research model, relevant modules should be reviewed by domain specialists and compared with established literature and models.
