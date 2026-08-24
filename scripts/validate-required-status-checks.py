#!/usr/bin/env python3
"""Validate the documented protected-main status-check contract against workflow YAML."""

from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
DOC = REPO_ROOT / "docs" / "required-status-checks.md"
WORKFLOWS = REPO_ROOT / ".github" / "workflows"

REQUIRED_CHECKS = (
    "Quality and tests",
    "Explorer and script validation",
    "Release build",
    "M5/M6 bundle integration",
    "Canonical M7.6 reference experiment",
    "Golden run (ubuntu-latest)",
    "Golden run (windows-latest)",
    "Golden run (macos-latest)",
    "Compare cross-platform golden runs",
    "M8.2 preprocessing validation",
    "Landscape golden run (ubuntu-latest)",
    "Landscape golden run (windows-latest)",
    "Landscape golden run (macos-latest)",
    "Compare landscape golden runs",
    "Spatial mechanism golden (ubuntu-latest)",
    "Spatial mechanism golden (windows-latest)",
    "Spatial mechanism golden (macos-latest)",
    "Compare transformed landscape golden runs",
    "Spatial M7 sweep integration",
    "Derive and inspect spatial observability",
    "Deterministic completed-run ZIP",
    "Automatic Git source identity",
    "New-directory resume Explorer compatibility",
)

LITERAL_JOB_NAMES = {
    "ci.yml": (
        "Quality and tests",
        "Explorer and script validation",
        "Release build",
        "M5/M6 bundle integration",
        "Canonical M7.6 reference experiment",
    ),
    "cross-platform-determinism.yml": ("Compare cross-platform golden runs",),
    "landscape-preprocessing.yml": ("M8.2 preprocessing validation",),
    "landscape-loading.yml": ("Compare landscape golden runs",),
    "spatial-mechanisms.yml": (
        "Compare transformed landscape golden runs",
        "Spatial M7 sweep integration",
    ),
    "spatial-observability.yml": ("Derive and inspect spatial observability",),
    "run-bundle-pack.yml": ("Deterministic completed-run ZIP",),
    "source-provenance.yml": ("Automatic Git source identity",),
    "resumed-explorer.yml": ("New-directory resume Explorer compatibility",),
}

MATRIX_JOBS = {
    "cross-platform-determinism.yml": "Golden run (${{ matrix.os }})",
    "landscape-loading.yml": "Landscape golden run (${{ matrix.os }})",
    "spatial-mechanisms.yml": "Spatial mechanism golden (${{ matrix.os }})",
}
MATRIX_OSES = ("ubuntu-latest", "windows-latest", "macos-latest")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def main() -> None:
    doc = DOC.read_text(encoding="utf-8")
    require(len(REQUIRED_CHECKS) == len(set(REQUIRED_CHECKS)), "duplicate required check name")
    for check in REQUIRED_CHECKS:
        require(f"`{check}`" in doc, f"required check is missing from documentation: {check}")

    for workflow_name, job_names in LITERAL_JOB_NAMES.items():
        text = (WORKFLOWS / workflow_name).read_text(encoding="utf-8")
        for job_name in job_names:
            require(
                f"name: {job_name}" in text,
                f"documented required check no longer matches {workflow_name}: {job_name}",
            )

    for workflow_name, matrix_name in MATRIX_JOBS.items():
        text = (WORKFLOWS / workflow_name).read_text(encoding="utf-8")
        require(
            f"name: {matrix_name}" in text,
            f"matrix required-check template changed in {workflow_name}: {matrix_name}",
        )
        for os_name in MATRIX_OSES:
            require(
                os_name in text,
                f"required matrix operating system {os_name} missing from {workflow_name}",
            )

    print(f"protected-main required status-check contract is coherent ({len(REQUIRED_CHECKS)} checks)")


if __name__ == "__main__":
    main()
