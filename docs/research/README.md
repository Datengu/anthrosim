# AnthroSim research documentation

AnthroSim's research documentation is organized around explicit model description, human-decision assumptions, model evaluation, evidence provenance and reproducible experiment records.

**Living framework state:** software version `v0.3.5` / current model semantics v34. Immutable `v0.3.5` is the Scientific Audit-v5 discovery target at model semantics v33; immutable `v0.3.4` remains the historical Audit-v4 target at model semantics v25. Current documentation must not rewrite either historical audit identity.

## Start here

- [`research-standards.md`](research-standards.md) — formal adoption and maintenance rules for **ODD 2020, ODD+D and TRACE**.
- [`odd.md`](odd.md) — formal ODD 2020 model description for the living current model semantics v34 line.
- [`odd-d.md`](odd-d.md) — ODD+D human decision-making supplement for the living line.
- [`trace.md`](trace.md) — living TRACE evaluation / research-readiness dossier.
- [`../scientific-model.md`](../scientific-model.md) — detailed normative scientific-model specification.
- [`../research-principles.md`](../research-principles.md) — general research-software/scientific principles.
- [`scientific-audit-protocol.md`](scientific-audit-protocol.md) — reusable adversarial scientific-audit protocol and convergence criteria.
- [`audit-reverification-version-drift.md`](audit-reverification-version-drift.md) — rule for preserving frozen adversaries while permitting narrowly adapted current-state re-verification when a historical harness assumption becomes intrinsically obsolete.
- [`audit-v5/README.md`](audit-v5/README.md) — active fifth independent scientific-audit charter for immutable `v0.3.5` / model semantics v33.
- [`audit-v5/STATUS.md`](audit-v5/STATUS.md) — repository-authoritative Audit-v5 discovery/remediation ledger; begin here when reconstructing or continuing the active audit.
- [`audit-v4/README.md`](audit-v4/README.md) — completed/historical fourth independent scientific-audit charter for immutable `v0.3.4` / model semantics v25; do not use it to restart Audit v4.
- [`audit-v4/STATUS.md`](audit-v4/STATUS.md) — final Audit-v4 discovery/remediation ledger: 15 findings demonstrated on the frozen target, 15/15 repaired/re-verified and closed on the living line.
- [`post-v0.3.4-documentation-consistency-audit.md`](post-v0.3.4-documentation-consistency-audit.md) — first living-document audit against the repaired v33 state after Audit-v4 closure.
- [`post-v0.3.4-documentation-consistency-audit-2026-09-07-pass-2.md`](post-v0.3.4-documentation-consistency-audit-2026-09-07-pass-2.md) — second repository-wide documentation pass covering standing guidance, citation metadata, completed CI/audit/milestone status and archival boundaries missed by the first pass.
- [`v0.3.4-documentation-readiness-audit.md`](v0.3.4-documentation-readiness-audit.md) — historical pre-release documentation/version-identity convergence record for the v25 release line.
- [`v0.3.5-release-readiness.md`](v0.3.5-release-readiness.md) — v0.3.5/v33 release-preparation record after completed Audit-v4 remediation and the two post-audit documentation consistency passes.

## Earlier audit generations

- [`audit-v4/README.md`](audit-v4/README.md) and [`audit-v4/STATUS.md`](audit-v4/STATUS.md) preserve the completed fourth independent audit against immutable `v0.3.4` / v25. Its 15 findings were later repaired and independently re-verified/dispositioned to produce the v33 line frozen as v0.3.5.
- [`audit-v3/README.md`](audit-v3/README.md) and [`audit-v3/STATUS.md`](audit-v3/STATUS.md) preserve the completed third independent audit against immutable `v0.3.3` / v21. Its 17 findings were later repaired to produce the v25 line frozen as v0.3.4.
- [`audit-v2/STATUS.md`](audit-v2/STATUS.md) preserves the second independent audit against its own frozen baseline.
- dated `trace-audit-*` documents and other audit-area records remain historical evidence and must not be mechanically rewritten to current v33 language.

Historical audit records describe the exact baseline they challenged. The active Audit-v5 state is represented by [`audit-v5/STATUS.md`](audit-v5/STATUS.md); earlier ledgers remain immutable historical evidence rather than being rewritten to mimic the current repository.

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
- [`general-scientific-demographic-baseline-v1.md`](general-scientific-demographic-baseline-v1.md) — current demographic-baseline/model-analysis record, including the v33 re-verification state.
- [`identifiability-equifinality-v1.md`](identifiability-equifinality-v1.md) — compatible-region and discriminating-prediction analysis contract.

Dated TRACE repair records preserve the rationale/evidence for individual historical repairs. Their version language is intentionally scoped to the repair they document.

## Current Audit-v5 state

Scientific Audit v5 starts all Areas A–N from zero against immutable `v0.3.5` / `anthrosim-model-semantics-v33`. Prior Audit-v4 findings, repairs, evidence PRs, release gates and permanent regressions are attack ideas and controls only; they do not count as v5 coverage.

The intended outcome is a fresh P1-clean convergence pass. Discovery remains separate from remediation: demonstrated defects are preserved as sequential `AV5-*` findings, full A–N discovery completes before ordinary production repair begins, and later P0/P1 repairs require independent post-merge re-verification under the scientific-audit protocol.

Audit v5 remains framework-verification evidence rather than empirical archaeological validation. The authoritative current state and next work item are always recorded in [`audit-v5/STATUS.md`](audit-v5/STATUS.md).

## Previous Audit-v4 outcome

Scientific Audit v4 restarted Areas A–N from zero against immutable `v0.3.4` / `anthrosim-model-semantics-v25`. It demonstrated **13 P1 and 2 P2 findings**. Post-discovery remediation repaired all 15 and independently re-verified/dispositioned them before closure. Authoritative repairs advanced the living line through model semantics v26–v33 where scientific continuation compatibility changed.

The v0.3.5/current-model-semantics-v33 line therefore differs scientifically from the immutable v0.3.4/v25 release target; this is intentional. Historical v25 language remains correct only where it is explicitly release/audit identity or historical evidence.

The Audit-v4 result is framework-verification evidence, not empirical archaeological validity. TRACE remains **NOT YET EMPIRICALLY RESEARCH-READY** for a generic real-world inferential claim: question-specific problem formulation, evidence roles, calibration/validation where appropriate, uncertainty and sensitivity analysis, identifiability/equifinality assessment, held-out corroboration and domain review remain separate requirements.

## Benchmark and study records

The M7 resource-variability, M8.6 terrain and M9.7 aggregation exercises are preserved regression/capability evidence. Their machine-readable checked-in references can advance when an upstream authoritative repair legitimately changes trajectories; older reviewed references remain historical evidence rather than being overwritten conceptually.

The general demographic-baseline study likewise records model-form/structural-sensitivity evidence rather than a universal prehistoric population calibration.

Module- and milestone-specific documents in this directory provide the detailed evidence, contracts, assumptions and benchmark records referenced by the living standards documents above.
