#!/usr/bin/env python3
import importlib.util
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("aggregate-m8-spatial-benchmark.py")
spec = importlib.util.spec_from_file_location("aggregate_m8_spatial_benchmark", MODULE_PATH)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)


def record(value: int, degenerate: bool = False):
    return {
        "terminalDegenerate": degenerate,
        "metrics": {"migrationTotalDistanceCells": value},
    }


def arm(values, degenerate_seed=None):
    return {
        "runs": {
            str(seed): record(value, seed == degenerate_seed)
            for seed, value in zip(module.SEEDS, values)
        }
    }

flat = arm([10] * 8)
treated = arm([20] * 8, degenerate_seed=module.SEEDS[0])
stats = module.pair_stats(flat, treated, "migrationTotalDistanceCells")
assert stats["availablePairs"] == 7, stats
assert stats["unavailablePairs"] == 1, stats
first = stats["pairs"][0]
assert first["available"] is False, first
assert "did not reach the declared duration" in first["reason"], first

all_duration = arm([20] * 8)
stats = module.pair_stats(flat, all_duration, "migrationTotalDistanceCells")
assert stats["availablePairs"] == 8, stats
assert stats["unavailablePairs"] == 0, stats
print("issue 226 M8 fixed-horizon tests passed")
