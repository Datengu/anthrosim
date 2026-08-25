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

The three platform-specific matrix jobs are required individually as well as their comparison jobs. Requiring only a downstream comparison is insufficient because a failed matrix dependency can leave a comparison job skipped/neutral rather than proving that every platform succeeded.

## Branch-rule settings

The intended `main` protection also requires:

- pull requests before merge;
- required checks to pass before merge;
- the branch to be up to date before merge;
- conversation resolution before merge;
- **no administrator bypass of the scientific/integrity requirements under the normal merge flow**.

If GitHub displays a setting such as **Do not allow bypassing the above settings**, **Include administrators**, or equivalent wording, it must be configured so repository administrators cannot merge while the required checks are pending/failing.

## Checks deliberately not globally required

The following jobs are useful but are not part of the global protected set:

- `Core benchmarks`;
- `1000-run ensemble soak`;
- `Performance and memory acceptance`;
- `Regenerate pinned open terrain input`;
- `Execute predeclared terrain null-model benchmark`;
- `Execute predeclared M9.7 aggregation benchmark`;
- `RustSec dependency audit`.

The first three are scale/performance gates whose failure is still taken seriously during release work but which are not necessary as everyday branch-protection contexts. The M8.6 and M9.7 benchmark jobs are path-filtered: globally requiring a path-filtered check can block unrelated pull requests waiting for a context that never runs.

`RustSec dependency audit` is likewise deliberately not globally required because its pull-request trigger is restricted to Cargo dependency state and the audit workflow itself, while its daily scheduled run detects advisories disclosed after merge. Dependency-changing pull requests must treat a failed audit as blocking even though unrelated pull requests do not receive that status context.

Named releases may impose a stronger release checklist than ordinary branch protection. In particular, `v0.2.0` required an explicit M8.6 canonical scientific regression run even though that path-filtered job was not globally required. The planned M9 `v0.3.0` release must likewise explicitly rerun the preserved M9.7 scientific regression benchmark as part of release verification rather than making its path-filtered context globally mandatory.

## Maintenance rule

Any pull request that adds, removes or renames an independent workflow/job protecting correctness, determinism, provenance, artifact integrity or research reproducibility must explicitly answer:

> Should this check be added to, removed from, or renamed in the protected `main` required-check set?

The repository test `required_status_checks_contract` verifies that the exact names above still correspond to workflow job definitions and matrix operating systems. It cannot mutate or query private repository administration settings from CI, so the live GitHub rule must still be verified after administrative changes.

A workflow rename must therefore update this document/test in the same reviewed change, and the live branch-protection setting must be reconciled before that change is considered governance-complete.
