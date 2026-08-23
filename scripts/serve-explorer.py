#!/usr/bin/env python3
"""Serve the AnthroSim explorer and exactly one run bundle read-only."""

from __future__ import annotations

import argparse
import mimetypes
import threading
import webbrowser
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlparse

BASE_RUN_FILES = {
    "world.json",
    "initial-population.json",
    "events.json",
    "metrics.json",
    "checkpoint.json",
}
OPTIONAL_RUN_FILES = {
    "manifest.json",
    "resume-start-population.json",
    "landscape.json",
    "landscape-manifest.json",
    "landscape-checkpoint.json",
    "spatial-mechanisms.json",
    "spatial-observability.json",
}
ALLOWED_RUN_FILES = BASE_RUN_FILES | OPTIONAL_RUN_FILES
EXPLORER_FILES = {
    "index.html",
    "app.mjs",
    "model.mjs",
    "spatial.mjs",
    "spatial-model.mjs",
    "style.css",
}


class ExplorerHandler(BaseHTTPRequestHandler):
    explorer_dir: Path
    run_dir: Path

    def _resolve(self) -> Path | None:
        path = unquote(urlparse(self.path).path)
        if path == "/":
            return self.explorer_dir / "index.html"
        if path.startswith("/run/"):
            name = path.removeprefix("/run/")
            if name not in ALLOWED_RUN_FILES:
                return None
            return self.run_dir / name
        name = path.removeprefix("/")
        if name not in EXPLORER_FILES:
            return None
        return self.explorer_dir / name

    def _send_file(self, *, head_only: bool = False) -> None:
        target = self._resolve()
        if target is None or not target.is_file():
            self.send_error(HTTPStatus.NOT_FOUND)
            return

        payload = target.read_bytes()
        content_type = mimetypes.guess_type(target.name)[0] or "application/octet-stream"
        if target.suffix == ".mjs":
            content_type = "text/javascript"

        self.send_response(HTTPStatus.OK)
        self.send_header("Content-Type", f"{content_type}; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header(
            "Content-Security-Policy",
            "default-src 'self'; script-src 'self'; style-src 'self'; "
            "connect-src 'self'; img-src 'self' data:; object-src 'none'; base-uri 'none'",
        )
        self.end_headers()
        if not head_only:
            self.wfile.write(payload)

    def do_GET(self) -> None:  # noqa: N802 - stdlib handler API
        self._send_file()

    def do_HEAD(self) -> None:  # noqa: N802 - stdlib handler API
        self._send_file(head_only=True)

    def do_POST(self) -> None:  # noqa: N802 - stdlib handler API
        self.send_error(HTTPStatus.METHOD_NOT_ALLOWED, "explorer is read-only")

    def do_PUT(self) -> None:  # noqa: N802 - stdlib handler API
        self.send_error(HTTPStatus.METHOD_NOT_ALLOWED, "explorer is read-only")

    def do_DELETE(self) -> None:  # noqa: N802 - stdlib handler API
        self.send_error(HTTPStatus.METHOD_NOT_ALLOWED, "explorer is read-only")

    def log_message(self, format: str, *args: object) -> None:
        print(f"[explorer] {self.address_string()} - {format % args}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Serve the read-only AnthroSim local explorer for one completed or paused run bundle."
    )
    parser.add_argument(
        "run_dir",
        type=Path,
        help="run directory containing world, original founder population, events, metrics and checkpoint; resume-boundary provenance, manifest and M8 spatial artifacts are optional",
    )
    parser.add_argument("--host", default="127.0.0.1", help="bind address; defaults to loopback only")
    parser.add_argument("--port", type=int, default=8765, help="local port; defaults to 8765")
    parser.add_argument("--no-browser", action="store_true", help="do not open the default browser automatically")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    repository_root = Path(__file__).resolve().parent.parent
    explorer_dir = repository_root / "explorer"
    run_dir = args.run_dir.resolve()

    missing = sorted(name for name in BASE_RUN_FILES if not (run_dir / name).is_file())
    if missing:
        raise SystemExit(f"run bundle is incomplete; missing: {', '.join(missing)}")
    missing_ui = sorted(name for name in EXPLORER_FILES if not (explorer_dir / name).is_file())
    if missing_ui:
        raise SystemExit(f"explorer installation is incomplete; missing: {', '.join(missing_ui)}")

    bundle_kind = "completed" if (run_dir / "manifest.json").is_file() else "paused checkpoint"
    handler = type(
        "BoundExplorerHandler",
        (ExplorerHandler,),
        {"explorer_dir": explorer_dir, "run_dir": run_dir},
    )
    server = ThreadingHTTPServer((args.host, args.port), handler)
    url = f"http://{args.host}:{args.port}/"
    print(f"AnthroSim explorer: {url}")
    print(f"Read-only {bundle_kind} bundle: {run_dir}")
    print("Press Ctrl+C to stop.")

    if not args.no_browser:
        threading.Timer(0.35, lambda: webbrowser.open(url)).start()

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping explorer.")
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
