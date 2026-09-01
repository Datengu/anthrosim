# AnthroSim release and versioning policy

AnthroSim uses several independent identities. They answer different questions and should not be collapsed into one numbering system.

## Identity layers

- **Roadmap milestone (`M#`)** answers: what research/development capability stage has been reached?
- **Software version (`vMAJOR.MINOR.PATCH`)** answers: which named release of AnthroSim is this?
- **Model semantics identity** answers: is the authoritative scientific/model meaning compatible with another run or checkpoint?
- **Git source revision** answers: exactly which source revision built the executable that produced a run?

Run provenance already records the software/model version, model-semantics identity and exact Git source identity. The Git identity is therefore the authoritative way to distinguish individual code changes; software versions are not commit counters.

## Versioning rules

AnthroSim uses semantic-style versioning while it remains pre-1.0:

- **Do not bump the package version for every commit or pull request.** Exact code identity is already captured by Git provenance.
- **Patch releases** (`0.x.Y`) are for named compatible bug-fix or small maintenance releases after a minor release has been cut.
- **Minor releases** (`0.X.0`) are for meaningful, deliberately named capability releases. From M9 onward, completion of a major roadmap milestone is normally expected to culminate in the next minor AnthroSim release after milestone acceptance, audit/hardening and release verification.
- **Major release `1.0.0`** is reserved for a separately defined stable/research-ready compatibility baseline. Reaching a particular milestone number does not automatically imply `1.0.0`.

A milestone number and software version remain intentionally independent. A milestone describes capability; a release identifies a named, preserved software baseline. A valid description may therefore look like `AnthroSim v0.3.1 — M9`.

The normal post-M8 lifecycle is:

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
cut and preserve the next minor release
```

This expected cadence does **not** make milestone completion and release publication the same event. It preserves the useful distinction demonstrated by M8: the capability can be complete before the audited release baseline representing it is cut.

## Current milestone-to-release targets

The practical release cadence is currently:

- **M8 → `v0.2.0`**: completed, audited and released as the preserved M8 baseline.
- **M9 → `v0.3.0`**: completed, audited and released as the preserved temporary-mobility / controlled-aggregation baseline.
- **`v0.3.1`**: post-M9 scientific-audit and research-readiness hardening patch.
- **`v0.3.2`**: documentation-convergence maintenance patch over the v19 model semantics preserved by the immutable `v0.3.2` tag; it corrects living-document drift present when v0.3.1 was tagged.
- **`v0.3.3`**: post-scientific-audit-v2 hardening/convergence patch preserving the repaired v21 model-semantics baseline before another fresh independent release-baseline audit.
- **Later major milestones** should normally target the next minor release in sequence unless the repository deliberately records a reason to do otherwise.

The living development line can and does advance its model-semantics identity after a named release. That does not rewrite the identity of a preserved release tag: `v0.3.2` remains v19, while the audited `v0.3.3` baseline preserves the later repaired v21 scientific state.

These are release targets, not identity equivalences. `M9` does not literally mean `v0.3.0`, and patch releases can exist within an already completed milestone.

During ordinary M9 implementation work the package version remained on the v0.2 line. The change to `0.3.0` is made only by the explicit v0.3.0 release-preparation change after M9 acceptance and post-milestone audit/reverification.

## Scientific changes are separate from release numbering

A software version bump does not by itself say that scientific meaning changed. Changes that alter authoritative simulation meaning must also follow the model-semantics compatibility policy and update `MODEL_SEMANTICS_ID` when checkpoint continuation or interpretation would become scientifically incompatible.

Documentation, packaging, explorer-only or other source-neutral changes may warrant a software release without changing model semantics. Conversely, a scientifically meaningful change must never be hidden behind an unchanged semantics identity merely because the package version is unchanged.

Each milestone release candidate must therefore independently review whether the milestone changed authoritative scientific/model meaning. The expected minor-version increase does not replace that review.

The `v0.3.3` release-preparation change itself does not introduce new authoritative model behaviour; it names and preserves the already-reviewed v21 state produced by the audit-v2 repair line. Therefore the release version changes while `MODEL_SEMANTICS_ID` remains v21.

## Milestone completion versus hardening

A completed roadmap milestone is not automatically reopened because a later audit discovers defects.

Use this distinction:

- a defect in functionality that was implemented for the milestone is **post-milestone hardening/remediation**;
- functionality required by the milestone acceptance criteria that was never actually implemented means the milestone was **not fully complete** and its status should be corrected.

Audit findings that affect correctness, determinism, reproducibility, provenance, data integrity, or stated milestone acceptance criteria should be resolved before cutting the named release intended to represent that milestone as a stable baseline.

## M8 / v0.2.0 baseline

M8 is completed and its audited baseline was released as `v0.2.0` on 2026-08-24. The exact release commit is preserved by the `v0.2.0` tag.

Compatible defects discovered in a preserved release baseline should normally be fixed in a patch release on the relevant release line when a named maintenance release is warranted. M9 development did not use opportunistic package-version bumps; its completed and audited minor release is `v0.3.0`.

## Preserving named release commits

A named release is not complete until its exact Git commit is preserved by an immutable-intent SemVer tag such as `v0.3.0`.

Use the `Preserve named release tag` workflow only after release preparation and exact-SHA verification are complete. Supply both the SemVer tag and the full 40-character commit SHA.

### Existing tags

Published release identity remains fail-closed and immutable:

- if the requested tag already resolves to the requested commit, the workflow succeeds without changing it;
- if the requested tag already resolves to another commit, the workflow refuses to move or rewrite it.

This idempotent existing-tag path preserves historical releases without retroactively applying newer release-candidate rules to a tag that already exists.

### Creating a new tag

Creating a missing named release tag is deliberately stricter than merely proving that a Git object exists. Before mutation, the workflow verifies that the supplied candidate itself proves the requested release identity:

1. the requested tag matches `vMAJOR.MINOR.PATCH` and the SHA is an exact lowercase 40-character commit identity;
2. the candidate is the **current protected `main` HEAD**, not an arbitrary older repository commit;
3. the root workspace package version equals the requested tag version;
4. `CITATION.cff` declares the same version;
5. `docs/releases/<tag>.md` exists and identifies the same release;
6. every status context currently required by protected `main` is successful for that exact SHA;
7. for named releases from `v0.3.0` onward, the exact SHA also has successful release-specific dispositions for:
   - `Execute predeclared terrain null-model benchmark` (M8.6);
   - `Execute predeclared M9.7 aggregation benchmark` (M9.7);
   - `RustSec dependency audit`.

The workflow prints the resolved tag, commit and gate disposition before creating the tag. Only after all checks pass does it create the lightweight `refs/tags/<version>` reference, and it immediately verifies that the new ref resolves to the exact requested SHA.

The current-`main` rule is intentional. A release candidate should be tagged **before unrelated subsequent work advances `main`**. If `main` has already advanced, prepare and verify the intended new HEAD rather than permanently assigning a new release identity to an older arbitrary commit through the tagging workflow.

The M8.6, M9.7 and RustSec checks are release-specific exact-SHA evidence, not substitutes for ordinary protected-main checks. When they are not produced automatically for a release-preparation change, run the corresponding workflows explicitly against the final release candidate before dispatching the tag workflow. The release workflow fails closed if those exact-SHA checks are absent, pending, skipped, cancelled or failing.

The release-candidate verifier also refuses a truncated check-run response rather than assuming unobserved checks succeeded.

The workflow retains a one-time push bootstrap for the already-audited `v0.2.0` release that predates repository-side release-candidate enforcement. Named release creation from `v0.3.0` onward uses the explicit manual workflow-dispatch path.

Creating or preserving a release tag does not change `MODEL_SEMANTICS_ID`, package contents, or simulation semantics. It only preserves the exact source identity of the named software release.

## Agent and contributor rule

Agents and contributors must not opportunistically change the package version while implementing ordinary issues. A version bump should be an explicit release decision or part of a task that specifically calls for a named release.

From M9 onward, agents should plan on a completed major milestone normally culminating in the next minor release, while still treating milestone completion, release publication, model-semantics identity and Git source identity as separate concerns. When uncertain during implementation, leave the package version unchanged and preserve exact provenance through the Git commit identity.
