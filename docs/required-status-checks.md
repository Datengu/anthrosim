# Required `main` status-check contract

AnthroSim's `main` branch is protected by GitHub. This file records the intended repository-governance contract for the checks that must pass before ordinary merges.

This contract is separate from the complete CI topology in `docs/ci-validation.md`: some workflows are valuable but deliberately not globally required because they are path-filtered or performance-oriented. The list below is the exact set intended for branch protection.

## Required GitHub Actions checks

1. `Quality and tests`
2. `Explorer and script validation`
3. `Release build`
4. `M5/M6 bundle integration`
5. `Canonical M7.6 reference experiment`
6. `Golden run (ubuntu-latest)`
7. `Golden run (windows-latest)`
8. `Golden run (macos-latest)`
9. `Compare cross-platform golden runs`
10. `M8.2 preprocessing validation`
11. `Landscape golden run (ubuntu-latest)`
12. `Landscape golden run (windows-latest)`
13. `Landscape golden run (macos-latest)`
14. `Compare landscape golden runs`
15. `Spatial mechanism golden (ubuntu-latest)`
16. `Spatial mechanism golden (windows-latest)`
17. `Spatial mechanism golden (macos-latest)`
18. `Compare transformed landscape golden runs`
19. `Spatial M7 sweep integration`
20. `Derive and inspect spatial observability`
21. `Deterministic completed-run ZIP`
22. `Automatic Git source identity`
23. `New-directory resume Explorer compatibility`
24. `Applicable scientific/security gates`

The three platform-specific matrix jobs are required individually as well as their comparison jobs. Requiring only a downstream comparison is insufficient because a failed matrix dependency can leave a comparison job skipped/neutral rather than proving that every platform succeeded.

## Branch-rule settings

The intended `main` protection also requires:

- pull requests before merge;
- required checks to pass before merge;
- the branch to be up to date before merge;
- conversation resolution before merge;
- **no administrator bypass of the scientific/integrity requirements under the normal merge flow**.

If GitHub displays a setting such as **Do not allow bypassing the above settings**, **Include administrators**, or equivalent wording, it must be configured so repository administrators cannot merge while the required checks are pending/failing.

## Applicable path-dependent scientific/security gates

`M8.6`, `M9.7` and `RustSec` remain conditional work, but their **disposition is no longer human-only**. The always-present `Applicable scientific/security gates` context is the protected-main enforcement point.

For every pull request it obtains the complete changed-file set from the GitHub API, verifies the retrieved file-object count matches GitHub's pull-request metadata, includes both the current and previous path for renamed files, applies the repository's versioned path classification, and then:

- invokes `Execute predeclared terrain null-model benchmark` when M8.6-relevant files changed;
- invokes `Execute predeclared M9.7 aggregation benchmark` when M9.7-relevant files changed;
- invokes `RustSec dependency audit` when dependency state changed;
- records an explicit `N/A` for each gate that is not applicable.

A relevant gate that fails, is cancelled or otherwise does not complete successfully makes the always-present context fail. Classification failure/ambiguity also fails closed. Changes to the classifier or aggregator themselves force **all three** conditional gates so the enforcement policy cannot weaken its own review surface.

The three expensive underlying job names are intentionally **not** required status contexts themselves. They may be absent/skipped when not applicable; branch protection requires the aggregator, whose own successful result proves either `PASS` or explicit `N/A` for each one.

The aggregator also emits a lightweight success on every push to protected `main`. This gives the merged commit a continuity context without rerunning all expensive conditional gates. A named release still requires release-specific exact-SHA M8.6/M9.7/RustSec evidence under the release policy; the post-merge continuity result does not substitute for those release reruns.

After the aggregator is merged, the live GitHub `main` branch protection must be updated to require `Applicable scientific/security gates`. Until that administrative reconciliation is complete, issue #175 is not governance-complete even though the repository-side workflow exists.

## Checks deliberately not globally required

The following scale/performance jobs remain useful but are not part of the global protected set:

- `Core benchmarks`;
- `1000-run ensemble soak`;
- `Performance and memory acceptance`;
- `Regenerate pinned open terrain input`.

The first three are scale/performance gates whose failure is still taken seriously during release work but which are not necessary as everyday branch-protection contexts. `Regenerate pinned open terrain input` is a data-maintenance path rather than an ordinary merge gate.

Named releases may impose a stronger release checklist than ordinary branch protection. The release-tag workflow separately verifies the exact release candidate and its release-specific scientific/security dispositions before a missing SemVer tag can be created.

## Maintenance rule

Any pull request that adds, removes or renames an independent workflow/job protecting correctness, determinism, provenance, artifact integrity or research reproducibility must explicitly answer:

> Should this check be added to, removed from, or renamed in the protected `main` required-check set?

The repository test `required_status_checks_contract` verifies that the exact names above still correspond to workflow job definitions and matrix operating systems. It cannot mutate or query private repository administration settings from CI, so the live GitHub rule must still be verified after administrative changes.

A workflow rename must therefore update this document/test in the same reviewed change, and the live branch-protection setting must be reconciled before that change is considered governance-complete.
