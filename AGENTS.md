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
- M9 is the planned temporary-mobility and aggregation capability milestone. Its target named release is `v0.3.0` after M9 completion, audit/hardening and release verification.
- From M9 onward, a completed major roadmap milestone should normally culminate in the next minor software release, but milestone identity and release identity remain independent.
- During ordinary milestone implementation, leave the package version unchanged unless the task explicitly concerns release preparation.
- After a named release, compatible bug-fix releases may increment the patch version without creating a new milestone.
- A change to authoritative scientific/model meaning may require a `MODEL_SEMANTICS_ID` change independently of the package version. Review this explicitly for milestone release candidates.

## Research integrity

Preserve deterministic behaviour, source/evidence provenance, explicit assumptions, schema compatibility rules, and the separation between authoritative simulation state and downstream visualisation/analysis. Do not tune the model toward a desired historical outcome.
