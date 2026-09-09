# AnthroSim agent guidance

This file contains standing instructions for coding/research agents working in this repository.

Before making substantive changes, read:

- `CONTRIBUTING.md`;
- `docs/roadmap.md`;
- `docs/release-versioning.md`;
- the relevant scientific/research documentation for the subsystem being changed.

## Milestones and releases

- `M#` labels are roadmap/capability milestones, not software versions.
- Do **not** bump the AnthroSim package version for every commit, issue or pull request. Exact code identity is already preserved by run Git provenance.
- Version bumps are explicit release decisions. Follow `docs/release-versioning.md`.
- M8 is completed and its audited baseline is released and preserved as `v0.2.0`.
- M9 is completed, audited and released as the preserved temporary-mobility / controlled-aggregation baseline `v0.3.0`.
- The later `v0.3.1`–`v0.3.6` releases are post-M9 scientific-hardening, documentation-convergence and audit-repair baselines. `v0.3.5` is the immutable Audit-v5 target at model semantics v33; `v0.3.6` freezes the fully remediated Audit-v5 line at model semantics v35. Audit v5 was a non-clean convergence pass because it discovered four P1 findings, so agents must not begin the first empirical/site-specific study until a fresh full audit of the newly frozen baseline achieves the project convergence gate. Current protected `main` may later advance scientifically beyond the latest named package version; derive current semantics from executable provenance and living documentation rather than assuming the latest release tag has moved.
- From M9 onward, a completed major roadmap milestone should normally culminate in the next minor software release, but milestone identity and release identity remain independent.
- There is no standing feature list that agents should invent as “M10”. New substantive capability work should follow the question-led strategy in `docs/roadmap.md` and the current repository state.
- During ordinary milestone implementation, leave the package version unchanged unless the task explicitly concerns release preparation.
- After a named release, compatible bug-fix releases may increment the patch version without creating a new milestone.
- A change to authoritative scientific/model meaning may require a `MODEL_SEMANTICS_ID` change independently of the package version. Review this explicitly for milestone release candidates.

## Research integrity

Preserve deterministic behaviour, source/evidence provenance, explicit assumptions, schema compatibility rules, and the separation between authoritative simulation state and downstream visualisation/analysis. Do not tune the model toward a desired historical outcome.
