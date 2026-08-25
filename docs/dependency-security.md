# Dependency security and advisory policy

AnthroSim treats dependency security as part of research-software integrity. A dependency graph that once passed deterministic and scientific regression tests can later become known-vulnerable without any AnthroSim source change, so the locked graph is checked against the current RustSec Advisory Database independently of ordinary build/test CI.

## Automated RustSec check

`.github/workflows/dependency-audit.yml` runs `cargo-audit` against the committed root `Cargo.lock`:

- on pull requests that change Cargo manifests, `Cargo.lock`, the audit policy, or the audit workflow;
- once per day on the default branch, so newly disclosed advisories are surfaced without a source-code change;
- on manual dispatch when an explicit recheck is useful.

The workflow uses the repository's pinned Rust 1.97.1 toolchain and installs exactly `cargo-audit 0.22.2` with `--locked`. The RustSec advisory database is intentionally fetched at run time rather than pinned: freshness is the purpose of the scheduled control, and a stale database is not accepted.

## Enforcement policy

The checked-in `.cargo/audit.toml` defines the machine-readable policy.

- **RustSec vulnerability advisory:** hard failure. Investigate applicability and remediation before accepting a dependency-changing pull request or a release candidate.
- **RustSec unsound advisory:** hard failure. Unsoundness can undermine memory safety, deterministic execution, or confidence in generated research artifacts even when it is classified as informational rather than a CVE-style vulnerability.
- **Unmaintained advisory:** visible warning, not an automatic failure. Maintenance status is a risk signal, but replacing a dependency solely to silence the warning can itself change deterministic behaviour or scientific reproducibility.
- **Yanked crate:** visible warning, not an automatic failure. A yanked version already locked by the project is assessed for the reason it was yanked and for available replacements; yanking alone does not authorize blind lockfile churn.

An advisory ignore must never be added merely to make CI green. Any future ignore must identify the exact advisory, have a documented applicability/risk rationale in the associated pull request or issue, and be revisited when the dependency graph or advisory changes.

## Remediation and reproducibility

Security remediation does not bypass AnthroSim's scientific regression requirements. When a vulnerable, unsound, unmaintained, or yanked dependency is changed:

1. prefer the smallest dependency/lockfile change that resolves the assessed risk;
2. record why the change is necessary and any expected behaviour/determinism impact;
3. run the ordinary workspace, provenance, bundle, deterministic and relevant scientific reference checks;
4. do not rebaseline a deterministic or scientific reference merely because a dependency update changed output—investigate the change first;
5. for a release candidate, explicitly assess any advisory warnings that remain even when the RustSec job is green.

This control complements GitHub-native dependency alerts or update tooling if those are enabled; it does not rely on them as the sole dependency-security mechanism.
