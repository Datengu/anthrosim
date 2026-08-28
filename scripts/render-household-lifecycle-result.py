import json
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
data = json.loads(source.read_text(encoding="utf-8"))
b = data["baseline"]
f = data["deterministicSizeFission"]
n = len(data["seeds"])

def mean(total):
    return total / n

lines = [
    "# Household lifecycle structural sensitivity — first result",
    "",
    "**Scientific status:** synthetic structural sensitivity; not empirical household validation.",
    "",
    f"Eight paired seeds were run for {data['durationYears']} years. The arms differ only in household lifecycle: `fixed_founder_v1` versus `deterministic_size_fission_v1` with a maximum of {data['alternativeMaxLivingMembers']} living members per eligible household after an annual boundary.",
    "",
    "| Observable | Fixed founder | Size fission |",
    "| --- | ---: | ---: |",
    f"| Completed runs | {b['completedRuns']}/{n} | {f['completedRuns']}/{n} |",
    f"| Extinct runs | {b['populationExtinctRuns']}/{n} | {f['populationExtinctRuns']}/{n} |",
    f"| Mean terminal living population | {mean(b['terminalLivingPopulationTotal']):.2f} | {mean(f['terminalLivingPopulationTotal']):.2f} |",
    f"| Mean terminal active households | {mean(b['terminalActiveHouseholdsTotal']):.2f} | {mean(f['terminalActiveHouseholdsTotal']):.2f} |",
    f"| Mean terminal largest household | {mean(b['terminalLargestHouseholdSizeTotal']):.2f} | {mean(f['terminalLargestHouseholdSizeTotal']):.2f} |",
    f"| Mean terminal multi-generational households | {mean(b['terminalMultigenerationalHouseholdsTotal']):.2f} | {mean(f['terminalMultigenerationalHouseholdsTotal']):.2f} |",
    f"| Mean terminal occupied residence cells | {mean(b['terminalLivingOccupiedCellsTotal']):.2f} | {mean(f['terminalLivingOccupiedCellsTotal']):.2f} |",
    f"| Total unmet resource need | {b['unmetNeedTotal']} | {f['unmetNeedTotal']} |",
    f"| Total M4 moves | {b['migrationMovesTotal']} | {f['migrationMovesTotal']} |",
    f"| Mean people per M4 move | {(b['migrationPeopleMovedTotal'] / b['migrationMovesTotal']) if b['migrationMovesTotal'] else 0:.3f} | {(f['migrationPeopleMovedTotal'] / f['migrationMovesTotal']) if f['migrationMovesTotal'] else 0:.3f} |",
    f"| Total M9 departures | {b['temporaryDeparturesTotal']} | {f['temporaryDeparturesTotal']} |",
    "",
    "## Interpretation",
    "",
]
materially_different = (
    b["terminalActiveHouseholdsTotal"] != f["terminalActiveHouseholdsTotal"]
    or b["migrationMovesTotal"] != f["migrationMovesTotal"]
    or b["temporaryDeparturesTotal"] != f["temporaryDeparturesTotal"]
    or b["unmetNeedTotal"] != f["unmetNeedTotal"]
)
if materially_different:
    lines.append(
        "The declared lifecycle contrast is **material for at least one predeclared household/resource/mobility observable** in this synthetic ensemble. Household lifecycle must therefore remain an explicit structural uncertainty dimension for claims that depend on household sharing, M4 permanent migration, or M9 participation. This does not establish which lifecycle is historically correct."
    )
else:
    lines.append(
        "The declared lifecycle contrast did not alter the predeclared aggregate observables in this synthetic ensemble. That is robustness evidence only for this exact contrast and does not validate either lifecycle historically."
    )
lines.extend([
    "",
    "The fixed-founder arm's household ages are exactly the run duration by construction. Its size and generation-span distributions can be regenerated from each checkpoint with `anthrosim-household-observability`; the alternative removes that permanent founder-topology assumption and creates younger household records at annual fission boundaries.",
    "",
    "The machine-readable aggregate used for this page is `research/household-lifecycle-sensitivity-v1/reference-result.json`.",
])
target.write_text("\n".join(lines) + "\n", encoding="utf-8")
