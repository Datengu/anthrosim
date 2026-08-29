#!/usr/bin/env python3
"""Focused hardening regressions for research-analysis-provenance.py."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

BASE_TEST = Path(__file__).with_name("test-research-analysis-provenance.py")
spec = importlib.util.spec_from_file_location("analysis_provenance_base_tests", BASE_TEST)
if spec is None or spec.loader is None:
    raise RuntimeError(f"cannot load {BASE_TEST}")
base = importlib.util.module_from_spec(spec)
spec.loader.exec_module(base)


def test_scripted_run_rejects_preexisting_output() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-hardening-") as directory:
        root = Path(directory)
        definition_path = base.make_study(root)
        stale = root / "analysis/result.json"
        stale.write_text('{"schemaVersion":1,"scaledTotal":999}\n', encoding="utf-8")
        failed = base.run("run", root, definition_path, expect_success=False)
        assert "output already exists" in failed.stderr
        assert json.loads(stale.read_text(encoding="utf-8"))["scaledTotal"] == 999
        assert not (root / "analysis/analysis-provenance.json").exists()


def test_confirmatory_requires_environment_artifact() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-hardening-") as directory:
        root = Path(directory)
        definition_path = base.make_study(root)
        current = base.definition()
        current["environment"] = []
        definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        failed = base.run("run", root, definition_path, expect_success=False)
        assert "requires at least one environment artifact" in failed.stderr
        assert not (root / "analysis/result.json").exists()
        assert not (root / "analysis/analysis-provenance.json").exists()


def test_record_exposes_preexecution_binding_flag() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-hardening-") as directory:
        root = Path(directory)
        definition_path = base.make_study(root)
        base.run("run", root, definition_path)
        record = base.load_record(root)
        assert record["study"]["boundBeforeExecution"] is True
        base.run("verify", root)



def test_independent_area_k_argument_binding_audit() -> None:
    audit = Path(__file__).parent.parent / "docs/research/audit-v2/area-k-analysis-argument-binding-audit.py"
    result = __import__("subprocess").run([__import__("sys").executable, str(audit)], text=True, capture_output=True, check=False)
    assert result.returncode == 0, result.stderr
    assert "v2_executed_scale_3=15" in result.stdout

def main() -> None:
    tests = [
        test_scripted_run_rejects_preexisting_output,
        test_confirmatory_requires_environment_artifact,
        test_record_exposes_preexecution_binding_flag,
        test_independent_area_k_argument_binding_audit,
    ]
    for test in tests:
        test()
        print(f"ok: {test.__name__}")
    print(f"{len(tests)} analysis provenance hardening tests passed")


if __name__ == "__main__":
    main()
