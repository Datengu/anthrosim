# Post-v0.3.4 living-document consistency audit

Audit snapshot: protected `main` commit `f5ad364c5a0026f28abf92ef00612195edc8bec8` after Scientific Audit v4 remediation and final ledger synchronization.

## Purpose

This is a documentation-consistency audit, not a fifth scientific-behaviour audit. Its purpose is to verify that living/current-facing documentation describes the repaired repository state after the immutable `v0.3.4` / `anthrosim-model-semantics-v25` release baseline, while preserving release notes, audit discovery records and per-semantics re-verification records as historical evidence.

## Authoritative current state

At the audit snapshot:

- workspace package version remains `0.3.4`; this is the development package version and does not rewrite the immutable `v0.3.4` release tag;
- executable `MODEL_SEMANTICS_ID` is `anthrosim-model-semantics-v33`;
- current checkpoint schema is 21;
- Scientific Audit v4 discovery and post-discovery remediation are complete;
- the frozen v0.3.4/v25 audit target demonstrated 13 P1 and 2 P2 findings;
- all 15 Audit-v4 findings are repaired, independently re-verified/dispositioned and closed;
- none of that establishes empirical validity or research readiness for a particular archaeological or anthropological inference.

The executable source, current Audit-v4 status ledger and living ODD/scientific-model surfaces are the primary cross-checks for this audit.

## Scope and classification rule

Audited current-facing surfaces include the repository README, architecture, roadmap, release/versioning policy, research-documentation index, TRACE dossier, ODD/ODD+D/scientific-model status, benchmark-result current-reference labels, model-semantics ADR wording, and the permanent documentation-consistency guard.

Historical material is intentionally not normalized to v33 when it is explicitly scoped to an immutable release, audit target, old model-semantics generation, dated repair record or evidence execution. In particular, `docs/releases/`, `docs/research/audit-v2/`, `docs/research/audit-v3/`, Audit-v4 discovery evidence, and `research/.../model-semantics-v*-reverification.md` retain the identities they actually evaluated.

## Findings

### DOC-001 — top-level README still presented v27 and Audit-v4 remediation as current

The README called the living source tree current model semantics v27 and said Audit-v4 repairs/re-verification were still the next priority. Current executable provenance is v33 and Audit-v4 is complete with 15/15 findings closed.

Disposition: update the living status and scientific-status narrative while preserving immutable v0.3.4/v25 and v0.3.3/v21 identities.

### DOC-002 — TRACE living dossier lagged the repaired state

TRACE still identified v27, called Audit-v4 remediation in progress, treated Audit v3 as the latest framework-level closure evidence, and assessed model description primarily against v0.3.4/v25 rather than the living v33 line.

Disposition: update TRACE to the fourth completed audit generation and current v33 model-description state without upgrading `NOT YET EMPIRICALLY RESEARCH-READY`.

### DOC-003 — research-documentation index stopped at early Audit-v4 remediation

`docs/research/README.md` linked Audit v3 prominently but omitted the completed Audit-v4 charter/status and version-drift re-verification addendum. Its summary stopped at AV4-001/v26.

Disposition: add the current Audit-v4 records and summarize full 15/15 closure on v33.

### DOC-004 — roadmap still described v27 and an active remediation sequence

The roadmap called the M8.6 current reference v27 and described Audit-v4 repair/re-verification as pending. The checked-in M8.6 machine reference is v33 and Audit-v4 remediation is complete.

Disposition: update current-reference and post-M9 direction language while retaining historical release identities and the question-led/no-fixed-M10 policy.

### DOC-005 — architecture lagged repaired scheduler and stochastic-coupling semantics

The architecture still summarized the M4 stochastic schedule at v28 and used an over-broad annual-boundary sentence for deaths. Current v33 semantics include person/household scientific coupling across fertility, both mortality streams, parentage, M4 scheduling, M9 tied destinations, scarce-resource remainder ties and M4 spatial-candidate equivalence classes. Background and condition-mediated mortality are resolved over elapsed M3 intervals; year-end M2 performs fertility/parentage among survivors.

Disposition: describe the current architecture rather than treating v28 as the terminal state.

### DOC-006 — ADR 0004 used a stale dynamic value

ADR 0004 said `MODEL_SEMANTICS_ID` was “currently” v1. The decision remains valid, but the literal current value is a moving executable property.

Disposition: preserve v1 as the value when the ADR was introduced and point current value authority to `provenance.rs`; record v33 only as this audit snapshot.

### DOC-007 — permanent current-semantics documentation guard was too narrow

`scripts/test-current-model-semantics-docs.py` watched only scientific-model/ODD/ODD+D and hard-coded v33. Consequently README, TRACE, roadmap and other living surfaces could drift while CI stayed green.

Disposition: derive the living semantics identity from executable provenance and extend the guarded living-document set and stale-current-pattern checks. Immutable release identities remain explicitly fixed.

### DOC-008 — release/versioning policy did not state the present post-release development identity

The policy correctly preserves v0.3.4/v25 but did not explicitly state that current `main` can still carry package version 0.3.4 while authoritative semantics have advanced to v33.

Disposition: add that distinction without creating or implying a new named release.

### DOC-009 — M8.6/M9.7 result narratives lagged their checked-in machine references

The M8.6 result document still labelled v29 as the current regression reference while its checked-in reference JSON is v33. The M9.7 result document still labelled v29 current while its checked-in reference JSON is v31. Older sections remain valid historical review records.

Disposition: make the actual checked-in machine-reference identity explicit at the top/current-result interpretation, and relabel older v29 material as historical rather than deleting it.

## Result

The remediation associated with this audit synchronizes living documentation to the repaired current state and strengthens CI so future model-semantics drift is detected across the principal current-facing surfaces. Historical evidence remains historically accurate instead of being mechanically rewritten to the newest identity.

This audit establishes documentation/code/status consistency only. It does not establish that AnthroSim is scientifically correct, empirically calibrated, archaeologically validated, or ready for a particular inferential study.
