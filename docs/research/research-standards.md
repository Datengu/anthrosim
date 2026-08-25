# AnthroSim research-model standards

**Status:** living research-governance document  
**Scientific status:** AnthroSim remains exploratory / unvalidated unless a question-specific study demonstrates otherwise.

AnthroSim formally adopts three complementary model-development and reporting frameworks:

1. **ODD 2020** — the standard Overview, Design concepts and Details protocol for describing agent-based and other simulation models.
2. **ODD+D** — the ODD extension for making assumptions about human decision-making explicit.
3. **TRACE** — the framework for planning, performing and documenting model evaluation: problem formulation, model description, data evaluation, conceptual-model evaluation, implementation verification, model-output verification, model analysis and independent output corroboration.

These frameworks are not certifications and do not by themselves make a model valid. They are used so that any claim of research readiness must be supported by inspectable evidence rather than by software maturity, deterministic replay or documentation volume alone.

## Canonical AnthroSim documents

- [`odd.md`](odd.md) — formal ODD 2020 model description / compliance view.
- [`odd-d.md`](odd-d.md) — ODD+D decision-making supplement.
- [`trace.md`](trace.md) — living TRACE evaluation dossier and research-readiness gates.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific-model semantics.
- [`../research-principles.md`](../research-principles.md) — repository-wide research principles.
- [`evidence-provenance.md`](evidence-provenance.md) — evidence and parameter/input provenance contract.
- [`../experiments-v0.1.md`](../experiments-v0.1.md) — experiment, ensemble, sweep and reproduction contract.
- [`../research-integrity.md`](../research-integrity.md) — cryptographic archive-integrity procedure.

The ODD/ODD+D documents do **not** replace the detailed scientific specification. Where a concise standards document and the detailed scientific-model specification appear inconsistent, the discrepancy is a documentation defect that must be resolved before relying on the affected model semantics.

## How the three frameworks divide responsibility

| Framework | Primary question | AnthroSim use |
|---|---|---|
| ODD 2020 | What is the model, what entities/processes exist, and how is it scheduled? | Stable model description suitable for review and reimplementation. |
| ODD+D | What is being assumed about human decisions, knowledge, objectives, adaptation and social context? | Prevent synthetic behavioural rules from being mistaken for empirically validated human psychology or social theory. |
| TRACE | Why should this model be trusted for a particular purpose? | Evidence trail for design rationale, data quality, verification, validation, sensitivity/uncertainty analysis and independent corroboration. |

## Required maintenance rule

A change is scientifically meaningful when it changes the model's causal or observational semantics, even if the software API remains compatible. Such a change must update, where relevant:

- the detailed scientific specification;
- the ODD description;
- the ODD+D supplement if human decision semantics change;
- the TRACE dossier if the change alters evidence, verification, validation, analysis or domain of applicability;
- experiment/model identity according to the repository's versioning/provenance rules.

Pure implementation changes that provably preserve model semantics do not require invented scientific rationale, but verification evidence must remain valid.

## Study-specific requirement

Repository-level ODD/ODD+D/TRACE documentation describes the **model framework and its evaluated baseline**. It does not replace a study protocol for a real archaeological or anthropological question.

Before a study is represented as inferential research, freeze a study-specific protocol containing at minimum:

- research question and domain of applicability;
- competing hypotheses/null models;
- permitted and prohibited interpretations;
- predeclared observables and comparison criteria;
- evidence used for model construction/parameterisation;
- evidence used for calibration, if any;
- evidence reserved for independent corroboration;
- parameter ranges and uncertainty representations;
- ensemble/seeds and stopping/exclusion rules;
- sensitivity and uncertainty-analysis plan;
- analysis method and decision criteria;
- predictions or discriminating observations that could falsify or separate hypotheses.

Exploratory work may precede such a protocol, but exploratory results must not be retrospectively described as confirmatory.

## Primary references

- Grimm, V. et al. (2020). *The ODD protocol for describing agent-based and other simulation models: A second update to improve clarity, replication, and structural realism.* Journal of Artificial Societies and Social Simulation 23(2):7. DOI: `10.18564/jasss.4259`.
- Müller, B. et al. (2013). *Describing human decisions in agent-based models – ODD + D, an extension of the ODD protocol.* Environmental Modelling & Software 48:37–48. DOI: `10.1016/j.envsoft.2013.06.003`.
- Grimm, V. et al. (2014). *Towards better modelling and decision support: Documenting model development, testing, and analysis using TRACE.* Ecological Modelling 280:129–139. DOI: `10.1016/j.ecolmodel.2014.01.018`.
