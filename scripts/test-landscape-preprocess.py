#!/usr/bin/env python3

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "build-landscape-bundle.py"
FIXTURE = ROOT / "examples" / "landscape-preprocess" / "recipe.json"


class LandscapePreprocessTests(unittest.TestCase):
    def run_builder(self, recipe: Path, output: Path, record: Path, expect_success: bool = True):
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                str(recipe),
                "--output",
                str(output),
                "--record",
                str(record),
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        if expect_success and result.returncode != 0:
            self.fail(f"builder failed: {result.stderr}\n{result.stdout}")
        return result

    def test_fixture_is_deterministic_and_has_expected_contract(self):
        with tempfile.TemporaryDirectory() as first_dir, tempfile.TemporaryDirectory() as second_dir:
            first_output = Path(first_dir) / "landscape.json"
            first_record = Path(first_dir) / "record.json"
            second_output = Path(second_dir) / "landscape.json"
            second_record = Path(second_dir) / "record.json"

            self.run_builder(FIXTURE, first_output, first_record)
            self.run_builder(FIXTURE, second_output, second_record)

            self.assertEqual(first_output.read_bytes(), second_output.read_bytes())
            self.assertEqual(first_record.read_bytes(), second_record.read_bytes())

            landscape = json.loads(first_output.read_text())
            record = json.loads(first_record.read_text())
            self.assertEqual(landscape["schemaVersion"], 1)
            self.assertEqual((landscape["width"], landscape["height"]), (2, 2))
            self.assertEqual(landscape["geometry"]["spatialReference"], "EPSG:27700")
            self.assertEqual(landscape["layers"][1]["values"], [900, 700, None, 300])
            self.assertEqual(len(record["landscapeSha256"]), 64)
            self.assertEqual(len(record["recipeSha256"]), 64)
            self.assertEqual(len(record["sources"][0]["sha256"]), 64)
            self.assertEqual(len(record["layerInputs"]), 2)
            self.assertIn("gdalwarp", record["steps"][0]["command"])

    def test_rejects_misaligned_grid_shape(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.csv").write_text("1,2\n3,4\n")
            (root / "layer.csv").write_text("1,2,3\n4,5,6\n")
            recipe = {
                "schemaVersion": 1,
                "geometry": {
                    "width": 2,
                    "height": 2,
                    "originX": 0,
                    "originY": 0,
                    "cellSizeX": 1,
                    "cellSizeY": 1,
                    "coordinateUnit": "metre",
                    "spatialReference": "LOCAL:TEST",
                },
                "toolchain": [{"name": "test", "version": "1"}],
                "steps": [{"description": "test step", "command": "test command"}],
                "sources": [
                    {
                        "sourceId": "source",
                        "path": "source.csv",
                        "citation": "synthetic test",
                    }
                ],
                "nodataPolicy": "NA remains nodata",
                "layers": [
                    {
                        "layerId": "terrain",
                        "role": "terrain_traversal",
                        "unit": "permille",
                        "path": "layer.csv",
                    }
                ],
            }
            recipe_path = root / "recipe.json"
            recipe_path.write_text(json.dumps(recipe))
            result = self.run_builder(
                recipe_path,
                root / "output.json",
                root / "record.json",
                expect_success=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("has 3 cells; expected 2", result.stderr)

    def test_rejects_values_outside_declared_domain(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.csv").write_text("1\n")
            (root / "layer.csv").write_text("1001\n")
            recipe = {
                "schemaVersion": 1,
                "geometry": {
                    "width": 1,
                    "height": 1,
                    "originX": 0,
                    "originY": 0,
                    "cellSizeX": 1,
                    "cellSizeY": 1,
                    "coordinateUnit": "metre",
                    "spatialReference": "LOCAL:TEST",
                },
                "toolchain": [{"name": "test", "version": "1"}],
                "steps": [{"description": "test step", "command": "test command"}],
                "sources": [
                    {
                        "sourceId": "source",
                        "path": "source.csv",
                        "citation": "synthetic test",
                    }
                ],
                "nodataPolicy": "none",
                "layers": [
                    {
                        "layerId": "terrain",
                        "role": "terrain_traversal",
                        "unit": "permille",
                        "valueDomain": {"min": 0, "max": 1000},
                        "path": "layer.csv",
                    }
                ],
            }
            recipe_path = root / "recipe.json"
            recipe_path.write_text(json.dumps(recipe))
            result = self.run_builder(
                recipe_path,
                root / "output.json",
                root / "record.json",
                expect_success=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("outside [0, 1000]", result.stderr)


if __name__ == "__main__":
    unittest.main()
