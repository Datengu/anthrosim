# M9 implementation sequence

**Status:** historical implementation plan — M9 completed, audited and released as `v0.3.0`  
**Semantics contract:** `docs/research/temporary-mobility-v1.md`

This document preserves the reviewable implementation sequence used to build M9. It is not the current project-status authority; use `docs/roadmap.md` and `docs/release-versioning.md` for the living development/release state.

M9 was implemented as reviewable slices. The issue sequence was intentionally capability-led and was not to be expanded with unrelated anthropological features.

1. **M9.0 — #114:** freeze temporary-mobility semantics and acceptance contract.
2. **M9.1 — #115:** separate persistent residence from temporary physical presence.
3. **M9.2 — #116:** add generic identity-bearing focal-region bindings.
4. **M9.3 — #117:** implement deterministic temporary-journey lifecycle and scheduling.
5. **M9.4 — #118:** add deterministic focal-region travel cost and duration semantics.
6. **M9.5 — #119:** make resource accounting duration-aware during temporary mobility.
7. **M9.6 — #120:** extend events, checkpoints, observability and experiment machinery.
8. **M9.7 — #121:** run the controlled continuous-residence versus intermittent-aggregation benchmark.

## Dependency rule

M9.0 is the semantic authority for later slices. Later PRs must not silently contradict it. If implementation evidence shows the contract is impossible or scientifically unsound, amend the contract explicitly in a reviewable PR rather than hiding a different assumption in code.

M9.1 and M9.2 established the state/input boundaries. M9.3 and M9.4 could be developed closely together but were intended to remain independently reviewable where possible. M9.5 depended on actual journey intervals. M9.6 integrated the completed mechanism into the existing experiment/provenance/analysis stack. M9.7 was the milestone acceptance benchmark and was not to be used to tune the model toward a preferred archaeological result.

## Release outcome

Ordinary M9 implementation remained on the then-current package release line. After M9.7 and milestone completion, AnthroSim entered audit/hardening and release verification and M9 was subsequently preserved as the `v0.3.0` release according to `docs/release-versioning.md`. Later patch releases and post-release model-semantics changes do not rewrite this historical implementation sequence.
