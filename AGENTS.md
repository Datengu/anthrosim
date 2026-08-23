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
- M8 is recorded as completed. Subsequent audit fixes are post-M8 hardening unless a finding proves that required M8 functionality was never implemented.
- `v0.2.0` is reserved for the audited and reverified M8 baseline. Do not cut or label `v0.2.0` until material post-M8 audit findings are resolved and the relevant checks pass.
- After a named release, compatible bug-fix releases may increment the patch version.
- A change to authoritative scientific/model meaning may require a `MODEL_SEMANTICS_ID` change independently of the package version.

## Research integrity

Preserve deterministic behaviour, source/evidence provenance, explicit assumptions, schema compatibility rules, and the separation between authoritative simulation state and downstream visualisation/analysis. Do not tune the model toward a desired historical outcome.
