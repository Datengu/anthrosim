#!/usr/bin/env python3
"""Regression tests for the fixed-file read-only Explorer server."""

from __future__ import annotations

import importlib.util
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


def main() -> None:
    with tempfile.TemporaryDirectory(prefix="anthrosim-explorer-server-") as temp:
        root = Path(temp)
        run_dir = root / "run"
        run_dir.mkdir()
        for name in server_module.BASE_RUN_FILES:
            (run_dir / name).write_text("{}\n")

        temporary_payload = b'{"schemaVersion":1,"summary":{"journeysStarted":2}}\n'
        (run_dir / "temporary-observability.json").write_bytes(temporary_payload)
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

            app_source = (REPO_ROOT / "explorer" / "app.mjs").read_text()
            index_source = (REPO_ROOT / "explorer" / "index.html").read_text()
            assert 'fetchArtifact("temporary-observability.json", { optional: true })' in app_source
            assert 'id="temporary-m9"' in index_source
            assert "renderTemporaryMobility();" in app_source
        finally:
            server.shutdown()
            server.server_close()
            thread.join(timeout=3)

    print("Explorer server M9 regression checks passed")


if __name__ == "__main__":
    main()
