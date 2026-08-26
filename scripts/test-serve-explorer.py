#!/usr/bin/env python3
"""Regression tests for the fixed-file read-only Explorer server."""

from __future__ import annotations

import importlib.util
import os
import tempfile
import threading
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SERVER_PATH = REPO_ROOT / "scripts" / "serve-explorer.py"

spec = importlib.util.spec_from_file_location("anthrosim_serve_explorer", SERVER_PATH)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load serve-explorer.py")
server_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(server_module)


def request(url: str, *, method: str = "GET") -> tuple[int, bytes]:
    request_object = urllib.request.Request(url, method=method)
    try:
        with urllib.request.urlopen(request_object, timeout=3) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def make_symlink(target: Path, link: Path) -> bool:
    try:
        os.symlink(target, link)
        return True
    except (OSError, NotImplementedError):
        return False


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-explorer-server-") as temp:
        root = Path(temp)
        run_dir = root / "run"
        run_dir.mkdir()
        for name in server_module.BASE_RUN_FILES:
            (run_dir / name).write_text("{}\n")

        temporary_payload = b'{"schemaVersion":1,"summary":{"journeysStarted":2}}\n'
        temporary_path = run_dir / "temporary-observability.json"
        temporary_path.write_bytes(temporary_payload)
        (run_dir / "secret.txt").write_text("must not be served\n")

        handler = type(
            "TestExplorerHandler",
            (server_module.ExplorerHandler,),
            {"explorer_dir": REPO_ROOT / "explorer", "run_dir": run_dir},
        )
        server = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        host, port = server.server_address
        base = f"http://{host}:{port}"

        try:
            status, payload = request(f"{base}/run/temporary-observability.json")
            assert status == 200, status
            assert payload == temporary_payload

            status, _ = request(f"{base}/run/secret.txt")
            assert status == 404, status

            status, _ = request(f"{base}/run/temporary-observability.json", method="POST")
            assert status == 405, status

            outside = root / "outside-secret.json"
            outside_payload = b'{"secret":"outside-run-directory"}\n'
            outside.write_bytes(outside_payload)
            temporary_path.unlink()
            if make_symlink(outside, temporary_path):
                assert not server_module.regular_file_without_symlink(temporary_path, run_dir)
                status, payload = request(f"{base}/run/temporary-observability.json")
                assert status == 404, status
                assert outside_payload not in payload
                temporary_path.unlink()

                missing = root / "missing-optional-artifact.json"
                if make_symlink(missing, temporary_path):
                    assert not server_module.regular_file_without_symlink(temporary_path, run_dir)
                    status, _ = request(f"{base}/run/temporary-observability.json")
                    assert status == 404, status
                    temporary_path.unlink()

                temporary_path.write_bytes(temporary_payload)

                required = run_dir / "checkpoint.json"
                required.unlink()
                if make_symlink(outside, required):
                    assert not server_module.regular_file_without_symlink(required, run_dir)

            app_source = (REPO_ROOT / "explorer" / "app.mjs").read_text()
            index_source = (REPO_ROOT / "explorer" / "index.html").read_text()
            assert 'fetchArtifact("temporary-observability.json", { optional: true })' in app_source
            assert 'id="temporary-m9"' in index_source
            assert "renderTemporaryMobility();" in app_source
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=3)

    print("Explorer server M9 and symlink regression checks passed")


if __name__ == "__main__":
    main()
