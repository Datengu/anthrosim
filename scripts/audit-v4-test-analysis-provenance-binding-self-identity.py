#!/usr/bin/env python3
"""Audit-v4 Area K adversary: downstream provenance must verify study-result self-identity."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
WRAPPER = ROOT / "scripts" / "research-analysis-provenance.py"


def load_wrapper():
    spec = importlib.util.spec_from_file_location("research_analysis_provenance", WRAPPER)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load research-analysis-provenance.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x00000100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


def producer_result_identity(binding: dict[str, Any]) -> str:
    identity = {
        "schemaVersion": binding["schemaVersion"],
        "studyExecutionId": binding["studyExecutionId"],
        "protocolIdentity": binding["protocolIdentity"],
        "protocolRevision": binding["protocolRevision"],
        "studyId": binding["studyId"],
        "scientificStatus": binding["scientificStatus"],
        "boundBeforeExecution": binding["boundBeforeExecution"],
        "confirmatoryPreResultClaimEligible": binding["confirmatoryPreResultClaimEligible"],
        "definitionIdentity": binding["definitionIdentity"],
        "researchId": binding["researchId"],
        "source": binding["source"],
        "researchRelativeDir": binding["researchRelativeDir"],
        "runCounts": binding["runCounts"],
        "resultArtifacts": binding["resultArtifacts"],
    }
    if binding.get("analysisRequirements"):
        identity["analysisRequirements"] = binding["analysisRequirements"]
    return f"study-result-v1-{fnv1a64(canonical_bytes(identity)):016x}"


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    wrapper = load_wrapper()
    with tempfile.TemporaryDirectory(prefix="anthrosim-audit-v4-area-k-") as directory:
        root = Path(directory)
        (root / "input.json").write_text('{"value":1}\n', encoding="utf-8")
        (root / "implementation.txt").write_text("frozen implementation\n", encoding="utf-8")
        (root / "result.txt").write_text("captured result\n", encoding="utf-8")

        binding = {
            "schemaVersion": 1,
            "resultIdentity": "",
            "studyExecutionId": "study-execution-v1-1111111111111111",
            "protocolIdentity": "study-protocol-v1-2222222222222222",
            "protocolRevision": 1,
            "studyId": "audit-v4-area-k-binding-self-identity",
            "scientificStatus": "exploratory",
            "boundBeforeExecution": True,
            "confirmatoryPreResultClaimEligible": False,
            "definitionIdentity": "research-definition-v1-3333333333333333",
            "researchId": "research-execution-v1-original",
            "source": {
                "modelVersion": "0.3.4",
                "modelSemanticsId": "anthrosim-model-semantics-v25",
                "gitCommit": "8996e99ffc4c5b91b9e00d1048eedd4227ea1d09",
            },
            "researchRelativeDir": "research",
            "runCounts": {"completed": 1, "failed": 0},
            "resultArtifacts": [
                {"path": "research/analysis/points.json", "digest64": 123456789}
            ],
            "analysisRequirements": [],
        }
        binding["resultIdentity"] = producer_result_identity(binding)
        original_identity = binding["resultIdentity"]
        assert producer_result_identity(binding) == original_identity

        # Post-finalization tamper: change a field that the producer includes in
        # resultIdentity, but deliberately leave resultIdentity itself untouched.
        binding["researchId"] = "research-execution-v1-tampered"
        assert producer_result_identity(binding) != original_identity
        write_json(root / "study-result-binding.json", binding)

        definition = {
            "schemaVersion": 2,
            "definitionType": "anthrosim-analysis-definition",
            "analysisId": "audit-v4-area-k-binding-self-identity",
            "analysisStatus": "exploratory",
            "executionMode": "external_or_manual",
            "workingDirectory": ".",
            "command": [],
            "runtimeDescription": "Audit-v4 controlled external/manual capture",
            "reproductionCriterion": "exact_output_bytes",
            "inputs": [{"path": "input.json", "role": "controlled-input"}],
            "implementation": [
                {"path": "implementation.txt", "role": "controlled-implementation"}
            ],
            "environment": [],
            "outputs": [{"path": "result.txt", "role": "controlled-output"}],
            "manualSteps": ["Use the already-produced controlled result."],
        }
        definition_path = root / "definition.json"
        write_json(definition_path, definition)

        record = wrapper.prepare_record(root, definition_path, None, execute=False)
        verified = wrapper.verify_record(root, None)

        print(f"original_result_identity={original_identity}")
        print(f"identity_after_tamper={producer_result_identity(binding)}")
        print(f"binding_research_id={binding['researchId']}")
        print(f"record_research_id={record['study']['researchId']}")
        print(f"capture_published={record['executionStatus']}")
        print(f"verify_accepted={verified['provenanceIdentity'] == record['provenanceIdentity']}")

        if verified["study"]["researchId"] == binding["researchId"]:
            raise AssertionError(
                "analysis provenance capture+verify accepted a study-result-binding whose "
                "resultIdentity no longer matches the producer-defined binding contents"
            )


if __name__ == "__main__":
    main()
