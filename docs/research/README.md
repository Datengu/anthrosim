# AnthroSim research documentation

AnthroSim's research documentation is organized around explicit model description, human-decision assumptions, model evaluation, evidence provenance and reproducible experiment records.

**Living framework state:** software version `v0.3.6` / current model semantics v41. Immutable `v0.3.6` is now the Scientific Audit-v6 discovery target at model semantics v35; immutable `v0.3.5` remains the historical Audit-v5 target at model semantics v33; immutable `v0.3.4` remains the historical Audit-v4 target at model semantics v25. Current documentation must not rewrite any historical audit identity.

## Start here

- [`research-standards.md`](research-standards.md) — formal adoption and maintenance rules for **ODD 2020, ODD+D and TRACE**.
- [`odd.md`](odd.md) — formal ODD 2020 model description for the living current model semantics v41 line.
- [`odd-d.md`](odd-d.md) — ODD+D human decision-making supplement for the living line.
- [`trace.md`](trace.md) — living TRACE evaluation / research-readiness dossier.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific-model specification.
- [`../research-principles.md`](../research-principles.md) — general research-software/scientific principles.
- [`scientific-audit-protocol.md`](scientific-audit-protocol.md) — reusable adversarial scientific-audit protocol and convergence criteria.
- [`audit-reverification-version-drift.md`](audit-reverification-version-drift.md) — rule for preserving frozen adversaries while permitting narrowly adapted current-state re-verification when a historical harness assumption becomes intrinsically obsolete.
- [`audit-v6/README.md`](audit-v6/README.md) — active sixth independent scientific-audit charter for immutable `v0.3.6` / model semantics v35.
- [`audit-v6/STATUS.md`](audit-v6/STATUS.md) — repository-authoritative Audit-v6 ledger; A–N discovery is complete with 14 findings (7 P1, 7 P2), and controlled remediation/re-verification is active.
- [`audit-v5/README.md`](audit-v5/README.md) — completed/historical fifth independent scientific-audit charter for immutable `v0.3.5` / model semantics v33; do not use it to restart Audit v5.
- [`audit-v5/STATUS.md`](audit-v5/STATUS.md) — final Audit-v5 ledger: 8 findings demonstrated (4 P1, 4 P2), 8/8 repaired/dispositioned and closed, with required P1 re-verification complete.
- [`audit-v4/README.md`](audit-v4/README.md) — completed/historical fourth independent scientific-audit charter for immutable `v0.3.4` / model semantics v25; do not use it to restart Audit v4.
- [`audit-v4/STATUS.md`](audit-v4/STATUS.md) — final Audit-v4 discovery/remediation ledger: 15 findings demonstrated on the frozen target, 15/15 repaired/re-verified and closed on the living line.
- [`post-v0.3.4-documentation-consistency-audit.md`](post-v0.3.4-documentation-consistency-audit.md) — first living-document audit against the repaired v33 state after Audit-v4 closure.
- [`post-v0.3.4-documentation-consistency-audit-2026-09-07-pass-2.md`](post-v0.3.4-documentation-consistency-audit-2026-09-07-pass-2.md) — second repository-wide documentation pass covering standing guidance, citation metadata, completed CI/audit/milestone status and archival boundaries missed by the first pass.
- [`v0.3.4-documentation-readiness-audit.md`](v0.3.4-documentation-readiness-audit.md) — historical pre-release documentation/version-identity convergence record for the v25 release line.
- [`v0.3.5-release-readiness.md`](v0.3.5-release-readiness.md) — historical v0.3.5/v33 release-preparation record after completed Audit-v4 remediation.
- [`v0.3.6-release-readiness.md`](v0.3.6-release-readiness.md) — v0.3.6/v35 release-preparation and documentation-convergence record after completed Audit-v5 remediation.

## Audit generations

- [`audit-v6/README.md`](audit-v6/README.md) and [`audit-v6/STATUS.md`](audit-v6/STATUS.md) define the sixth independent audit against immutable `v0.3.6` / v35. Fresh A–N discovery completed from zero and demonstrated 14 findings (7 P1, 7 P2); controlled remediation and required P1 re-verification are now active.
- [`audit-v5/README.md`](audit-v5/README.md) and [`audit-v5/STATUS.md`](audit-v5/STATUS.md) preserve the completed fifth independent audit against immutable `v0.3.5` / v33. It found 4 P1 and 4 P2 defects; all were repaired/dispositioned, producing v35, but the pass remains non-clean for convergence purposes.
- [`audit-v4/README.md`](audit-v4/README.md) and [`audit-v4/STATUS.md`](audit-v4/STATUS.md) preserve the completed fourth independent audit against immutable `v0.3.4` / v25. Its 15 findings were later repaired and independently re-verified/dispositioned to produce the v33 line frozen as v0.3.5.
- [`audit-v3/README.md`](audit-v3/README.md) and [`audit-v3/STATUS.md`](audit-v3/STATUS.md) preserve the completed third independent audit against immutable `v0.3.3` / v21. Its 17 findings were later repaired to produce the v25 line frozen as v0.3.4.
- [`audit-v2/STATUS.md`](audit-v2/STATUS.md) preserves the second independent audit against its own frozen baseline.
- dated `trace-audit-*` documents and other audit-area records remain historical evidence and must not be mechanically rewritten to current living-semantics language.

Historical audit records describe the exact baseline they challenged. The active Audit-v6 state is represented by [`audit-v6/STATUS.md`](audit-v6/STATUS.md); completed audit ledgers remain historical evidence rather than being rewritten to mimic the current repository.

## Core scientific contracts

- [`evidence-provenance.md`](evidence-provenance.md) — evidence provenance and transformation contract.
- [`spatial-boundary-dependence-v1.md`](spatial-boundary-dependence-v1.md) — finite-domain/boundary-dependence contract.
- [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md) — annual demographic-time contract: annual background risk is executed over elapsed M3 intervals with condition-mediated mortality as an order-invariant competing risk; year-end M2 performs fertility/parentage among survivors.
- [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md) — declared/synthetic founder state, reproductive history and genealogy contract.
- [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md) — derived demographic validation/diagnostic surface.
- [`m3-resource-time-contract-v1.md`](m3-resource-time-contract-v1.md) — exact elapsed-day annual resource accounting.
- [`m3-response-time-contract-v1.md`](m3-response-time-contract-v1.md) — independent M3 response and M4 decision timing semantics.
- [`spatial-mechanisms-v1.md`](spatial-mechanisms-v1.md) — M8 model-facing spatial transformation semantics.
- [`temporary-mobility-v1.md`](temporary-mobility-v1.md) — M9 residence/presence and temporary-journey semantics.
- [`m9-temporary-travel-semantics-v1.md`](m9-temporary-travel-semantics-v1.md) — M9 travel cost/duration/routing contract.
- [`m9-duration-aware-resource-semantics-v1.md`](m9-duration-aware-resource-semantics-v1.md) — temporary-presence resource accounting.
- [`temporary-mobility-observability-v1.md`](temporary-mobility-observability-v1.md) — M9 physical-presence observability contract.
- [`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md) — current demographic-baseline/model-analysis record; its checked result is rebound to v36 after a fresh 780-run confirmation reproduced the scientific payload exactly apart from source/semantics provenance, while earlier v35/v33 evidence remains historical.
- [`identifiability-equifinality-v1.md`](identifiability-equifinality-v1.md) — compatible-region and discriminating-prediction analysis contract.

Dated TRACE repair records preserve the rationale/evidence for individual historical repairs. Their version language is intentionally scoped to the repair they document.

## Active Audit-v6 convergence pass

Scientific Audit v6 restarts all Areas A–N from zero against immutable `v0.3.6` / `anthrosim-model-semantics-v35`. The target is the fully remediated Audit-v5 line, now frozen as a named release so discovery evidence cannot drift with living development.

Fresh A–N discovery demonstrated **14 findings: 7 P1 and 7 P2**, so Audit v6 is a **non-clean convergence pass**. Prior audits, green CI, release gates, preserved references and regression tests remain attack-hypothesis sources rather than substitutes for v6 evidence. Because discovery is complete, controlled production remediation is now permitted; every P1 remains open until independent post-merge adversarial re-verification succeeds.

No empirical/site-specific study should begin while v6 findings remain unresolved. After all findings are dispositioned and required P1 re-verifications pass, the repaired line must be frozen as a new immutable baseline and challenged by another genuinely fresh A–N audit. Only a P1-clean convergence pass satisfies the framework gate; any later empirical work still requires independent question-specific evidence, calibration/parameterization, uncertainty, sensitivity, identifiability/equifinality and domain review.

## Completed Audit-v5 outcome

Scientific Audit v5 restarted all Areas A–N from zero against immutable `v0.3.5` / `anthrosim-model-semantics-v33`. It demonstrated **8 findings: 4 P1 and 4 P2**. All eight are now repaired/dispositioned and closed on the living line, and every P1 repair received independent post-merge adversarial re-verification. Causal repairs advanced the living semantics to v35.

The v5 discovery result is nevertheless a **non-clean convergence pass**, because new P1 defects were discovered. That non-clean result is the reason v6 now audits the newly frozen `v0.3.6` / v35 baseline from zero rather than restarting or extending Audit v5.

Audit v5 remains framework/software scientific-verification evidence rather than empirical archaeological validation.

## Previous Audit-v4 outcome

Scientific Audit v4 restarted Areas A–N from zero against immutable `v0.3.4` / `anthrosim-model-semantics-v25`. It demonstrated **13 P1 and 2 P2 findings**. Post-discovery remediation repaired all 15 and independently re-verified/dispositioned them before closure. Authoritative repairs advanced the living line through model semantics v26–v33 where scientific continuation compatibility changed.

The immutable v0.3.5/model-semantics-v33 release line therefore differs scientifically from the immutable v0.3.4/v25 release target; this historical distinction is intentional. Historical v25 language remains correct only where it is explicitly release/audit identity or historical evidence.

The Audit-v4 result is framework-verification evidence, not empirical archaeological validity. TRACE remains **NOT YET EMPIRICALLY RESEARCH-READY** for a generic real-world inferential claim: question-specific problem formulation, evidence roles, calibration/validation where appropriate, uncertainty and sensitivity analysis, identifiability/equifinality assessment, held-out corroboration and domain review remain separate requirements.

## Benchmark and study records

The M7 resource-variability, M8.6 terrain and M9.7 aggregation exercises are preserved regression/capability evidence. Their machine-readable checked-in references can advance when an upstream authoritative repair legitimately changes trajectories; older reviewed references remain historical evidence rather than being overwritten conceptually.

The general demographic-baseline study likewise records model-form/structural-sensitivity evidence rather than a universal prehistoric population calibration.

Module- and milestone-specific documents in this directory provide the detailed evidence, contracts, assumptions and benchmark records referenced by the living standards documents above.
