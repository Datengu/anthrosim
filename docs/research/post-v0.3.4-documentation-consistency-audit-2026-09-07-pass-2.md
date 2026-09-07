# Post-v0.3.4 documentation consistency audit — 2026-09-07 pass 2

**Audit type:** repository-wide documentation/current-state consistency audit  
**Audited protected-main baseline:** `cd5267cf875733e91d90e5e169a63091761323f6`  
**Baseline package version:** `0.3.4`  
**Baseline executable model semantics:** `anthrosim-model-semantics-v33`  
**Immutable named release baseline:** `v0.3.4` / `anthrosim-model-semantics-v25`  
**Scientific-behaviour audit:** no — this pass does not reopen Scientific Audit v4 or re-audit simulation behaviour

## Purpose

This is a second independent documentation-consistency pass after the first post-v0.3.4 synchronization was merged in PR #601. The purpose is to check surfaces outside the narrower current-model-semantics guard and ensure that repository documentation cannot reasonably be mistaken about current milestone/release state, completed audit state, citation identity, CI/governance rollout, or whether historical implementation records are still active plans.

The audit deliberately separates three classes:

1. **Living/current-facing guidance** must describe the present repository state.
2. **Immutable release/audit/evidence records** must retain the identities and conclusions they actually evaluated; old numbers inside such records are not stale merely because `main` advanced.
3. **Historical milestone/implementation records** retain their original evidence, but receive an explicit archival/status marker when pre-closure wording could otherwise be mistaken for current project status.

## Authoritative identities checked

The pass rechecked the following against the audited baseline:

- executable `MODEL_SEMANTICS_ID` authority in `crates/anthrosim-core/src/provenance.rs`: `anthrosim-model-semantics-v33`;
- workspace package/current citation release line: `0.3.4`;
- immutable `v0.3.4` release semantics: v25;
- immutable `v0.3.3` release semantics: v21;
- immutable `v0.3.2` release semantics: v19;
- Audit-v4 `STATUS.md`: A–N complete, 15/15 findings repaired/reverified/dispositioned, no open Audit-v4 finding;
- current M8.6 checked-in machine reference: model semantics v33;
- current M9.7 checked-in machine reference: model semantics v31, intentionally retained because v32/v33 pathways are causally inapplicable to that frozen benchmark;
- current open repository work at audit start: no open PRs or issues and only protected `main` existed.

## Findings and dispositions

### DOC2-001 — standing agent guidance still planned an already completed M9

`AGENTS.md` described M9 as the planned temporary-mobility/aggregation milestone and `v0.3.0` as its future target release.

**Disposition:** describe M8 and M9 as completed/released, distinguish later post-M9 patch releases from living model-semantics development, and explicitly retain the question-led rule rather than inventing a fixed M10 feature list.

### DOC2-002 — vision still presented completed M8 as the next development boundary

`docs/vision.md` called evidence-grounded environments a natural next step and linked to a planned M8 boundary.

**Disposition:** state that M8 evidence-grounded spatial experiments and M9 temporary mobility/controlled aggregation are established capabilities; future substantive work remains question-led.

### DOC2-003 — citation guidance still described v0.1 metadata as current

`docs/research/citation.md` said repository citation metadata identified version `0.1.0`, while root `CITATION.cff` identifies the current named release metadata as `0.3.4`.

**Disposition:** synchronize guidance to `0.3.4`, preserve the immutable named-release versus living-main distinction, and require exact Git/model-semantics provenance for research using an unreleased development state.

### DOC2-004 — path-aware CI documentation still described completed #317 as an active rollout

`docs/ci-path-classification.md` said AnthroSim was “introducing” path-aware CI under #317 and referred to future #317 changes even though the issue is closed as completed.

**Disposition:** describe the mechanism as current enforcement originally implemented under completed #317; future changes are classifier/path-routing changes rather than continuation of the closed issue.

### DOC2-005 — required-check documentation still described closed #175 as governance-incomplete

`docs/required-status-checks.md` retained the pre-reconciliation instruction saying #175 was incomplete until the aggregator was added to live branch protection. Issue #175 is closed as completed and the repository-side intended list already contains the aggregator as context 24.

**Disposition:** record #175 as the completed rollout/reconciliation issue and preserve the rule that any later administrative branch-protection change must reverify the live required set.

**Audit limitation:** the connected GitHub integration cannot read the live `main` branch-protection administration endpoint (`403 Resource not accessible by integration`). This pass therefore does not claim a fresh independent API read of the current administrator-only rule. It records the repository contract and closed issue state without overstating tool access.

### DOC2-006 — completed M9 implementation sequence still looked like an active plan

`docs/research/m9-implementation-sequence.md` was labelled an active implementation plan and still described `v0.3.0` as planned.

**Disposition:** preserve the sequence as historical implementation evidence, mark it explicitly historical, and record that it culminated in completed/audited `v0.3.0`.

### DOC2-007 — M9.6 records exposed pre-closure status without an archival boundary

`m9-6-acceptance.md` and `m9-6-integration-audit.md` correctly preserve the state that existed during M9.6, including statements that final validation/M9.7 were still pending. Without a banner, those correct historical statements can be mistaken for current project status.

**Disposition:** add explicit historical-record banners and label their status as “at time of record”; do not rewrite the underlying evidence or later-project assumptions into the historical body.

### DOC2-008 — completed Audit-v3/v4 charters could instruct a new agent to restart closed audits

Both audit charters retained cross-session instructions to continue the next incomplete audit task. Their authoritative ledgers now record completed audits; for v4, all 15 findings are repaired/reverified/dispositioned.

**Disposition:** add clear complete/historical-charter banners and replace restart instructions with reconstruction-only guidance. Preserve frozen audit targets, methodology and historical operating rules unchanged.

### DOC2-009 — formal living model descriptions used ambiguous “Audit-v4 remediation line” wording after closure

`docs/scientific-model.md`, `docs/research/odd.md`, and `docs/research/odd-d.md` correctly identified current v33 but described it as the “Audit-v4 remediation line”. That lineage description was technically defensible but could imply remediation remained active.

**Disposition:** clarify that these are the repaired post-v0.3.4 living documents **after completed Audit-v4 remediation**, without changing current semantics or immutable release identities.

## Permanent guard extension

`scripts/test-current-model-semantics-docs.py` is extended beyond its original eight principal model/status surfaces to guard the new failure modes discovered here:

- stale planned-M9 language in `AGENTS.md`;
- stale planned-M8/“next step” language in the vision;
- citation guidance versus `CITATION.cff` release version;
- completed #317/#175 rollout wording;
- required historical markers on the M9 implementation/M9.6 records;
- required completion/do-not-restart markers on the Audit-v3/v4 charters.

Historical release/audit/evidence records remain excluded from any rule that would mechanically rewrite them to the newest living model semantics.

## Surfaces rechecked with no correction required

The following principal surfaces were rechecked and remained consistent at the audited baseline:

- root `README.md`;
- `docs/roadmap.md`;
- `docs/architecture.md`;
- `docs/research/README.md` current-state material;
- `docs/research/trace.md`;
- `docs/release-versioning.md`;
- ADR 0004 model-semantics compatibility identity;
- M8.6 current benchmark-result narrative/reference binding;
- M9.7 current benchmark-result narrative/reference binding;
- Audit-v4 authoritative `STATUS.md`;
- `CONTRIBUTING.md`, `SECURITY.md`, source-provenance, research-integrity, research-principles, run-directory recovery and current CI-topology documentation.

Older release notes, dated TRACE/audit reports, audit-v2/v3 evidence, release-readiness audits and versioned implementation contracts were intentionally not rewritten merely because they mention an older package/model identity. They are historical evidence, not current-state claims.

## Result

The first post-v0.3.4 documentation synchronization was materially correct on its targeted surfaces, but it was not repository-wide: this second pass found **nine additional documentation-status/clarity findings** outside or at the edge of the original guard. The repair changes are documentation/guard changes only and do not alter authoritative simulator behaviour, package version, model semantics, immutable release identities, or scientific benchmark conclusions.

The appropriate completion criterion for this pass is green exact-head CI on the repair PR plus reinspection that the corrected living/historical boundaries are present. Passing this audit does not establish empirical or archaeological readiness.
