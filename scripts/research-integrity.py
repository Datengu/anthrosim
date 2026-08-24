#!/usr/bin/env python3
"""Create and verify deterministic SHA-256 manifests for preserved research archives."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import sys
import tempfile
from pathlib import Path, PurePosixPath
from typing import Iterable

SCHEMA_VERSION = 1
MANIFEST_TYPE = "anthrosim-research-integrity"
ALGORITHM = "sha256"
DEFAULT_MANIFEST_NAME = "integrity-manifest.json"
READ_CHUNK_SIZE = 1024 * 1024


class IntegrityError(Exception):
    """Raised when an archive cannot be safely manifested or verified."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Create or verify a deterministic SHA-256 manifest for a preserved "
            "AnthroSim research archive directory."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser("create", help="create or replace an integrity manifest")
    create.add_argument("root", type=Path, help="archive directory to hash recursively")
    create.add_argument(
        "--output",
        type=Path,
        help=f"manifest path (default: ROOT/{DEFAULT_MANIFEST_NAME})",
    )

    verify = subparsers.add_parser("verify", help="verify an archive against a manifest")
    verify.add_argument("root", type=Path, help="archive directory to verify")
    verify.add_argument(
        "--manifest",
        type=Path,
        help=f"manifest path (default: ROOT/{DEFAULT_MANIFEST_NAME})",
    )

    return parser.parse_args()


def canonical_root(root: Path) -> Path:
    try:
        resolved = root.resolve(strict=True)
    except FileNotFoundError as error:
        raise IntegrityError(f"archive root does not exist: {root}") from error
    if not resolved.is_dir():
        raise IntegrityError(f"archive root is not a directory: {root}")
    return resolved


def manifest_path_inside_root(root: Path, manifest_path: Path) -> str | None:
    absolute = manifest_path if manifest_path.is_absolute() else Path.cwd() / manifest_path
    normalized = absolute.resolve(strict=False)
    try:
        relative = normalized.relative_to(root)
    except ValueError:
        return None
    if not relative.parts:
        raise IntegrityError("manifest path cannot be the archive root directory")
    return relative.as_posix()


def validate_relative_path(value: object) -> str:
    if not isinstance(value, str) or not value:
        raise IntegrityError("manifest file path must be a non-empty string")
    if "\\" in value:
        raise IntegrityError(f"manifest file path must use POSIX separators: {value!r}")
    path = PurePosixPath(value)
    if path.is_absolute() or value == "." or any(part in ("", ".", "..") for part in path.parts):
        raise IntegrityError(f"manifest contains unsafe relative path: {value!r}")
    normalized = path.as_posix()
    if normalized != value:
        raise IntegrityError(f"manifest contains non-canonical relative path: {value!r}")
    return normalized


def enumerate_regular_files(
    root: Path, excluded_relative_paths: Iterable[str] = ()
) -> list[tuple[str, Path]]:
    excluded = set(excluded_relative_paths)
    files: list[tuple[str, Path]] = []

    for current, directory_names, file_names in os.walk(root, topdown=True, followlinks=False):
        current_path = Path(current)
        directory_names.sort()
        file_names.sort()

        for name in list(directory_names):
            path = current_path / name
            mode = os.lstat(path).st_mode
            if stat.S_ISLNK(mode):
                raise IntegrityError(
                    f"symbolic links are not allowed in research archives: {path}"
                )
            if not stat.S_ISDIR(mode):
                raise IntegrityError(
                    f"non-directory entry found where a directory was expected: {path}"
                )

        for name in file_names:
            path = current_path / name
            mode = os.lstat(path).st_mode
            if stat.S_ISLNK(mode):
                raise IntegrityError(
                    f"symbolic links are not allowed in research archives: {path}"
                )
            if not stat.S_ISREG(mode):
                raise IntegrityError(
                    f"non-regular file is not supported in research archives: {path}"
                )
            relative = path.relative_to(root).as_posix()
            if relative in excluded:
                continue
            files.append((relative, path))

    files.sort(key=lambda item: item[0])
    return files


def sha256_file(path: Path) -> tuple[str, int]:
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        before = os.fstat(handle.fileno())
        while True:
            chunk = handle.read(READ_CHUNK_SIZE)
            if not chunk:
                break
            digest.update(chunk)
            size += len(chunk)
        after = os.fstat(handle.fileno())
    if (
        before.st_size != after.st_size
        or before.st_mtime_ns != after.st_mtime_ns
        or size != after.st_size
    ):
        raise IntegrityError(f"file changed while hashing: {path}")
    return digest.hexdigest(), size


def build_manifest(root: Path, manifest_path: Path) -> dict:
    excluded = []
    manifest_relative = manifest_path_inside_root(root, manifest_path)
    if manifest_relative is not None:
        excluded.append(manifest_relative)

    entries = []
    for relative, path in enumerate_regular_files(root, excluded):
        digest, size = sha256_file(path)
        entries.append({"path": relative, "sha256": digest, "sizeBytes": size})

    return {
        "algorithm": ALGORITHM,
        "fileCount": len(entries),
        "files": entries,
        "manifestType": MANIFEST_TYPE,
        "pathSemantics": "relative-posix-v1",
        "schemaVersion": SCHEMA_VERSION,
    }


def write_manifest(path: Path, manifest: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    with tempfile.NamedTemporaryFile(
        mode="w",
        encoding="utf-8",
        newline="\n",
        dir=path.parent,
        prefix=f".{path.name}.",
        suffix=".tmp",
        delete=False,
    ) as handle:
        temporary = Path(handle.name)
        handle.write(payload)
        handle.flush()
        os.fsync(handle.fileno())
    try:
        os.replace(temporary, path)
    except Exception:
        temporary.unlink(missing_ok=True)
        raise


def load_manifest(path: Path) -> dict:
    if path.is_symlink():
        raise IntegrityError(f"integrity manifest must not be a symbolic link: {path}")
    if not path.is_file():
        raise IntegrityError(
            f"integrity manifest does not exist or is not a regular file: {path}"
        )
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise IntegrityError(f"cannot read integrity manifest {path}: {error}") from error
    if not isinstance(manifest, dict):
        raise IntegrityError("integrity manifest root must be a JSON object")
    return manifest


def parse_manifest_entries(manifest: dict) -> list[dict]:
    if manifest.get("manifestType") != MANIFEST_TYPE:
        raise IntegrityError("unsupported integrity manifest type")
    if manifest.get("schemaVersion") != SCHEMA_VERSION:
        raise IntegrityError("unsupported integrity manifest schema version")
    if manifest.get("algorithm") != ALGORITHM:
        raise IntegrityError("unsupported integrity manifest hash algorithm")
    if manifest.get("pathSemantics") != "relative-posix-v1":
        raise IntegrityError("unsupported integrity manifest path semantics")

    raw_entries = manifest.get("files")
    if not isinstance(raw_entries, list):
        raise IntegrityError("integrity manifest files must be an array")
    if manifest.get("fileCount") != len(raw_entries):
        raise IntegrityError("integrity manifest fileCount does not match files array")

    entries: list[dict] = []
    seen: set[str] = set()
    for raw in raw_entries:
        if not isinstance(raw, dict):
            raise IntegrityError("integrity manifest file entry must be an object")
        relative = validate_relative_path(raw.get("path"))
        if relative in seen:
            raise IntegrityError(f"integrity manifest contains duplicate path: {relative}")
        seen.add(relative)

        size = raw.get("sizeBytes")
        digest = raw.get("sha256")
        if not isinstance(size, int) or isinstance(size, bool) or size < 0:
            raise IntegrityError(f"invalid sizeBytes for {relative}")
        if (
            not isinstance(digest, str)
            or len(digest) != 64
            or any(character not in "0123456789abcdef" for character in digest)
        ):
            raise IntegrityError(f"invalid sha256 for {relative}")
        entries.append({"path": relative, "sha256": digest, "sizeBytes": size})

    if [entry["path"] for entry in entries] != sorted(
        entry["path"] for entry in entries
    ):
        raise IntegrityError(
            "integrity manifest file entries are not in canonical path order"
        )
    return entries


def verify_manifest(root: Path, manifest_path: Path, manifest: dict) -> None:
    entries = parse_manifest_entries(manifest)
    manifest_relative = manifest_path_inside_root(root, manifest_path)
    excluded = [manifest_relative] if manifest_relative is not None else []
    actual = {
        relative: path for relative, path in enumerate_regular_files(root, excluded)
    }
    expected = {entry["path"]: entry for entry in entries}

    missing = sorted(set(expected) - set(actual))
    unexpected = sorted(set(actual) - set(expected))
    if missing:
        raise IntegrityError(
            "archive is missing manifested file(s): " + ", ".join(missing)
        )
    if unexpected:
        raise IntegrityError(
            "archive contains unexpected file(s): " + ", ".join(unexpected)
        )

    for relative in sorted(expected):
        path = actual[relative]
        expected_entry = expected[relative]
        size = path.stat().st_size
        if size != expected_entry["sizeBytes"]:
            raise IntegrityError(
                f"size mismatch for {relative}: expected {expected_entry['sizeBytes']}, got {size}"
            )
        digest, observed_size = sha256_file(path)
        if observed_size != size:
            raise IntegrityError(f"file changed while verifying: {relative}")
        if digest != expected_entry["sha256"]:
            raise IntegrityError(f"SHA-256 mismatch for {relative}")


def main() -> int:
    args = parse_args()
    try:
        root = canonical_root(args.root)
        if args.command == "create":
            output = (args.output or (root / DEFAULT_MANIFEST_NAME)).resolve(
                strict=False
            )
            manifest = build_manifest(root, output)
            write_manifest(output, manifest)
            print(f"wrote {output} for {manifest['fileCount']} file(s)")
        else:
            manifest_path = (args.manifest or (root / DEFAULT_MANIFEST_NAME)).resolve(
                strict=False
            )
            manifest = load_manifest(manifest_path)
            verify_manifest(root, manifest_path, manifest)
            print(
                f"verified {manifest_path} for {manifest['fileCount']} file(s)"
            )
        return 0
    except (IntegrityError, OSError) as error:
        print(f"research-integrity: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
