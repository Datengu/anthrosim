# AnthroSim research documentation

AnthroSim's research documentation is organized around explicit model description, human-decision assumptions, model evaluation, evidence provenance and reproducible experiment records.

Start here:

- [`research-standards.md`](research-standards.md) — formal adoption and maintenance rules for **ODD 2020, ODD+D and TRACE**.
- [`odd.md`](odd.md) — formal ODD 2020 model description.
- [`odd-d.md`](odd-d.md) — ODD+D human decision-making supplement.
- [`trace.md`](trace.md) — living TRACE evaluation / research-readiness dossier.
- [`scientific-audit-protocol.md`](scientific-audit-protocol.md) — reusable adversarial scientific-audit protocol and convergence criteria.
- [`audit-v3/README.md`](audit-v3/README.md) — completed third independent scientific-audit charter for the immutable `v0.3.3` / model-semantics-v21 discovery baseline.
- [`audit-v3/STATUS.md`](audit-v3/STATUS.md) — immutable discovery-phase Audit-v3 ledger and A–N evidence record; its finding statuses describe the frozen discovery state, not the later remediation state.
- [`v0.3.4-documentation-readiness-audit.md`](v0.3.4-documentation-readiness-audit.md) — pre-release living-document/release-identity convergence record for the fully remediated v25 line.
- [`audit-v2/STATUS.md`](audit-v2/STATUS.md) — closed second independent audit ledger; historical evidence, not completion evidence for audit v3.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific-model specification.
- [`../research-principles.md`](../research-principles.md) — general research-software/scientific principles.
- [`evidence-provenance.md`](evidence-provenance.md) — evidence provenance and transformation contract.
- [`spatial-boundary-dependence-v1.md`](spatial-boundary-dependence-v1.md) — normative finite-domain contract for distinguishing arbitrary GIS crops from declared barriers, separating simulation and analysis domains, exposing M4/M9 edge dependence, and demonstrating study-specific extent convergence.
- [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md) — authoritative annual M2 demographic-time semantics introduced by the post-M9 scientific-hardening repair programme; defines interval age, competing mortality/fertility, executable birth spacing, same-day M4 parentage locality, newborn condition and founder-history semantics.
- [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) — explicit synthetic-versus-declared founder-state contract, including signed pre-run reproductive timing and living direct-parent links.
- [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md) — versioned derived M2 validation/diagnostic surface for mortality exposures, fertility opportunity denominators, requested-versus-executable spacing, interbirth intervals and explicitly censored completed fertility.
- [`trace-m2-demographic-time-repair-2026-08-25.md`](trace-m2-demographic-time-repair-2026-08-25.md) — TRACE change record for the first M2 transition-semantics repair, including verification evidence and deliberate reference regeneration after the semantics change.
- [`trace-m2-founder-initialization-repair-2026-08-25.md`](trace-m2-founder-initialization-repair-2026-08-25.md) — TRACE record for explicit provenance-bearing founder reproductive/genealogical state.
- [`trace-m2-demography-observability-2026-08-26.md`](trace-m2-demography-observability-2026-08-26.md) — TRACE record for run-facing M2 opportunity diagnostics and the #179/#191/#193/#227/#228 acceptance closure surface.
- [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md) — authoritative v8 M3 annual resource-accounting contract: exact half-open resource intervals, annual-quantity conservation, mean-preserving seasonal redistribution, historical shared-boundary M3/M4 demand alignment and zero-demand condition neutrality; retained under v9 for those rules.
- [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md) — authoritative v9 #204 response-time contract: elapsed-time condition/scarcity response, independent M3 and M4 clocks, M4 interval demand and merged scheduler semantics.
- [`m4-travel-condition-loss-observability-v1.md`](m4-travel-condition-loss-observability-v1.md) — #225 observability contract distinguishing nominal per-person M4 travel-condition cost from exact realized bounded loss per completed household move, with event-to-aggregate reconciliation.
- [`resources-v0.1.md`](resources-v0.1.md) — M3 synthetic resource-model assumptions, units and empirical-evidence boundary; resource accounting remains grounded in the v8 contract while v9 response timing is defined separately.
- [`trace-m3-resource-time-accounting-2026-08-26.md`](trace-m3-resource-time-accounting-2026-08-26.md) — TRACE change record for the #180/#189/#199 resource-time repair and the condition/timing boundaries it deliberately left open at v8.
- [`trace-m3-response-time-repair-2026-08-26.md`](trace-m3-response-time-repair-2026-08-26.md) — TRACE change record for the v9 #204 timing repair, model-semantics bump and explicit exclusions #200/#208/#201.

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

Those earlier TRACE passes and scientific audit v2 are preserved as historical evidence. Audit v2 subsequently completed the reusable protocol's full Areas A–N, repaired and reverified all 15 AV2 findings, and closed with no open P0–P3 scientific finding. The repaired v21 state was then frozen as immutable `v0.3.3` at `358ae93b57a9b8f7053575dc6651aa959de2b4f9`.

Scientific Audit v3 then restarted full A–N coverage from zero against that frozen release. It demonstrated 17 findings (1 P0, 6 P1 and 10 P2). The post-discovery remediation programme repaired all 17 through protected production changes and independently re-ran the original adversaries against merged `main` before closing each finding. The resulting living line is `anthrosim-model-semantics-v25`, and `v0.3.4` is the intended named freeze for that state. This is strong framework-verification convergence evidence, not proof of correctness or empirical archaeological validity.

The post-M9 repair programme proceeded by explicit causal contracts rather than issue-by-issue tuning. M2 repairs are anchored by [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md); M3 annual resource accounting is anchored by [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md), with the independent response/decision timing repair anchored by [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md). Changes to authoritative behavior invalidate exact synthetic-output references from older model-semantics identities; affected M7/M8/M9 references are deliberately regenerated, reviewed and preserved only after the changed outputs are mechanistically explained. Observability-only additions remain downstream of authoritative state and do not by themselves change `MODEL_SEMANTICS_ID`.

Module- and milestone-specific documents in this directory provide the detailed evidence, contracts, assumptions and benchmark records referenced by those standards documents.

The presence of ODD, ODD+D and TRACE documentation does **not** certify the model as empirically valid. The current baseline remains exploratory/unvalidated until the relevant TRACE gates are supported by question-specific verification, sensitivity/uncertainty analysis, empirical output testing and independent corroboration.
