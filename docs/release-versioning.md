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
- **Minor releases** (`0.X.0`) are for meaningful, deliberately named capability releases. Completion and verification of a major roadmap milestone is a natural, but not mandatory, point for a minor release.
- **Major release `1.0.0`** is reserved for a separately defined stable/research-ready compatibility baseline. Reaching a particular milestone number does not automatically imply `1.0.0`.

A milestone number and software version are intentionally independent. A valid description may therefore look like `AnthroSim v0.2.1 — M8`.

## Scientific changes are separate from release numbering

A software version bump does not by itself say that scientific meaning changed. Changes that alter authoritative simulation meaning must also follow the model-semantics compatibility policy and update `MODEL_SEMANTICS_ID` when checkpoint continuation or interpretation would become scientifically incompatible.

Documentation, packaging, explorer-only or other source-neutral changes may warrant a software release without changing model semantics. Conversely, a scientifically meaningful change must never be hidden behind an unchanged semantics identity merely because the package version is unchanged.

## Milestone completion versus hardening

A completed roadmap milestone is not automatically reopened because a later audit discovers defects.

Use this distinction:

- a defect in functionality that was implemented for the milestone is **post-milestone hardening/remediation**;
- functionality required by the milestone acceptance criteria that was never actually implemented means the milestone was **not fully complete** and its status should be corrected.

Audit findings that affect correctness, determinism, reproducibility, provenance, data integrity, or stated milestone acceptance criteria should be resolved before cutting the named release intended to represent that milestone as a stable baseline.

## M8 release rule

The roadmap records M8 as completed. Current audit/remediation work should therefore be treated as post-M8 hardening unless a finding demonstrates that required M8 functionality was absent.

**Do not cut or label `v0.2.0` merely because M8 implementation is marked complete.** `v0.2.0` is reserved for the audited and reverified M8 baseline after material post-M8 audit findings have been resolved and the relevant acceptance/reproducibility checks pass.

After `v0.2.0` is released, compatible defects discovered in that released baseline should normally be fixed in patch releases such as `v0.2.1`, `v0.2.2`, and so on. A later substantial capability release may become `v0.3.0` even if it does not map one-to-one to a roadmap milestone.

## Agent and contributor rule

Agents and contributors must not opportunistically change the package version while implementing ordinary issues. A version bump should be an explicit release decision or part of a task that specifically calls for a named release. When uncertain, leave the package version unchanged and preserve exact provenance through the Git commit identity.
