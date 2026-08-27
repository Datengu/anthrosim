#!/usr/bin/env python3
from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-analysis-window.py")


class ResearchAnalysisWindowCliTests(unittest.TestCase):
    def run_cli(
        self, root: Path, protocol: Path, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), str(root), str(protocol)],
            text=True,
            capture_output=True,
            check=False,
        )
        if check and result.returncode != 0:
            self.fail(f"command failed: {result.stderr}\n{result.stdout}")
        return result

    @staticmethod
    def write_json(path: Path, value: object) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def make_research_root(self, root: Path) -> tuple[dict, Path]:
        relative_dir = "points/point-000000/runs/seed-000000-00000000000000000011"
        manifest = {
            "schemaVersion": 1,
            "researchId": "research-execution-v1-test",
            "definitionIdentity": "research-definition-v1-test",
            "source": {
                "modelVersion": "0.3.0",
                "modelSemanticsId": "anthrosim-model-semantics-v13",
                "gitCommit": "0123456789abcdef",
            },
            "definition": {"schemaVersion": 1},
            "points": [
                {
                    "point": {
                        "schemaVersion": 1,
                        "index": 0,
                        "pointId": "research-point-v1-test",
                        "coordinates": [
                            {
                                "id": "resource_scale",
                                "kind": "numeric",
                                "path": "/experiment/resources/cellProductivityUnitsPerPeriod",
                                "value": 100,
                            }
                        ],
                        "runConfig": {"experiment": {"durationYears": 2}},
                    },
                    "runs": [
                        {
                            "seed": 11,
                            "runId": "research-run-v1-test",
                            "relativeDir": relative_dir,
                            "runConfig": {"experiment": {"durationYears": 2}},
                        }
                    ],
                }
            ],
        }
        self.write_json(root / "research-manifest.json", manifest)
        self.write_json(root / "research-plan.json", manifest)
        state = {
            "schemaVersion": 1,
            "researchId": manifest["researchId"],
            "runs": {
                "research-run-v1-test": {
                    "runId": "research-run-v1-test",
                    "pointId": "research-point-v1-test",
                    "seed": 11,
                    "relativeDir": relative_dir,
                    "attempt": 1,
                    "state": "completed",
                    "stateDigest64": 123,
                }
            },
        }
        self.write_json(root / "research-state.json", state)
        run_dir = root / relative_dir
        self.write_json(
            run_dir / "metrics.json",
            {
                "schemaVersion": 3,
                "cadence": "annual_boundary_plus_terminal",
                "snapshots": [
                    {"schemaVersion": 3, "day": 0},
                    {"schemaVersion": 3, "day": 365},
                    {"schemaVersion": 3, "day": 730},
                ],
            },
        )
        protocol_path = root.parent / "analysis-window.json"
        self.write_json(
            protocol_path,
            {
                "schemaVersion": 1,
                "studyId": "burn-in-example",
                "analysisWindow": {
                    "analysisStartDay": 365,
                    "selectionRule": "predeclared_fixed_duration",
                    "rationale": "Exclude one predeclared initialization year.",
                },
                "sensitivityWindows": [
                    {
                        "id": "no_burn_in",
                        "analysisStartDay": 0,
                        "rationale": "Initialization-sensitivity comparison.",
                    },
                    {
                        "id": "longer_burn_in",
                        "analysisStartDay": 500,
                        "analysisEndDayInclusive": 730,
                        "rationale": "Longer plausible equilibration comparison.",
                    },
                ],
            },
        )
        return manifest, protocol_path

    def test_declared_window_is_bound_to_research_identity_and_filters_snapshot_days(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "research"
            root.mkdir()
            manifest, protocol = self.make_research_root(root)

            first = self.run_cli(root, protocol)
            output_dir = Path(first.stdout.strip())
            result_path = output_dir / "analysis-window-manifest.json"
            first_bytes = result_path.read_bytes()
            result = json.loads(first_bytes)

            self.assertEqual(result["manifestType"], "anthrosim-research-analysis-window")
            self.assertEqual(result["schemaVersion"], 1)
            self.assertEqual(result["researchId"], manifest["researchId"])
            self.assertEqual(result["definitionIdentity"], manifest["definitionIdentity"])
            self.assertTrue(result["protocolIdentity"].startswith("analysis-window-protocol-v1-sha256-"))
            self.assertEqual(result["runCount"], 1)
            self.assertEqual(result["completedRunCount"], 1)

            row = result["runs"][0]
            self.assertEqual(
                row["primaryWindow"]["executionInterval"],
                {"startDay": 0, "endDayInclusive": 730},
            )
            self.assertEqual(
                row["primaryWindow"]["burnInInterval"],
                {"startDay": 0, "endDayExclusive": 365},
            )
            self.assertEqual(
                row["primaryWindow"]["analysisInterval"],
                {"startDay": 365, "endDayInclusive": 730},
            )
            selection = row["metricSnapshotSelection"]
            self.assertEqual(selection["precedingSnapshotDay"], 0)
            self.assertTrue(selection["analysisStartBoundarySnapshotAvailable"])
            self.assertEqual(selection["includedSnapshotDays"], [365, 730])
            self.assertNotIn(0, selection["includedSnapshotDays"])

            second = self.run_cli(root, protocol)
            self.assertEqual(Path(second.stdout.strip()), output_dir)
            self.assertEqual(first_bytes, result_path.read_bytes())

    def test_changing_analysis_start_creates_new_protocol_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "research"
            root.mkdir()
            _, protocol = self.make_research_root(root)
            first = Path(self.run_cli(root, protocol).stdout.strip())

            changed = json.loads(protocol.read_text(encoding="utf-8"))
            changed["analysisWindow"]["analysisStartDay"] = 400
            changed["analysisWindow"]["rationale"] = "Predeclared 400-day burn-in."
            changed_protocol = parent / "analysis-window-changed.json"
            self.write_json(changed_protocol, changed)
            second = Path(self.run_cli(root, changed_protocol).stdout.strip())

            self.assertNotEqual(first, second)
            result = json.loads((second / "analysis-window-manifest.json").read_text())
            selection = result["runs"][0]["metricSnapshotSelection"]
            self.assertFalse(selection["analysisStartBoundarySnapshotAvailable"])
            self.assertEqual(selection["precedingSnapshotDay"], 365)
            self.assertEqual(selection["includedSnapshotDays"], [730])

    def test_window_beyond_execution_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "research"
            root.mkdir()
            _, protocol = self.make_research_root(root)
            bad = json.loads(protocol.read_text(encoding="utf-8"))
            bad["analysisWindow"]["analysisStartDay"] = 731
            bad_path = parent / "bad-window.json"
            self.write_json(bad_path, bad)

            result = self.run_cli(root, bad_path, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("beyond run terminal day 730", result.stderr)

    def test_duplicate_sensitivity_window_and_empty_rationale_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "research"
            root.mkdir()
            _, protocol = self.make_research_root(root)
            bad = json.loads(protocol.read_text(encoding="utf-8"))
            duplicate = copy.deepcopy(bad["sensitivityWindows"][0])
            bad["sensitivityWindows"].append(duplicate)
            bad_path = parent / "duplicate.json"
            self.write_json(bad_path, bad)
            duplicate_result = self.run_cli(root, bad_path, check=False)
            self.assertIn("duplicate sensitivity window id", duplicate_result.stderr)

            bad = json.loads(protocol.read_text(encoding="utf-8"))
            bad["analysisWindow"]["rationale"] = "   "
            bad_path = parent / "empty-rationale.json"
            self.write_json(bad_path, bad)
            rationale_result = self.run_cli(root, bad_path, check=False)
            self.assertIn("rationale must be a non-empty string", rationale_result.stderr)

    def test_mismatched_immutable_plan_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "research"
            root.mkdir()
            manifest, protocol = self.make_research_root(root)
            changed_plan = copy.deepcopy(manifest)
            changed_plan["researchId"] = "different-research"
            self.write_json(root / "research-plan.json", changed_plan)

            result = self.run_cli(root, protocol, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("do not contain the same immutable plan", result.stderr)


if __name__ == "__main__":
    unittest.main()
