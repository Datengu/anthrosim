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
- [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md) — authoritative annual M2 demographic-time semantics introduced by the post-M9 scientific-hardening repair programme; defines interval age, competing mortality/fertility, executable birth spacing, same-day M4 parentage locality, newborn condition, and the still-open founder/observability work.
- [`trace-m2-demographic-time-repair-2026-08-25.md`](trace-m2-demographic-time-repair-2026-08-25.md) — TRACE change record for the first M2 repair slice, including verification evidence, invalidated synthetic references and remaining gates.

## TRACE audit records

- [`trace-audit-2026-08-25.md`](trace-audit-2026-08-25.md) — first repository-wide TRACE-structured deep scientific audit after formal ODD/ODD+D/TRACE adoption; records the eight-element assessment, deduplicated findings and resulting research gates/issues.
- [`trace-audit-2026-08-25-pass-2.md`](trace-audit-2026-08-25-pass-2.md) — second independent adversarial pass focused on finite-domain effects, seed/environment separation, spatial initialization, counterfactual RNG semantics, identifiability, observability and long-run/path-dependence safeguards.
- [`trace-audit-2026-08-25-pass-3.md`](trace-audit-2026-08-25-pass-3.md) — third independent pass focused on population/resource conservation, intervention/symmetry checks, event replay and scientific-summary semantics; records the undefined-empty-set mean P1 and the mechanisms that held up under this lens.
- [`trace-audit-2026-08-25-pass-4.md`](trace-audit-2026-08-25-pass-4.md) — fourth pass focused on lifecycle extremes and scientific-output fidelity; records the M8 initial-resource-state P1 plus planned/realized travel, M4 condition-loss and exposure-normalization findings.
- [`trace-audit-2026-08-25-pass-5.md`](trace-audit-2026-08-25-pass-5.md) — fifth pass focused on causal opportunity structure and demographic interval semantics; records the mortality/fertility competing-event P1, demographic-opportunity observability and the M2 extension to raster-resolution sensitivity.
- [`trace-audit-2026-08-25-pass-6.md`](trace-audit-2026-08-25-pass-6.md) — sixth pass focused on limiting cases, null interventions, parameter directionality and metamorphic semantics; the first independent pass in the current sequence to find no new P1.
- [`trace-audit-2026-08-25-pass-7.md`](trace-audit-2026-08-25-pass-7.md) — confirmatory-study integrity, Monte Carlo precision and executable downstream-analysis provenance; no new P1 discovered.
- [`trace-audit-2026-08-25-pass-8.md`](trace-audit-2026-08-25-pass-8.md) — writer→reader causal-graph audit across condition, residence, resources, temporary presence, genealogy and world fields; no new P1 discovered.
- [`trace-audit-2026-08-25-pass-9.md`](trace-audit-2026-08-25-pass-9.md) — symmetry, relabelling and arbitrary-bookkeeping audit; no new P1 discovered and current audit-first discovery phase judged converged.

The current discovery sequence has now produced multiple genuinely different clean P1-discovery passes. This is a stopping signal for **finding new foundational defects on the current code**, not a claim that verification is complete. The known blocking P1 backlog must now be repaired by causal cluster, after which the relevant adversarial audit families must be repeated on the corrected implementation.

The M2 repair programme begins from [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md). The first implementation slice changes authoritative model semantics and therefore invalidates exact synthetic-output references generated under the previous semantics; affected M7/M8/M9 references must be deliberately regenerated, reviewed, and preserved under the new model-semantics identity rather than tuned back to the old outputs. This is verification/reference maintenance, not empirical calibration.

Module- and milestone-specific documents in this directory provide the detailed evidence, contracts, assumptions and benchmark records referenced by those standards documents.

The presence of ODD, ODD+D and TRACE documentation does **not** certify the model as empirically valid. The current baseline remains exploratory/unvalidated until the relevant TRACE gates are supported by question-specific verification, sensitivity/uncertainty analysis, empirical output testing and independent corroboration.