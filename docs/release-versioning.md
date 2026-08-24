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
- **M9 → target `v0.3.0`**: after M9 is completed, its material correctness/reproducibility findings are resolved and the intended release candidate passes the required verification.
- **Later major milestones** should normally target the next minor release in sequence unless the repository deliberately records a reason to do otherwise.

These are release targets, not identity equivalences. `M9` does not literally mean `v0.3.0`, and patch releases can exist within an already completed milestone.

During ordinary M9 implementation work the package version should remain at the current released line unless an explicit release task says otherwise. The version change belongs in release preparation rather than routine milestone PRs.

## Scientific changes are separate from release numbering

A software version bump does not by itself say that scientific meaning changed. Changes that alter authoritative simulation meaning must also follow the model-semantics compatibility policy and update `MODEL_SEMANTICS_ID` when checkpoint continuation or interpretation would become scientifically incompatible.

Documentation, packaging, explorer-only or other source-neutral changes may warrant a software release without changing model semantics. Conversely, a scientifically meaningful change must never be hidden behind an unchanged semantics identity merely because the package version is unchanged.

Each milestone release candidate must therefore independently review whether the milestone changed authoritative scientific/model meaning. The expected minor-version increase does not replace that review.

## Milestone completion versus hardening

A completed roadmap milestone is not automatically reopened because a later audit discovers defects.

Use this distinction:

- a defect in functionality that was implemented for the milestone is **post-milestone hardening/remediation**;
- functionality required by the milestone acceptance criteria that was never actually implemented means the milestone was **not fully complete** and its status should be corrected.

Audit findings that affect correctness, determinism, reproducibility, provenance, data integrity, or stated milestone acceptance criteria should be resolved before cutting the named release intended to represent that milestone as a stable baseline.

## M8 / v0.2.0 baseline

M8 is completed and its audited baseline was released as `v0.2.0` on 2026-08-24. The exact release commit is preserved by the `v0.2.0` tag.

Compatible defects discovered in the released M8 baseline should normally be fixed in patch releases such as `v0.2.1`, `v0.2.2`, and so on when a named maintenance release is warranted. M9 development does not require opportunistic package-version bumps; its planned minor release is `v0.3.0` after M9 completion and release verification.

## Preserving named release commits

A named release is not complete until its exact Git commit is preserved by an immutable-intent SemVer tag such as `v0.2.0`.

Use the `Preserve named release tag` workflow only after the release candidate has passed its required verification and the intended release commit is known exactly. Supply both the SemVer tag and the full 40-character commit SHA. The workflow:

- accepts only `vMAJOR.MINOR.PATCH` tags and exact lowercase commit SHAs;
- verifies that the target commit exists in this repository;
- creates a lightweight `refs/tags/<version>` reference when none exists;
- succeeds without changing anything when the tag already resolves to the requested commit;
- fails closed if the release tag already exists at a different commit, rather than moving or rewriting a published release identity.

The workflow has a one-time push bootstrap for `v0.2.0` because the audited M8 release candidate was merged before repository-side tag preservation tooling existed. Future named releases, including the planned M9 `v0.3.0` release, should use the explicit manual workflow input path after their release PR is merged and verified.

Creating or preserving a release tag does not change `MODEL_SEMANTICS_ID`, package contents, or simulation semantics. It only preserves the exact source identity of the named software release.

## Agent and contributor rule

Agents and contributors must not opportunistically change the package version while implementing ordinary issues. A version bump should be an explicit release decision or part of a task that specifically calls for a named release.

From M9 onward, agents should plan on a completed major milestone normally culminating in the next minor release, while still treating milestone completion, release publication, model-semantics identity and Git source identity as separate concerns. When uncertain during implementation, leave the package version unchanged and preserve exact provenance through the Git commit identity.
