# AnthroSim research documentation

AnthroSim's research documentation is organized around explicit model description, human-decision assumptions, model evaluation, evidence provenance and reproducible experiment records.

Start here:

- [`research-standards.md`](research-standards.md) — formal adoption and maintenance rules for **ODD 2020, ODD+D and TRACE**.
- [`odd.md`](odd.md) — formal ODD 2020 model description.
- [`odd-d.md`](odd-d.md) — ODD+D human decision-making supplement.
- [`trace.md`](trace.md) — living TRACE evaluation / research-readiness dossier.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific-model specification.
- [`../research-principles.md`](../research-principles.md) — general research-software/scientific principles.
- [`evidence-provenance.md`](evidence-provenance.md) — evidence provenance and transformation contract.

## TRACE audit records

- [`trace-audit-2026-08-25.md`](trace-audit-2026-08-25.md) — first repository-wide TRACE-structured deep scientific audit after formal ODD/ODD+D/TRACE adoption; records the eight-element assessment, deduplicated findings and resulting research gates/issues.
- [`trace-audit-2026-08-25-pass-2.md`](trace-audit-2026-08-25-pass-2.md) — second independent adversarial pass focused on finite-domain effects, seed/environment separation, spatial initialization, counterfactual RNG semantics, identifiability, observability and long-run/path-dependence safeguards.
- [`trace-audit-2026-08-25-pass-3.md`](trace-audit-2026-08-25-pass-3.md) — third independent pass focused on population/resource conservation, intervention/symmetry checks, event replay and scientific-summary semantics; records the undefined-empty-set mean P1 and the mechanisms that held up under this lens.
- [`trace-audit-2026-08-25-pass-4.md`](trace-audit-2026-08-25-pass-4.md) — fourth pass focused on lifecycle extremes and scientific-output fidelity; records the M8 initial-resource-state P1 plus planned/realized travel, M4 condition-loss and exposure-normalization findings.
- [`trace-audit-2026-08-25-pass-5.md`](trace-audit-2026-08-25-pass-5.md) — fifth pass focused on causal opportunity structure and demographic interval semantics; records the mortality/fertility competing-event P1, demographic-opportunity observability and the M2 extension to raster-resolution sensitivity.
- [`trace-audit-2026-08-25-pass-6.md`](trace-audit-2026-08-25-pass-6.md) — sixth pass focused on limiting cases, null interventions, parameter directionality and metamorphic semantics; the first independent pass in the current sequence to find no new P1.

Module- and milestone-specific documents in this directory provide the detailed evidence, contracts, assumptions and benchmark records referenced by those standards documents.

The presence of ODD, ODD+D and TRACE documentation does **not** certify the model as empirically valid. The current baseline remains exploratory/unvalidated until the relevant TRACE gates are supported by question-specific verification, sensitivity/uncertainty analysis, empirical output testing and independent corroboration.