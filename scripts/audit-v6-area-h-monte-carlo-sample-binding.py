#!/usr/bin/env python3
"""Audit-v6 Area-H adversary for confirmatory Monte Carlo sample-value binding.

Build a real finalized 30-seed study, prove the canonical research run table says every
run completed, then feed the official confirmatory Monte Carlo gate the same exact seeds
with values claiming zero completions. Execute that contradictory sample through the
canonical v2 analysis-provenance wrapper, verify it, and require isolated replay.

The final assertion is intentionally a scientific oracle: a confirmatory precision result
must not be able to contradict the authoritative study output from which its per-seed
estimand is purportedly derived.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE_DEFINITION = ROOT / "research" / "general-demography-baseline-v1" / "confirmatory-definition.json"
BASE_PROTOCOL = ROOT / "examples" / "study-protocol-v1.json"
ENGINE = ROOT / "scripts" / "research-monte-carlo-sufficiency.py"
CONFIRMATORY = ROOT / "scripts" / "research-monte-carlo-confirmatory.py"
PROVENANCE = ROOT / "scripts" / "research-analysis-provenance.py"

MC_IMPLEMENTATION_FILES = [
    "research-monte-carlo-confirmatory.py",
    "research-monte-carlo-sufficiency.py",
    "research-monte-carlo-sufficiency-legacy.py",
    "research-study-result-binding.py",
    "research-study-result-binding-legacy.py",
]


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run(command: list[str], role: str, *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    if completed.returncode != 0:
        raise AssertionError(
            f"{role} failed with status {completed.returncode}\n"
            f"stdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    return completed


def tiny_definition(seeds: list[int]) -> dict:
    definition = read_json(BASE_DEFINITION)
    definition["seeds"] = seeds
    experiment = definition["base"]["experiment"]
    experiment["seed"] = seeds[0]
    experiment["durationYears"] = 1
    experiment["population"]["initialPopulation"] = 20
    experiment["population"]["maxPersonRecords"] = 10_000
    experiment["world"] = {"schemaVersion": 1, "width": 4, "height": 4}
    definition["base"].pop("spatial", None)
    definition["dimensions"] = []
    return definition


def precision_plan(seeds: list[int]) -> dict:
    return {
        "schemaVersion": 1,
        "planIdentity": "",
        "planId": "audit-v6-area-h-completion-probability",
        "uncertaintyCategory": "process_stochastic_monte_carlo",
        "estimand": {
            "kind": "probability",
            "confidenceLevel": 0.95,
            "maxHalfWidth": 0.2,
        },
        "design": {"mode": "fixed", "seedBatches": [seeds]},
        "pairing": "independent",
        "rationale": (
            "Audit-only controlled probability estimand. The authoritative value is one "
            "iff the canonical research run row is completed."
        ),
    }


def protocol(plan_identity: str) -> dict:
    value = read_json(BASE_PROTOCOL)
    value["studyId"] = "audit-v6-area-h-monte-carlo-sample-binding"
    value["researchQuestion"] = (
        "Can a confirmatory per-seed Monte Carlo probability be proven to come from the "
        "authoritative frozen study outputs?"
    )
    value["applicabilityDomain"] = "Audit-only synthetic executable provenance adversary."
    value["hypotheses"] = [
        {
            "id": "authoritative_completion",
            "kind": "null_model",
            "statement": "The per-seed sample agrees with canonical run completion state.",
        },
        {
            "id": "contradictory_sample",
            "kind": "alternative",
            "statement": "A contradictory free-form sample is accepted despite canonical run state.",
        },
    ]
    value["analysisWindows"] = [
        {
            "id": "primary_window",
            "analysisStartDay": 0,
            "selectionRule": "initial_state_in_scope",
            "rationale": "The audit estimand is the terminal canonical run completion state.",
        }
    ]
    value["observables"] = [
        {
            "id": "run_completed_probability",
            "role": "primary",
            "source": "research.analysis.runs.state",
            "analysisWindowId": "primary_window",
            "interpretation": "Per-seed Bernoulli value is one exactly when the authoritative run row state is completed.",
        }
    ]
    value["comparisons"] = [
        {
            "id": "sample_binding_check",
            "hypothesisIds": ["authoritative_completion", "contradictory_sample"],
            "observableIds": ["run_completed_probability"],
            "prediction": "The confirmatory Monte Carlo sample must agree seed-wise with canonical run state.",
            "decisionCriterion": "Reject a per-seed sample that contradicts the frozen study output.",
        }
    ]
    value["uncertainty"] = {"parameterUncertainty": [], "structuralUncertainty": []}
    value["ensemblePolicy"] = {
        "seedPolicy": "Use exactly the ordered seeds in the frozen ResearchExperimentDefinition.",
        "pairingPolicy": "One Bernoulli completion observation per declared process seed.",
        "replicationPolicy": "monte-carlo-precision-plan-v1:" + plan_identity,
    }
    value["runHandling"] = {
        "stoppingRules": ["Execute all predeclared seeds."],
        "exclusionRules": ["No post-hoc seed exclusions."],
        "censoringRules": ["Canonical non-completed run state remains a zero observation for this audit-only estimand."],
    }
    value["sensitivityPlan"] = ["No sensitivity extension: this is a semantic provenance adversary."]
    value["equifinalityPlan"] = ["Not applicable to this audit-only semantic binding check."]
    value["manipulationChecks"] = [
        {
            "id": "canonical_state_read",
            "mechanism": "research analysis run-state table",
            "criterion": "All planned seed rows are present and their state is read directly before sample construction.",
            "failureHandling": "Abort the adversary if the authoritative state cannot be established.",
        }
    ]
    value["analysisMethod"] = (
        "Use the frozen precision plan and the official confirmatory Monte Carlo entry point; "
        "the per-seed Bernoulli value is defined by canonical research run state."
    )
    value["multiplicityPolicy"] = "One audit-only primary probability estimand."
    value["heldOutCorroboration"] = []
    value["permittedInterpretations"] = ["Research-governance sample-value binding only."]
    value["prohibitedInterpretations"] = ["Empirical or archaeological inference."]
    return value


def samples(seeds: list[int], value: int) -> dict:
    return {
        "schemaVersion": 1,
        "groups": [
            {
                "id": "run_completed_probability",
                "replicates": [{"seed": seed, "value": value} for seed in seeds],
            }
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--study-binary", type=Path, required=True)
    parser.add_argument("--research-binary", type=Path, required=True)
    args = parser.parse_args()

    study_binary = args.study_binary.resolve(strict=True)
    research_binary = args.research_binary.resolve(strict=True)
    seeds = list(range(8_601, 8_631))

    with tempfile.TemporaryDirectory(prefix="anthrosim-av6-area-h-sample-binding-") as temporary:
        work = Path(temporary)
        study_dir = work / "study"
        definition_source = work / "definition.json"
        protocol_source = work / "protocol.json"
        plan_source = work / "plan.json"

        plan = precision_plan(seeds)
        write_json(plan_source, plan)
        identity = run([sys.executable, str(ENGINE), "identity", str(plan_source)], "precision plan identity").stdout.strip()
        assert identity.startswith("monte-carlo-precision-plan-v1-")
        plan["planIdentity"] = identity
        write_json(plan_source, plan)
        run([sys.executable, str(ENGINE), "validate-plan", str(plan_source)], "precision plan validation")

        write_json(definition_source, tiny_definition(seeds))
        write_json(protocol_source, protocol(identity))

        run(
            [
                str(study_binary),
                "prepare",
                "--protocol",
                str(protocol_source),
                "--definition",
                str(definition_source),
                "--study-dir",
                str(study_dir),
            ],
            "anthrosim-study prepare",
        )
        run(
            [
                str(research_binary),
                "--definition",
                str(study_dir / "research-definition.json"),
                "--run-dir",
                str(study_dir / "research"),
            ],
            "anthrosim-research execution",
        )
        run([str(study_binary), "finalize", "--study-dir", str(study_dir)], "anthrosim-study finalize")

        runs = read_json(study_dir / "research" / "analysis" / "runs.json")
        rows = runs["runs"]
        assert len(rows) == len(seeds), f"expected {len(seeds)} authoritative run rows, got {len(rows)}"
        by_seed = {int(row["seed"]): row for row in rows}
        assert set(by_seed) == set(seeds), "authoritative run rows do not match the exact planned seeds"
        assert all(by_seed[seed]["state"] == "completed" for seed in seeds), (
            "positive control requires every authoritative run to be completed"
        )

        analysis_dir = study_dir / "analysis"
        analysis_dir.mkdir(parents=True, exist_ok=True)
        plan_path = analysis_dir / "plan.json"
        authoritative_samples_path = analysis_dir / "authoritative-samples.json"
        contradictory_samples_path = analysis_dir / "samples.json"
        authoritative_output = work / "authoritative-diagnostic.json"
        diagnostic_path = analysis_dir / "diagnostic.json"
        write_json(plan_path, plan)
        write_json(authoritative_samples_path, samples(seeds, 1))
        write_json(contradictory_samples_path, samples(seeds, 0))

        authoritative = run(
            [
                sys.executable,
                str(CONFIRMATORY),
                str(plan_path),
                str(authoritative_samples_path),
                str(authoritative_output),
                "--study-dir",
                str(study_dir),
            ],
            "authoritative confirmatory control",
        )
        del authoritative
        authoritative_diagnostic = read_json(authoritative_output)
        assert authoritative_diagnostic["seedIdentities"] == seeds
        assert authoritative_diagnostic["precision"]["estimate"] == 1.0
        assert authoritative_diagnostic["decision"] == "sufficient_stop"

        for name in MC_IMPLEMENTATION_FILES:
            shutil.copy2(ROOT / "scripts" / name, analysis_dir / name)
        (analysis_dir / "environment.lock").write_text(
            f"python={sys.executable}\nstdlib-only=true\n", encoding="utf-8"
        )

        definition_path = analysis_dir / "definition.json"
        implementation = [
            {"path": f"analysis/{name}", "role": "monte-carlo-confirmatory-implementation"}
            for name in MC_IMPLEMENTATION_FILES
        ]
        write_json(
            definition_path,
            {
                "schemaVersion": 2,
                "definitionType": "anthrosim-analysis-definition",
                "analysisId": "audit-v6-area-h-contradictory-completion-sample",
                "analysisStatus": "confirmatory",
                "executionMode": "scripted",
                "workingDirectory": ".",
                "command": [
                    sys.executable,
                    "analysis/research-monte-carlo-confirmatory.py",
                    "analysis/plan.json",
                    "analysis/samples.json",
                    "analysis/diagnostic.json",
                    "--study-dir",
                    ".",
                ],
                "annotations": {
                    "auditOracle": "per-seed values must agree with research/analysis/runs.json state",
                    "authoritativeCompletedRuns": len(seeds),
                    "submittedCompletedValues": 0,
                },
                "runtimeDescription": "Python standard library; interpreter and implementation bytes are provenance-bound.",
                "reproductionCriterion": "exact_output_bytes",
                "inputs": [
                    {"path": "analysis/plan.json", "role": "frozen-monte-carlo-precision-plan"},
                    {"path": "analysis/samples.json", "role": "submitted-per-seed-estimand-sample"},
                    {"path": "research/analysis/runs.json", "role": "authoritative-research-run-table"},
                    {"path": "study-protocol.json", "role": "frozen-study-protocol"},
                    {"path": "research-definition.json", "role": "frozen-research-definition"},
                ],
                "implementation": implementation,
                "environment": [
                    {"path": "analysis/environment.lock", "role": "analysis-environment-lock"}
                ],
                "outputs": [
                    {"path": "analysis/diagnostic.json", "role": "canonical-monte-carlo-precision-diagnostic"}
                ],
                "manualSteps": [],
            },
        )

        provenance_run = run(
            [sys.executable, str(PROVENANCE), "run", str(study_dir), str(definition_path)],
            "analysis provenance run",
        )
        provenance_lines = [line.strip() for line in provenance_run.stdout.splitlines() if line.strip()]
        assert provenance_lines, "analysis provenance run produced no stdout"
        provenance_identity = provenance_lines[-1]
        assert provenance_identity.startswith("analysis-provenance-v2-sha256-")
        run([sys.executable, str(PROVENANCE), "verify", str(study_dir)], "analysis provenance verify")
        run([sys.executable, str(PROVENANCE), "replay", str(study_dir)], "analysis provenance replay")

        contradictory_diagnostic = read_json(diagnostic_path)
        provenance = read_json(analysis_dir / "analysis-provenance.json")
        assert contradictory_diagnostic["seedIdentities"] == seeds
        assert contradictory_diagnostic["precision"]["estimate"] == 0.0
        assert contradictory_diagnostic["decision"] == "sufficient_stop"
        assert provenance["executionStatus"] == "executed_by_wrapper"
        assert provenance["study"]["boundBeforeExecution"] is True

        print(f"authoritative_run_rows={len(rows)}")
        print(f"authoritative_completed_rows={sum(1 for row in rows if row['state'] == 'completed')}")
        print(f"authoritative_sample_estimate={authoritative_diagnostic['precision']['estimate']:.12f}")
        print(f"authoritative_sample_decision={authoritative_diagnostic['decision']}")
        print(f"contradictory_sample_estimate={contradictory_diagnostic['precision']['estimate']:.12f}")
        print(f"contradictory_sample_decision={contradictory_diagnostic['decision']}")
        print(f"contradictory_sample_seed_count={len(contradictory_diagnostic['seedIdentities'])}")
        print(f"provenance_identity={provenance_identity}")
        print(f"provenance_execution_status={provenance['executionStatus']}")
        print("analysis_provenance_verify=pass")
        print("analysis_provenance_replay=pass")

        raise AssertionError(
            "predeclared sample-binding oracle failed: a fully finalized confirmatory study has "
            "30/30 authoritative completed runs, yet the canonical provenance-bound Monte Carlo "
            "analysis accepted the same exact seed identities with 0/30 completion values, "
            "returned sufficient_stop at estimate 0.0, verified, and replayed exactly"
        )


if __name__ == "__main__":
    raise SystemExit(main())
