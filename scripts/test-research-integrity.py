#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("research-integrity.py")


class ResearchIntegrityCliTests(unittest.TestCase):
    def run_cli(
        self, *args: object, check: bool = True
    ) -> subprocess.CompletedProcess[str]:
        result = subprocess.run(
            [sys.executable, str(SCRIPT), *(str(arg) for arg in args)],
            text=True,
            capture_output=True,
            check=False,
        )
        if check and result.returncode != 0:
            self.fail(f"command failed: {result.stderr}\n{result.stdout}")
        return result

    def make_archive(self, root: Path) -> None:
        (root / "nested").mkdir(parents=True)
        (root / "alpha.txt").write_text("alpha\n", encoding="utf-8")
        (root / "nested" / "beta.bin").write_bytes(b"\x00\x01\x02")

    def test_create_is_deterministic_and_verify_succeeds(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_archive(root)
            self.run_cli("create", root)
            first = (root / "integrity-manifest.json").read_bytes()
            self.run_cli("create", root)
            second = (root / "integrity-manifest.json").read_bytes()
            self.assertEqual(first, second)
            manifest = json.loads(first)
            self.assertEqual(
                manifest["manifestType"], "anthrosim-research-integrity"
            )
            self.assertEqual(manifest["schemaVersion"], 1)
            self.assertEqual(manifest["algorithm"], "sha256")
            self.assertEqual(
                [entry["path"] for entry in manifest["files"]],
                ["alpha.txt", "nested/beta.bin"],
            )
            self.run_cli("verify", root)

    def test_modified_file_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_archive(root)
            self.run_cli("create", root)
            (root / "alpha.txt").write_text("altered\n", encoding="utf-8")
            result = self.run_cli("verify", root, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("mismatch", result.stderr)

    def test_missing_and_unexpected_files_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_archive(root)
            self.run_cli("create", root)
            (root / "alpha.txt").unlink()
            missing = self.run_cli("verify", root, check=False)
            self.assertIn("missing manifested file", missing.stderr)

            (root / "alpha.txt").write_text("alpha\n", encoding="utf-8")
            self.run_cli("create", root)
            (root / "extra.txt").write_text("extra\n", encoding="utf-8")
            unexpected = self.run_cli("verify", root, check=False)
            self.assertIn("unexpected file", unexpected.stderr)

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks unavailable")
    def test_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_archive(root)
            os.symlink(root / "alpha.txt", root / "alias.txt")
            result = self.run_cli("create", root, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("symbolic links are not allowed", result.stderr)

    def test_unsafe_manifest_path_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_archive(root)
            self.run_cli("create", root)
            manifest_path = root / "integrity-manifest.json"
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["files"][0]["path"] = "../outside.txt"
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            result = self.run_cli("verify", root, check=False)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("unsafe relative path", result.stderr)


if __name__ == "__main__":
    unittest.main()
