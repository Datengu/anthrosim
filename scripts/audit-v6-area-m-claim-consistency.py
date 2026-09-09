#!/usr/bin/env python3
"""Audit-v6 Area-M claim-consistency adversary for living scientific documentation.

This pass deliberately targets normative claims rather than creating another simulator attack.
It compares the current documented contracts with executable/repository-authoritative evidence
already preserved by Audit v6, while keeping empirical-readiness wording as a positive control.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LONG_RUN_DOC = ROOT / "docs" / "research" / "long-run-regime-diagnostics-v1.md"
PROVENANCE_DOC = ROOT / "docs" / "research" / "analysis-provenance-v2.md"
TRACE_DOC = ROOT / "docs" / "research" / "trace.md"
AREA_K = ROOT / "docs" / "research" / "audit-v6" / "area-k-2026-09-09.md"
AREA_L = ROOT / "docs" / "research" / "audit-v6" / "area-l-2026-09-09.md"
LONG_RUN_SCRIPT = ROOT / "scripts" / "research-long-run-diagnostics.py"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def drifting_run(initialization: str, terminal_mean: int) -> dict:
    return {
        "runId": f"{initialization}-{terminal_mean}",
        "pointId": "same-treatment",
        "seed": terminal_mean,
        "initialization": initialization,
        "environment": "default",
        "treatmentContext": "default",
        "terminalDay": 12 * 365,
        "stopReason": "durationReached",
        "primary": {
            "status": "drifting",
            "regimeSignature": None,
            "metrics": [
                {
                    "metricId": "living_population",
                    "stable": False,
                    "terminalWindowMeanFloor": terminal_mean,
                    "terminalCycleAmplitudePermille": 0,
                    "comparisons": [],
                }
            ],
        },
        "runLengthSensitivity": [],
        "analysisStartSensitivity": [],
        "analysisEndSensitivity": [],
    }


def main() -> int:
    long_run_doc = read(LONG_RUN_DOC)
    provenance_doc = read(PROVENANCE_DOC)
    trace_doc = read(TRACE_DOC)
    area_k = read(AREA_K)
    area_l = read(AREA_L)
    long_run = load_module(LONG_RUN_SCRIPT, "audit_v6_area_m_long_run")

    full_distribution_claim = (
        "Initialization/environment dependence compares the **full normalized outcome distributions**, "
        "not just which regime labels appear."
    )
    assert full_distribution_claim in long_run_doc

    runs = [
        drifting_run('founder_state="A"', 210),
        drifting_run('founder_state="A"', 210),
        drifting_run('founder_state="B"', 2100),
        drifting_run('founder_state="B"', 2100),
    ]
    frequencies = long_run.grouped_frequencies(runs, "initialization")
    detected = long_run.dependence_detected(frequencies)
    assert detected is False
    assert frequencies == {
        "default": {
            'founder_state="A"': {"status:drifting": 2},
            'founder_state="B"': {"status:drifting": 2},
        }
    }
    assert "drifting_initialization_dependence=false" in area_l
    assert "terminal_population_ratio=10.0" in area_l

    provenance_claim = (
        "Recomputing a new internally consistent `resultIdentity` after falsifying those authoritative "
        "artifacts therefore does not make the binding acceptable."
    )
    assert provenance_claim in provenance_doc
    assert "producer_finalize_rejects=true" in area_k
    assert "root_verifier_accepted=true" in area_k

    # Positive control: the top-level TRACE boundary must remain conservative even when internal
    # normative subcontracts are overstated or stale.
    trace_empirical_boundary = "**Overall scientific status:** **NOT YET EMPIRICALLY RESEARCH-READY**"
    assert trace_empirical_boundary in trace_doc
    assert "A green software build, deterministic replay, an ODD description, completed scientific audits or a completed benchmark are not sufficient evidence of empirical scientific validity." in trace_doc

    print(f"long_run_full_distribution_claim_present={str(full_distribution_claim in long_run_doc).lower()}")
    print(f"drifting_grouped_frequencies={frequencies}")
    print(f"drifting_initialization_dependence={str(detected).lower()}")
    print("drifting_terminal_mean_ratio=10.0")
    print(f"provenance_fresh_reidentity_rejection_claim_present={str(provenance_claim in provenance_doc).lower()}")
    print("producer_finalize_rejects=true")
    print("root_verifier_accepted=true")
    print("trace_empirical_boundary_conservative=true")
    print("area_m_claim_consistency_result=non_clean_via_existing_AV6_013_AV6_014")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
