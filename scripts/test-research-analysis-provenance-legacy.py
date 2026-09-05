#!/usr/bin/env python3
"""Regression tests for research-analysis-provenance.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-analysis-provenance.py")
INTEGRITY = Path(__file__).with_name("research-integrity.py")


def run(*args: object, expect_success: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), *(str(arg) for arg in args)],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if expect_success and result.returncode != 0:
        raise AssertionError(
            f"command failed: {args}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    if not expect_success and result.returncode == 0:
        raise AssertionError(f"command unexpectedly succeeded: {args}")
    return result


def binding(*, eligible: bool = True, status: str = "confirmatory") -> dict:
    return {
        "schemaVersion": 1,
        "resultIdentity": "study-result-v1-deadbeef",
        "studyExecutionId": "study-execution-v1-cafebabe",
        "protocolIdentity": "study-protocol-v1-sha256-" + "1" * 64,
        "protocolRevision": 1,
        "studyId": "synthetic-analysis-provenance",
        "scientificStatus": status,
        "boundBeforeExecution": True,
        "confirmatoryPreResultClaimEligible": eligible,
        "definitionIdentity": "research-definition-v1-" + "2" * 16,
        "researchId": "research-execution-v1-" + "3" * 16,
        "source": {
            "modelVersion": "0.3.0",
            "modelSemanticsId": "anthrosim-model-semantics-v14",
            "gitCommit": "synthetic-fixture",
        },
        "researchRelativeDir": "research",
        "runCounts": {"completed": 2, "failed": 0},
        "resultArtifacts": [],
    }


ANALYSIS_PROGRAM = r'''import argparse, json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--input", required=True)
parser.add_argument("--output", required=True)
scale_source = parser.add_mutually_exclusive_group(required=True)
scale_source.add_argument("--scale", type=int)
scale_source.add_argument("--config")
parser.add_argument("--mutate-input", action="store_true")
args = parser.parse_args()

source = Path(args.input)
rows = json.loads(source.read_text(encoding="utf-8"))["runs"]
scale = args.scale
if args.config is not None:
    scale = int(json.loads(Path(args.config).read_text(encoding="utf-8"))["scale"])
value = sum(row["value"] for row in rows) * scale
Path(args.output).write_text(
    json.dumps({"schemaVersion": 1, "scaledTotal": value}, sort_keys=True) + "\n",
    encoding="utf-8",
)
if args.mutate_input:
    source.write_text(source.read_text(encoding="utf-8") + "\n", encoding="utf-8")
'''


def definition(*, mode: str = "scripted", mutate_input: bool = False) -> dict:
    command = []
    manual_steps: list[str] = []
    if mode == "scripted":
        command = [
            sys.executable,
            "analysis/analyze.py",
            "--input",
            "research/analysis/runs.json",
            "--output",
            "analysis/result.json",
            "--scale",
            "2",
        ]
        if mutate_input:
            command.append("--mutate-input")
    else:
        manual_steps = [
            "Imported research/analysis/runs.json into an external analysis tool.",
            "Exported the canonical result to analysis/result.json without further edits.",
        ]
    return {
        "schemaVersion": 2,
        "definitionType": "anthrosim-analysis-definition",
        "analysisId": "synthetic-total-v1",
        "analysisStatus": "confirmatory",
        "executionMode": mode,
        "workingDirectory": ".",
        "command": command,
        "annotations": {
            "estimand": "scaled total",
            "filter": "all completed synthetic rows",
        },
        "runtimeDescription": "Python standard library; fixture lock file records the test environment.",
        "reproductionCriterion": "exact_output_bytes",
        "inputs": [
            {"path": "research/analysis/runs.json", "role": "immutable-derived-run-table"}
        ],
        "implementation": [
            {"path": "analysis/analyze.py", "role": "canonical-analysis-script"}
        ],
        "environment": [
            {"path": "analysis/environment.lock", "role": "analysis-environment-lock"}
        ],
        "outputs": [
            {"path": "analysis/result.json", "role": "canonical-machine-readable-result"}
        ],
        "manualSteps": manual_steps,
    }


def make_study(root: Path, *, eligible: bool = True, status: str = "confirmatory") -> Path:
    (root / "research/analysis").mkdir(parents=True)
    (root / "analysis").mkdir(parents=True)
    current_binding = binding(eligible=eligible, status=status)
    (root / "study-result-binding.json").write_text(
        json.dumps(current_binding, indent=2) + "\n",
        encoding="utf-8",
    )
    (root / "research/analysis/runs.json").write_text(
        json.dumps(
            {
                "schemaVersion": 1,
                "researchId": current_binding["researchId"],
                "runs": [{"runId": "a", "value": 2}, {"runId": "b", "value": 3}],
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    (root / "analysis/analyze.py").write_text(ANALYSIS_PROGRAM, encoding="utf-8")
    (root / "analysis/environment.lock").write_text(
        "python=standard-library\n", encoding="utf-8"
    )
    definition_path = root / "analysis-definition.json"
    definition_path.write_text(json.dumps(definition(), indent=2) + "\n", encoding="utf-8")
    return definition_path


def load_record(root: Path) -> dict:
    return json.loads((root / "analysis/analysis-provenance.json").read_text(encoding="utf-8"))


def test_scripted_run_and_verify() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        result = run("run", root, definition_path)
        identity = result.stdout.strip()
        assert identity.startswith("analysis-provenance-v2-sha256-")
        assert json.loads((root / "analysis/result.json").read_text())["scaledTotal"] == 10
        record = load_record(root)
        assert record["provenanceIdentity"] == identity
        assert "analysisRngSeeds" not in record["definition"]
        assert record["executionStatus"] == "executed_by_wrapper"
        assert record["study"]["protocolIdentity"] == binding()["protocolIdentity"]
        assert record["study"]["artifact"]["sha256"]
        assert record["artifacts"]["inputs"][0]["sha256"]
        assert record["artifacts"]["implementation"][0]["sha256"]
        assert record["artifacts"]["environment"][0]["sha256"]
        assert record["artifacts"]["outputs"][0]["sha256"]
        verified = run("verify", root)
        assert verified.stdout.strip() == identity

        second = run("run", root, definition_path, expect_success=False)
        assert "already exists" in second.stderr


def test_replay_reproduces_exact_output_bytes() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        identity = run("run", root, definition_path).stdout.strip()
        replayed = run("replay", root)
        assert replayed.stdout.strip() == identity


def test_output_tamper_is_rejected() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        run("run", root, definition_path)
        (root / "analysis/result.json").write_text('{"scaledTotal":999}\n', encoding="utf-8")
        failed = run("verify", root, expect_success=False)
        assert "digest/size mismatch" in failed.stderr or "differ" in failed.stderr


def test_source_mutation_during_execution_fails_closed() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        mutated = definition(mutate_input=True)
        definition_path.write_text(json.dumps(mutated, indent=2) + "\n", encoding="utf-8")
        failed = run("run", root, definition_path, expect_success=False)
        assert "changed during analysis execution" in failed.stderr
        assert not (root / "analysis/analysis-provenance.json").exists()


def test_confirmatory_status_requires_eligible_frozen_study() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root, eligible=False)
        failed = run("run", root, definition_path, expect_success=False)
        assert "eligible for a pre-result confirmatory claim" in failed.stderr
        assert not (root / "analysis/result.json").exists()


def test_executable_argument_change_changes_execution_and_identity() -> None:
    identities = []
    outputs = []
    for scale in (2, 3):
        with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
            root = Path(directory)
            definition_path = make_study(root)
            current = definition()
            scale_index = current["command"].index("--scale") + 1
            current["command"][scale_index] = str(scale)
            definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
            identities.append(run("run", root, definition_path).stdout.strip())
            outputs.append(json.loads((root / "analysis/result.json").read_text())["scaledTotal"])
    assert outputs == [10, 15]
    assert identities[0] != identities[1]


def test_config_file_configuration_is_execution_bound() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        config_path = root / "analysis/config.json"
        config_path.write_text(json.dumps({"scale": 3}) + "\n", encoding="utf-8")
        current = definition()
        scale_flag = current["command"].index("--scale")
        current["command"][scale_flag:scale_flag + 2] = ["--config", "analysis/config.json"]
        current["implementation"].append({"path": "analysis/config.json", "role": "analysis-configuration"})
        definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        identity = run("run", root, definition_path).stdout.strip()
        assert json.loads((root / "analysis/result.json").read_text())["scaledTotal"] == 15
        assert run("replay", root).stdout.strip() == identity
        config_path.write_text(json.dumps({"scale": 30}) + "\n", encoding="utf-8")
        failed = run("verify", root, expect_success=False)
        assert "digest/size mismatch" in failed.stderr or "differ" in failed.stderr


def test_annotations_are_explicitly_nonexecuted_metadata() -> None:
    identities = []
    outputs = []
    for label in ("descriptive-a", "descriptive-b"):
        with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
            root = Path(directory)
            definition_path = make_study(root)
            current = definition()
            current["annotations"]["note"] = label
            definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
            identities.append(run("run", root, definition_path).stdout.strip())
            outputs.append(json.loads((root / "analysis/result.json").read_text())["scaledTotal"])
    assert outputs == [10, 10]
    assert identities[0] != identities[1]


def test_unbound_observation_model_identity_is_rejected() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        current = definition()
        current["observationModelIdentity"] = "observation-model-that-command-does-not-select"
        definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        failed = run("run", root, definition_path, expect_success=False)
        assert "unknown field" in failed.stderr
        assert not (root / "analysis/result.json").exists()


def test_unbound_rng_seed_metadata_is_rejected() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        current = definition()
        current["analysisRngSeeds"] = [9001]
        definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        failed = run("run", root, definition_path, expect_success=False)
        assert "unknown field" in failed.stderr
        assert not (root / "analysis/result.json").exists()


def test_legacy_v1_arguments_definition_is_rejected() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        current = definition()
        current["schemaVersion"] = 1
        current["arguments"] = {"scale": 3}
        definition_path.write_text(json.dumps(current, indent=2) + "\n", encoding="utf-8")
        failed = run("run", root, definition_path, expect_success=False)
        assert "unsupported analysis definition schema" in failed.stderr or "unknown field" in failed.stderr
        assert not (root / "analysis/result.json").exists()


def test_manual_capture_is_visible_and_verifiable() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        manual = definition(mode="external_or_manual")
        definition_path.write_text(json.dumps(manual, indent=2) + "\n", encoding="utf-8")
        (root / "analysis/result.json").write_text(
            '{"schemaVersion":1,"scaledTotal":10}\n', encoding="utf-8"
        )
        identity = run("capture", root, definition_path).stdout.strip()
        record = load_record(root)
        assert record["executionStatus"] == "captured_external_or_manual"
        assert record["definition"]["manualSteps"]
        assert run("verify", root).stdout.strip() == identity


def test_manual_capture_requires_declared_steps() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        manual = definition(mode="external_or_manual")
        manual["manualSteps"] = []
        definition_path.write_text(json.dumps(manual, indent=2) + "\n", encoding="utf-8")
        (root / "analysis/result.json").write_text('{"scaledTotal":10}\n', encoding="utf-8")
        failed = run("capture", root, definition_path, expect_success=False)
        assert "requires at least one declared manualStep" in failed.stderr


def test_record_tamper_breaks_identity() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        run("run", root, definition_path)
        path = root / "analysis/analysis-provenance.json"
        record = json.loads(path.read_text(encoding="utf-8"))
        scale_index = record["definition"]["command"].index("--scale") + 1
        record["definition"]["command"][scale_index] = "999"
        path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        failed = run("verify", root, expect_success=False)
        assert "definition identity mismatch" in failed.stderr or "provenance identity mismatch" in failed.stderr


def test_integrity_archive_closes_end_to_end_chain() -> None:
    if not INTEGRITY.is_file():
        return
    with tempfile.TemporaryDirectory(prefix="anthrosim-analysis-prov-") as directory:
        root = Path(directory)
        definition_path = make_study(root)
        run("run", root, definition_path)
        create = subprocess.run(
            [sys.executable, str(INTEGRITY), "create", str(root)],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        assert create.returncode == 0, create.stderr
        verify = subprocess.run(
            [sys.executable, str(INTEGRITY), "verify", str(root)],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        assert verify.returncode == 0, verify.stderr
        (root / "analysis/result.json").write_text('{"scaledTotal":1234}\n', encoding="utf-8")
        changed = subprocess.run(
            [sys.executable, str(INTEGRITY), "verify", str(root)],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        assert changed.returncode != 0


def main() -> None:
    assert SCRIPT.is_file(), SCRIPT
    tests = [
        test_scripted_run_and_verify,
        test_replay_reproduces_exact_output_bytes,
        test_output_tamper_is_rejected,
        test_source_mutation_during_execution_fails_closed,
        test_confirmatory_status_requires_eligible_frozen_study,
        test_executable_argument_change_changes_execution_and_identity,
        test_config_file_configuration_is_execution_bound,
        test_annotations_are_explicitly_nonexecuted_metadata,
        test_unbound_observation_model_identity_is_rejected,
        test_unbound_rng_seed_metadata_is_rejected,
        test_legacy_v1_arguments_definition_is_rejected,
        test_manual_capture_is_visible_and_verifiable,
        test_manual_capture_requires_declared_steps,
        test_record_tamper_breaks_identity,
        test_integrity_archive_closes_end_to_end_chain,
    ]
    for test in tests:
        test()
        print(f"ok: {test.__name__}")
    print(f"{len(tests)} research analysis-provenance regression tests passed")


if __name__ == "__main__":
    main()
