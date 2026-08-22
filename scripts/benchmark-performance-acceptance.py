#!/usr/bin/env python3
"""Reproducible M7.5 end-to-end performance acceptance measurement.

The harness runs the release CLI as an external process, measures wall/CPU/RSS,
reads the emitted immutable run manifest, derives throughput, writes a JSON
report, and optionally enforces deliberately broad CI acceptance floors.

This is an engineering benchmark. It does not validate anthropological model
assumptions or change simulation semantics.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import resource
import subprocess
import sys
import tempfile
import time
from pathlib import Path

DAYS_PER_YEAR = 365


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("target/release/anthrosim"))
    parser.add_argument("--report", type=Path)
    parser.add_argument("--years", type=int, default=2_000)
    parser.add_argument("--population", type=int, default=10_000)
    parser.add_argument("--world-width", type=int, default=64)
    parser.add_argument("--world-height", type=int, default=64)
    parser.add_argument("--seed", type=int, default=1_847_291)
    parser.add_argument("--max-person-records", type=int, default=1_000_000)
    parser.add_argument("--min-years-per-second", type=float, default=25.0)
    parser.add_argument("--max-wall-seconds", type=float, default=120.0)
    parser.add_argument("--max-rss-mib", type=float, default=1_024.0)
    parser.add_argument("--enforce", action="store_true")
    return parser.parse_args()


def rss_to_mib(raw_maxrss: int) -> float:
    # Linux reports ru_maxrss in KiB; macOS/BSD report bytes.
    bytes_used = raw_maxrss if sys.platform == "darwin" else raw_maxrss * 1024
    return bytes_used / (1024 * 1024)


def main() -> int:
    args = parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        raise SystemExit(f"benchmark binary not found: {binary}")

    with tempfile.TemporaryDirectory(prefix="anthrosim-m7-5-") as temp_dir:
        manifest_path = Path(temp_dir) / "manifest.json"
        command = [
            str(binary),
            "run",
            "--years",
            str(args.years),
            "--world-width",
            str(args.world_width),
            "--world-height",
            str(args.world_height),
            "--population",
            str(args.population),
            "--max-person-records",
            str(args.max_person_records),
            "--seed",
            str(args.seed),
            "--output",
            str(manifest_path),
        ]

        before = resource.getrusage(resource.RUSAGE_CHILDREN)
        started = time.perf_counter()
        completed = subprocess.run(command, check=False, text=True, capture_output=True)
        wall_seconds = time.perf_counter() - started
        after = resource.getrusage(resource.RUSAGE_CHILDREN)

        if completed.returncode != 0:
            sys.stderr.write(completed.stdout)
            sys.stderr.write(completed.stderr)
            return completed.returncode
        if not manifest_path.is_file():
            raise SystemExit("AnthroSim completed without writing the benchmark manifest")

        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    statistics = manifest["statistics"]
    population = manifest["population"]
    simulated_days = int(statistics["simulatedDays"])
    simulated_years = simulated_days / DAYS_PER_YEAR
    event_count = int(statistics["authoritativeEventCount"])
    metric_count = int(statistics["metricSnapshotCount"])
    max_rss_mib = rss_to_mib(after.ru_maxrss)
    cpu_seconds = max(
        0.0,
        (after.ru_utime + after.ru_stime) - (before.ru_utime + before.ru_stime),
    )
    years_per_second = simulated_years / wall_seconds if wall_seconds > 0 else float("inf")
    events_per_second = event_count / wall_seconds if wall_seconds > 0 else float("inf")
    person_records = int(population["personRecords"])
    living_population = int(population["livingPopulation"])
    rss_bytes = max_rss_mib * 1024 * 1024

    checks = {
        "process_exit_success": True,
        "duration_reached": manifest["stopReason"] == "duration_reached",
        "requested_years_reached": simulated_days == args.years * DAYS_PER_YEAR,
        "minimum_years_per_second": years_per_second >= args.min_years_per_second,
        "maximum_wall_seconds": wall_seconds <= args.max_wall_seconds,
        "maximum_rss_mib": max_rss_mib <= args.max_rss_mib,
    }

    report = {
        "schemaVersion": 1,
        "benchmarkId": "v0.1_10k_people_2000_year_end_to_end",
        "purpose": "engineering_performance_acceptance_not_scientific_validation",
        "environment": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "logicalCpuCount": os.cpu_count(),
            "pythonVersion": platform.python_version(),
        },
        "configuration": {
            "years": args.years,
            "population": args.population,
            "worldWidth": args.world_width,
            "worldHeight": args.world_height,
            "seed": args.seed,
            "maxPersonRecords": args.max_person_records,
        },
        "model": {
            "modelVersion": manifest["modelVersion"],
            "gitCommit": manifest.get("gitCommit"),
            "stopReason": manifest["stopReason"],
            "stateDigest64": manifest["stateDigest64"],
        },
        "result": {
            "simulatedDays": simulated_days,
            "simulatedYears": simulated_years,
            "wallSeconds": wall_seconds,
            "cpuSeconds": cpu_seconds,
            "simulatedYearsPerSecond": years_per_second,
            "authoritativeEventCount": event_count,
            "authoritativeEventsPerSecond": events_per_second,
            "metricSnapshotCount": metric_count,
            "maxResidentSetMiB": max_rss_mib,
            "finalPersonRecords": person_records,
            "finalLivingPopulation": living_population,
            "approxRssBytesPerPersonRecord": (rss_bytes / person_records) if person_records else None,
            "approxRssBytesPerLivingPerson": (rss_bytes / living_population) if living_population else None,
        },
        "acceptance": {
            "minYearsPerSecond": args.min_years_per_second,
            "maxWallSeconds": args.max_wall_seconds,
            "maxResidentSetMiB": args.max_rss_mib,
            "checks": checks,
            "passed": all(checks.values()),
        },
    }

    rendered = json.dumps(report, indent=2, sort_keys=True)
    print(rendered)
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(rendered + "\n", encoding="utf-8")

    if args.enforce and not report["acceptance"]["passed"]:
        failed = ", ".join(name for name, passed in checks.items() if not passed)
        print(f"M7.5 performance acceptance failed: {failed}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
