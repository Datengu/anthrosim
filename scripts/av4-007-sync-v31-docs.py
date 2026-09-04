from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one target, found {count}")
    p.write_text(text.replace(old, new, 1))

for path in (
    "docs/scientific-model.md",
    "docs/research/odd.md",
    "docs/research/odd-d.md",
):
    replace_once(path, "current model semantics v30", "current model semantics v31")

replace_once(
    "scripts/test-current-model-semantics-docs.py",
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v30"\nCURRENT_SHORT = "v30"',
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v31"\nCURRENT_SHORT = "v31"',
)

replace_once(
    "docs/research/m9-equal-cost-destination-choice-v1.md",
    """When a household actually evaluates a trigger from a tied origin, AnthroSim chooses one candidate using the versioned keyed policy `m9/equal-cost-destination-keyed-v1`. The key contains the authoritative M9 tie seed, origin cell, household identity and trigger index and is passed through a fixed integer avalanche before reduction to the candidate count. Core runs use the experiment seed for this role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

This choice is deterministic and platform independent, but it consumes no sequential RNG draw. Therefore adding or removing a tied journey cannot shift M2, M3, M4 or other stochastic streams. Replaying the same experiment/household/trigger produces the same destination exactly.
""",
    """Model semantics v31 replaces the original canonical-`HouseholdId` component of that rule with the versioned policy `m9/equal-cost-destination-scientific-coupling-v2`. Authoritative execution keys a tied destination from the authoritative M9 tie seed, origin cell, trigger index and the household's scientific coupling key: the minimum persistent person `stochastic_coupling_rank` among its living members. Canonical `HouseholdId` is bookkeeping only and is not a causal input. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed from the spatial-realization provenance contract.

The policy identifier is bound into the travel-table/program identity, and authoritative tied-departure events record the scientific coupling key used so observability can independently verify the selected destination. A label-neutral compatibility resolver exists for callers that have only a canonical household ID; authoritative simulation execution never uses that ID as the tie key.

This choice is deterministic and platform independent, but it consumes no sequential RNG draw. Therefore adding or removing a tied journey cannot shift M2, M3, M4 or other stochastic streams. Replaying the same scientific household/trigger under the same program reproduces the same destination exactly.
""",
)
replace_once(
    "docs/research/m9-equal-cost-destination-choice-v1.md",
    """This changes authoritative M9 destination behavior and advances `MODEL_SEMANTICS_ID` from v17 to v18. It does not change travel-cost equations, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream.
""",
    """The original equal-minimum preservation advanced `MODEL_SEMANTICS_ID` from v17 to v18. Audit-v4 AV4-007 remediation advances the current line from v30 to v31 because future tied M9 destinations now use scientific household coupling identity rather than canonical `HouseholdId`. It does not change travel-cost equations, travel-duration conversion, M4 migration decisions, mortality, resource allocation rules, or any sequential RNG stream.
""",
)

replace_once(
    "docs/research/m9-temporary-travel-semantics-v1.md",
    """When a household actually evaluates a temporary-travel trigger from a tied origin, `TemporaryTravelTable::resolution_for(...)` chooses among the equal minima with the versioned keyed policy `m9/equal-cost-destination-keyed-v1`. The key is composed from:

- the authoritative M9 destination-tie seed stored with the travel table;
- the origin `CellId`;
- the household identity;
- the trigger index.

The keyed value is passed through the fixed integer avalanche defined by that policy and reduced to the candidate count. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed declared by the spatial-realization provenance contract.

This choice is deterministic and platform-independent, but it does **not** consume a mutable sequential RNG stream. Therefore an added, removed or reordered tied journey cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative tie seed, origin, household and trigger index reproduces the same destination exactly.
""",
    """When a household actually evaluates a temporary-travel trigger from a tied origin, authoritative execution chooses among the equal minima with the versioned policy `m9/equal-cost-destination-scientific-coupling-v2`. The key is composed from:

- the authoritative M9 destination-tie seed stored with the travel table;
- the origin `CellId`;
- the household scientific coupling key, defined as the minimum persistent person `stochastic_coupling_rank` among its living members;
- the trigger index.

Canonical `HouseholdId` is excluded from the causal key. The keyed value is passed through the fixed integer avalanche defined by that policy and reduced to the candidate count. Core runs use the experiment seed for the tie-seed role; spatial runs use the resolved process seed declared by the spatial-realization provenance contract. The tie-policy identifier is included in travel-table/program identity, and tied authoritative departure events record the coupling key used for downstream verification.

This choice is deterministic and platform-independent, but it does **not** consume a mutable sequential RNG stream. Therefore an added, removed or reordered tied journey cannot shift M2, M3, M4 or other sequential stochastic streams. Replaying the same authoritative tie seed, origin, scientific household coupling key and trigger index reproduces the same destination exactly.
""",
)
replace_once(
    "docs/research/m9-temporary-travel-semantics-v1.md",
    """- the selected destination implied by the keyed household/trigger resolution when a journey is evaluated;
""",
    """- the selected destination implied by the keyed scientific-household/trigger resolution when a journey is evaluated;
""",
)
replace_once(
    "docs/research/m9-temporary-travel-semantics-v1.md",
    """- deterministic keyed equal-cost destination resolution using the declared tie seed, origin, household and trigger index without consuming sequential RNG state;
""",
    """- deterministic keyed equal-cost destination resolution using the declared tie seed, origin, scientific household coupling key and trigger index without consuming sequential RNG state or canonical HouseholdId;
""",
)
