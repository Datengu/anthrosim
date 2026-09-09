#!/usr/bin/env python3
"""Small audit-only wrapper around the production finalized-study root verifier."""

from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINDING_PATH = ROOT / "scripts" / "research-study-result-binding.py"


def load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


binding = load_module(BINDING_PATH, "audit_v6_area_k_cross_root_verifier")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("study_root", type=Path)
    args = parser.parse_args()
    try:
        context = binding.validate_study_root(args.study_root.resolve(strict=True))
    except binding.StudyBindingError as error:
        print("accepted=false")
        print(f"error={error}")
        return 3
    print("accepted=true")
    print(f"study_id={context['binding']['studyId']}")
    print(f"protocol_identity={context['binding']['protocolIdentity']}")
    print(f"research_id={context['binding']['researchId']}")
    print(f"result_identity={context['binding']['resultIdentity']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
