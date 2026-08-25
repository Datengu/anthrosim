# Security Policy

AnthroSim is research-oriented simulation software. Security reports are welcome, especially where a flaw could affect users running the CLI/explorer, compromise generated artifacts, or undermine the integrity or reproducibility of simulation results.

## Supported versions

Security fixes are made against the current `main` branch. Older commits, experimental branches, and historical artifact schemas are not independently supported unless a report also affects the current codebase.

## Dependency advisory policy

The `Dependency advisory audit` workflow runs the pinned `cargo-audit` 0.22.2 tool against the locked Rust dependency graph on dependency-changing pull requests, after dependency changes reach `main`, on a daily schedule, and when manually requested. The scheduled run matters because a dependency can become known-vulnerable after the commit that introduced it has already passed CI.

A RustSec vulnerability that makes `cargo audit` exit non-zero is an actionable failure and must be assessed before release or continued wider use. Informational categories such as unmaintained/unsound/notice advisories and yanked-crate warnings are still surfaced for review, but are not automatically treated as a reason to churn the dependency graph. If an advisory is ever explicitly ignored, the repository must record the advisory identifier, rationale, scope, and removal condition; there are no standing advisory ignores by default.

Dependency remediation remains a scientific-reproducibility change as well as a security change. Updating a crate to clear an advisory does not bypass the ordinary locked build, determinism, checkpoint/resume, artifact-integrity, and preserved reference/benchmark checks that apply to other code changes.

## Reporting a vulnerability

Please do **not** publish exploit details, credentials, private data, or a proof of concept in a public issue.

If GitHub offers **Report a vulnerability** on this repository's **Security** tab, use that private reporting channel. If private vulnerability reporting is unavailable, open a minimal public issue stating that you have a security concern and need a private contact channel, without including sensitive technical details.

A useful report should include, where possible:

- the affected commit/version and component;
- the impact you believe is possible;
- the conditions required to reproduce it;
- a minimal reproduction that does not expose secrets or unrelated private data;
- any suggested mitigation.

Security reports will be assessed for impact on confidentiality, integrity, availability, and scientific/reproducibility guarantees. Not every modelling defect is a security vulnerability; scientific or behavioural correctness issues that do not create a security impact should be reported through the ordinary issue tracker.

## Secrets and sensitive data

Do not commit API keys, credentials, private datasets, embargoed research material, or sensitive site/location data to this repository. Local `.env` files and ordinary generated run directories are excluded by `.gitignore`, but contributors remain responsible for checking changes before pushing them.
