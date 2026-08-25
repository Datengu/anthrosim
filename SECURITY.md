# Security Policy

AnthroSim is research-oriented simulation software. Security reports are welcome, especially where a flaw could affect users running the CLI/explorer, compromise generated artifacts, or undermine the integrity or reproducibility of simulation results.

## Supported versions

Security fixes are made against the current `main` branch. Older commits, experimental branches, and historical artifact schemas are not independently supported unless a report also affects the current codebase.

## Dependency advisories

The committed Rust dependency graph is checked automatically against the current RustSec Advisory Database when dependency state changes and on a daily schedule. Applicable vulnerability and unsoundness advisories are treated as failing security/integrity conditions; unmaintained and yanked dependencies remain visible for explicit assessment rather than triggering blind dependency churn.

The complete machine-readable policy, scheduled-control rationale, advisory-ignore rule and reproducibility requirements for dependency remediation are documented in [`docs/dependency-security.md`](docs/dependency-security.md). A security-driven dependency update remains subject to AnthroSim's ordinary deterministic and scientific regression checks.

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