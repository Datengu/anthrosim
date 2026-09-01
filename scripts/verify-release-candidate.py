#!/usr/bin/env python3
"""Fail-closed verification for AnthroSim named release candidates."""

from __future__ import annotations

import argparse
import json
import re
import tempfile
import tomllib
from pathlib import Path

SEMVER_TAG = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
RELEASE_GATES_FROM = (0, 3, 0)
RELEASE_SPECIFIC_GATES = (
    "Execute predeclared terrain null-model benchmark",
    "Execute predeclared M9.7 aggregation benchmark",
    "RustSec dependency audit",
)
WORKSPACE_PACKAGES = ("anthrosim-cli", "anthrosim-core")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def parse_tag(tag: str) -> tuple[str, tuple[int, int, int]]:
    match = SEMVER_TAG.fullmatch(tag)
    require(match is not None, f"release tag must match vMAJOR.MINOR.PATCH: {tag}")
    assert match is not None
    return tag[1:], tuple(int(part) for part in match.groups())


def citation_version(path: Path) -> str:
    values: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("version:"):
            value = line.split(":", 1)[1].strip().strip('"\'')
            if value:
                values.append(value)
    require(len(values) == 1, "CITATION.cff must contain exactly one top-level version field")
    return values[0]


def lock_workspace_versions(path: Path) -> dict[str, str]:
    lock = tomllib.loads(path.read_text(encoding="utf-8"))
    packages = lock.get("package")
    require(isinstance(packages, list), "Cargo.lock is missing package entries")
    versions: dict[str, str] = {}
    for package in packages:
        if not isinstance(package, dict):
            continue
        name = package.get("name")
        version = package.get("version")
        if name in WORKSPACE_PACKAGES and isinstance(version, str):
            require(name not in versions, f"Cargo.lock contains duplicate workspace package entry: {name}")
            versions[name] = version
    return versions


def latest_check_runs(checks: dict) -> dict[str, dict]:
    runs = checks.get("check_runs")
    require(isinstance(runs, list), "check-runs payload is missing check_runs")
    total = checks.get("total_count", len(runs))
    require(total == len(runs), "check-runs response was truncated; fetch every check run before verification")
    latest: dict[str, dict] = {}
    for run in runs:
        name = run.get("name")
        run_id = run.get("id")
        if not isinstance(name, str) or not isinstance(run_id, int):
            continue
        previous = latest.get(name)
        if previous is None or run_id > previous.get("id", -1):
            latest[name] = run
    return latest


def latest_statuses(status_payload: dict) -> dict[str, dict]:
    statuses = status_payload.get("statuses")
    require(isinstance(statuses, list), "combined-status payload is missing statuses")
    latest: dict[str, dict] = {}
    for status in statuses:
        context = status.get("context")
        status_id = status.get("id")
        if not isinstance(context, str) or not isinstance(status_id, int):
            continue
        previous = latest.get(context)
        if previous is None or status_id > previous.get("id", -1):
            latest[context] = status
    return latest


def context_succeeded(name: str, check_runs: dict[str, dict], statuses: dict[str, dict]) -> bool:
    run = check_runs.get(name)
    if run is not None:
        return run.get("status") == "completed" and run.get("conclusion") == "success"
    status = statuses.get(name)
    if status is not None:
        return status.get("state") == "success"
    return False


def verify(
    *,
    tag: str,
    sha: str,
    repo_root: Path,
    branch_payload: dict,
    checks_payload: dict,
    status_payload: dict,
) -> list[str]:
    version, version_tuple = parse_tag(tag)
    require(re.fullmatch(r"[0-9a-f]{40}", sha) is not None, "release SHA must be 40 lowercase hex characters")

    cargo = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    workspace_version = cargo.get("workspace", {}).get("package", {}).get("version")
    require(workspace_version == version, f"Cargo workspace version {workspace_version!r} does not match {tag}")

    lock_versions = lock_workspace_versions(repo_root / "Cargo.lock")
    for package in WORKSPACE_PACKAGES:
        require(
            lock_versions.get(package) == version,
            f"Cargo.lock workspace package {package} version {lock_versions.get(package)!r} does not match {tag}",
        )

    cff_version = citation_version(repo_root / "CITATION.cff")
    require(cff_version == version, f"CITATION.cff version {cff_version!r} does not match {tag}")

    release_notes = repo_root / "docs" / "releases" / f"{tag}.md"
    require(release_notes.is_file(), f"release notes are missing: {release_notes.relative_to(repo_root)}")
    first_line = release_notes.read_text(encoding="utf-8").splitlines()[0]
    require(first_line == f"# AnthroSim {tag}", f"release notes title does not identify {tag}: {first_line!r}")

    main_sha = branch_payload.get("commit", {}).get("sha")
    require(main_sha == sha, f"release candidate {sha} is not the current protected main HEAD ({main_sha})")
    protection = branch_payload.get("protection")
    require(isinstance(protection, dict) and protection.get("enabled") is True, "main is not reported as protected")
    required_contexts = protection.get("required_status_checks", {}).get("contexts")
    require(isinstance(required_contexts, list) and required_contexts, "protected main reports no required status contexts")
    require(len(required_contexts) == len(set(required_contexts)), "protected main reports duplicate required status contexts")

    check_runs = latest_check_runs(checks_payload)
    statuses = latest_statuses(status_payload)
    missing = [name for name in required_contexts if not context_succeeded(name, check_runs, statuses)]
    require(not missing, "required protected-main checks are not successful on the exact release SHA: " + ", ".join(missing))

    release_gates: list[str] = []
    if version_tuple >= RELEASE_GATES_FROM:
        release_gates = list(RELEASE_SPECIFIC_GATES)
        missing_release = [name for name in release_gates if not context_succeeded(name, check_runs, statuses)]
        require(
            not missing_release,
            "release-specific M8/M9/security gates are not successful on the exact release SHA: "
            + ", ".join(missing_release),
        )

    return list(required_contexts) + release_gates


def _fixture(root: Path) -> None:
    (root / "docs" / "releases").mkdir(parents=True, exist_ok=True)
    (root / "Cargo.toml").write_text('[workspace]\n[workspace.package]\nversion = "0.4.0"\n', encoding="utf-8")
    (root / "Cargo.lock").write_text(
        'version = 4\n\n'
        '[[package]]\nname = "anthrosim-cli"\nversion = "0.4.0"\n\n'
        '[[package]]\nname = "anthrosim-core"\nversion = "0.4.0"\n',
        encoding="utf-8",
    )
    (root / "CITATION.cff").write_text('cff-version: 1.2.0\nversion: "0.4.0"\n', encoding="utf-8")
    (root / "docs" / "releases" / "v0.4.0.md").write_text("# AnthroSim v0.4.0\n", encoding="utf-8")


def self_test() -> None:
    sha = "a" * 40
    required = ["Quality and tests", "Applicable scientific/security gates"]
    branch = {
        "commit": {"sha": sha},
        "protection": {"enabled": True, "required_status_checks": {"contexts": required}},
    }
    all_names = required + list(RELEASE_SPECIFIC_GATES)
    checks = {
        "total_count": len(all_names),
        "check_runs": [
            {"id": index + 1, "name": name, "status": "completed", "conclusion": "success"}
            for index, name in enumerate(all_names)
        ],
    }
    status = {"statuses": []}
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        _fixture(root)
        verified = verify(
            tag="v0.4.0",
            sha=sha,
            repo_root=root,
            branch_payload=branch,
            checks_payload=checks,
            status_payload=status,
        )
        require(set(verified) == set(all_names), "positive verifier fixture omitted a gate")

        bad_checks = json.loads(json.dumps(checks))
        bad_checks["check_runs"][-1]["conclusion"] = "failure"
        try:
            verify(
                tag="v0.4.0",
                sha=sha,
                repo_root=root,
                branch_payload=branch,
                checks_payload=bad_checks,
                status_payload=status,
            )
        except ValueError as error:
            require("release-specific" in str(error), "failed release gate produced wrong diagnosis")
        else:
            raise ValueError("failed release-specific check was accepted")

        bad_lock = root / "Cargo.lock"
        bad_lock.write_text(
            'version = 4\n\n'
            '[[package]]\nname = "anthrosim-cli"\nversion = "0.4.0"\n\n'
            '[[package]]\nname = "anthrosim-core"\nversion = "0.3.9"\n',
            encoding="utf-8",
        )
        try:
            verify(
                tag="v0.4.0",
                sha=sha,
                repo_root=root,
                branch_payload=branch,
                checks_payload=checks,
                status_payload=status,
            )
        except ValueError as error:
            require("Cargo.lock workspace package anthrosim-core" in str(error), "lock mismatch produced wrong diagnosis")
        else:
            raise ValueError("Cargo.lock workspace version mismatch was accepted")
        _fixture(root)

        try:
            verify(
                tag="v0.4.1",
                sha=sha,
                repo_root=root,
                branch_payload=branch,
                checks_payload=checks,
                status_payload=status,
            )
        except ValueError as error:
            require("Cargo workspace version" in str(error), "version mismatch produced wrong diagnosis")
        else:
            raise ValueError("version mismatch was accepted")

        wrong_branch = json.loads(json.dumps(branch))
        wrong_branch["commit"]["sha"] = "b" * 40
        try:
            verify(
                tag="v0.4.0",
                sha=sha,
                repo_root=root,
                branch_payload=wrong_branch,
                checks_payload=checks,
                status_payload=status,
            )
        except ValueError as error:
            require("current protected main HEAD" in str(error), "non-HEAD candidate produced wrong diagnosis")
        else:
            raise ValueError("non-main-HEAD release candidate was accepted")

    print("release-candidate verifier self-test passed")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--tag")
    parser.add_argument("--sha")
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--branch-json", type=Path)
    parser.add_argument("--checks-json", type=Path)
    parser.add_argument("--status-json", type=Path)
    args = parser.parse_args()

    if args.self_test:
        self_test()
        return

    for name in ("tag", "sha", "branch_json", "checks_json", "status_json"):
        require(getattr(args, name) is not None, f"--{name.replace('_', '-')} is required")

    verified = verify(
        tag=args.tag,
        sha=args.sha,
        repo_root=args.repo_root,
        branch_payload=json.loads(args.branch_json.read_text(encoding="utf-8")),
        checks_payload=json.loads(args.checks_json.read_text(encoding="utf-8")),
        status_payload=json.loads(args.status_json.read_text(encoding="utf-8")),
    )
    print(f"release candidate verified: {args.tag} -> {args.sha}")
    for name in verified:
        print(f"  PASS {name}")


if __name__ == "__main__":
    main()
