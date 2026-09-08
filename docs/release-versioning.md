# AnthroSim release and versioning policy

AnthroSim uses several independent identities. They answer different questions and should not be collapsed into one numbering system.

## Identity layers

- **Roadmap milestone (`M#`)** — what research/development capability stage has been reached?
- **Software version (`vMAJOR.MINOR.PATCH`)** — which named release/package version is this?
- **Model semantics identity** — is authoritative scientific/model meaning compatible with another run or checkpoint?
- **Git source revision** — exactly which source revision built the executable that produced an artifact or execution segment?

Run provenance records software/model version, model-semantics identity and exact Git source identity. Git is therefore the authoritative way to distinguish individual source changes; software versions are not commit counters.

## Versioning rules

AnthroSim uses semantic-style versioning while pre-1.0:

- **Do not bump package version for every commit or PR.**
- **Patch releases** (`0.x.Y`) are named compatible bug-fix/maintenance releases.
- **Minor releases** (`0.X.0`) are meaningful named capability releases; major roadmap milestones normally culminate in a later minor release after acceptance/audit/hardening, but milestone and release identity remain separate.
- **Major `1.0.0`** is reserved for a separately defined stable/research-ready compatibility baseline; no milestone number implies it automatically.

A milestone describes capability. A release identifies a named preserved source baseline. Model semantics describes scientific continuation compatibility. Git identifies exact source. These identities may advance independently.

## Current release history and living development line

The practical release history is:

- **M8 → `v0.2.0`**: completed, audited and released as the preserved M8 baseline.
- **M9 → `v0.3.0`**: completed, audited and released as the preserved temporary-mobility / controlled-aggregation baseline.
- **`v0.3.1`**: post-M9 scientific-audit and research-readiness hardening patch.
- **`v0.3.2`**: documentation-convergence maintenance patch over the v19 model semantics preserved by the immutable `v0.3.2` tag; it corrects living-document drift present when v0.3.1 was tagged.
- **`v0.3.3`**: post-scientific-audit-v2 hardening/convergence patch preserving the repaired v21 model-semantics baseline before another fresh independent release-baseline audit.
- **`v0.3.4`**: post-Scientific-Audit-v3 convergence patch preserving the fully remediated and independently reverified v25 model-semantics baseline.
- **`v0.3.5`**: post-Scientific-Audit-v4 repaired convergence patch preserving the fully remediated/reverified v33 model-semantics baseline plus the completed post-audit documentation-consistency reconciliation.

The immutable `v0.3.4` tag remains `anthrosim-model-semantics-v25`. Scientific Audit v4 subsequently audited that frozen target, and authoritative repairs advanced the living line through v26–v33. **The immutable `v0.3.5` release remains package version `0.3.5` at `MODEL_SEMANTICS_ID = anthrosim-model-semantics-v33`.** It names and preserves the fully repaired Audit-v4 state; later Audit-v5 remediation does not retroactively mutate that tag or its evidence.

Post-`v0.3.5` living development may keep workspace package version `0.3.5` while model semantics advance when a scientifically causal repair requires it. Audit-v5 AV5-001/#606 is such a change: the living line advances from v33 to `anthrosim-model-semantics-v34` because future equal-cost M9 destinations use a household-local coupling identity instead of a globally ordinal population rank. This is an unreleased development state until a later explicit release-preparation task assigns a new software version/tag.

The same distinction applies to older releases: `v0.3.2` remains v19 and `v0.3.3` remains v21 even though living main is newer.

Later major milestones should normally target the next minor release in sequence unless the repository deliberately records another release plan.

## Normal milestone/release lifecycle

```text
implement milestone capability
        ↓
meet milestone acceptance criteria
        ↓
declare milestone complete
        ↓
audit / harden / reverify the completed capability
        ↓
prepare the named release candidate
        ↓
cut and preserve the release
```

This does not make milestone completion and release publication the same event. Capability may be complete before the audited release baseline representing it is cut.

## Scientific changes are separate from release numbering

A software version bump does not by itself say scientific meaning changed. Changes that alter authoritative simulation meaning must follow the model-semantics compatibility policy and update `MODEL_SEMANTICS_ID` when checkpoint continuation or interpretation would become scientifically incompatible.

Documentation, packaging, explorer-only or other source-neutral changes may warrant a software release without changing model semantics. Conversely, a scientifically meaningful change must never be hidden behind an unchanged semantics identity merely because package version is unchanged.

The `v0.3.4` release-preparation change itself did not introduce new authoritative model behaviour; it named and preserved the already-reviewed v25 state produced by Audit-v3 remediation. Audit-v4 repairs after that release advanced authoritative semantics through v33. The `v0.3.5` release-preparation change likewise introduced no new causal model behaviour: it assigned a named patch-release identity to the already-reviewed v33 repaired state after Audit-v4 closure and documentation reconciliation. Later Audit-v5 causal repairs can therefore advance the living semantics identity without rewriting the preserved `v0.3.5` release.

## Milestone completion versus hardening

A completed roadmap milestone is not automatically reopened because a later audit discovers defects.

- a defect in functionality already implemented for the milestone is **post-milestone hardening/remediation**;
- functionality required by milestone acceptance criteria that was never implemented means milestone status should be corrected.

Audit findings affecting correctness, determinism, reproducibility, provenance, data integrity or stated acceptance criteria should be resolved before cutting a named release intended to represent the repaired state as a stable baseline.

## Preserving named release commits

A named release is not complete until its exact Git commit is preserved by an immutable-intent SemVer tag such as `v0.3.0`.

Use the `Preserve named release tag` workflow only after release preparation and exact-SHA verification are complete. The workflow is fail-closed:

- an existing tag that already resolves to the requested commit is accepted without mutation;
- an existing tag that resolves elsewhere is not moved;
- a new tag must match `vMAJOR.MINOR.PATCH`, target the exact current protected-main release candidate, agree with workspace/CITATION/release-note identity, and have the required exact-SHA protected/release-specific gates green before creation.

Named releases from `v0.3.0` onward additionally require the repository's release-specific benchmark/dependency-audit dispositions where configured. A release should be tagged before unrelated later work advances `main`; the tagging workflow must not assign a new release identity to an arbitrary older commit simply because the object exists.

Creating/preserving a release tag does not itself change `MODEL_SEMANTICS_ID`, package contents or simulation semantics.

## Agent and contributor rule

Agents and contributors must not opportunistically change package version during ordinary issue implementation. A version bump should be an explicit release decision/task. When uncertain, leave the package version unchanged and preserve exact provenance through Git/source/model identities.

## Exceptional privacy-driven history rewrites

Release tags are immutable-intent scientific/version identities and must not ordinarily move. The sole exception is an explicitly authorized repository-wide privacy/sensitive-data sanitisation where retaining original Git objects would preserve information that must be removed.

When such an exceptional rewrite occurs:

- release version and `MODEL_SEMANTICS_ID` do not change merely because Git object identities change;
- rewritten tags must preserve released scientific semantics apart from non-semantic sanitisation;
- living provenance documentation must reconcile to rewritten tag SHA and state that a privacy-driven rewrite occurred;
- the rewrite is not a new scientific validation or release;
- subsequent releases use ordinary exact-candidate protected/release verification.

On 2026-09-02, AnthroSim underwent such an authorized privacy sanitisation. Historical release-tag commit identities were rewritten while release versions and model-semantics identities were preserved.
