from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


# Roadmap: M9 remains in progress, but M9.6 is complete and only M9.7 remains.
path = ROOT / "docs/roadmap.md"
text = path.read_text()
text = replace_once(
    text,
    "**Status:** in progress. M9.0 has frozen the temporary-mobility scientific/software semantics; implementation proceeds through M9.1-M9.7. The authoritative M9 semantics contract is `docs/research/temporary-mobility-v1.md`; the capability audit that motivated it is `docs/research/m9-temporary-mobility-capability-audit.md`.",
    "**Status:** in progress. M9.0-M9.6 are complete; M9.7, the controlled aggregation benchmark, remains before milestone acceptance. The authoritative M9 semantics contract is `docs/research/temporary-mobility-v1.md`; the capability audit that motivated it is `docs/research/m9-temporary-mobility-capability-audit.md`.",
    "roadmap M9 status",
)
old = '''#### M9.6 — Temporary-mobility observability and experiment integration

Extend checkpoints, run bundles, events, invariants, ensemble/sweep execution and downstream spatial observability for the new process.

Machine-readable outputs should make it possible to distinguish at least:

- permanent residents;
- temporary visitors;
- arrivals and returns;
- visit counts and duration;
- peak and mean presence in a focal region;
- living person-days by residence/presence regime;
- journey distance/time/cost and origin catchment;
- permanent migration from temporary movement.
'''
new = '''#### M9.6 — Temporary-mobility observability and experiment integration — complete

M9 temporary mobility now participates in ordinary transformed-spatial execution, immutable experiment identity, run/ensemble/sweep inputs, checkpoint/resume, completed/paused artifact workflows and deterministic downstream observability.

The world-independent experiment definition preserves focal region, schedule and travel-model assumptions, while every run derives its resolved travel table from that run's own authoritative world. `anthrosim-temporary-observability` regenerates a separate machine-readable report from preserved authoritative artifacts rather than changing the meaning of M8 spatial observability.

The implemented report distinguishes:

- persistent residence from physical temporary presence;
- temporary visitors from focal-region residents;
- outbound and return transit without assigning transit to arbitrary cells;
- starts, explicit non-start outcomes, arrivals, return departures and completions;
- visit-duration distributions and peak/mean visitor presence;
- persistent-residence, at-residence, visitor and transit person-days with exact accounting identities;
- journey time/cost, derived route edge distance where it reconciles to M9.4 routing, and origin catchment;
- permanent M4 migration from temporary movement.

Completed bundles can carry the derived report and fail closed if it cannot be regenerated exactly. Paused runs with resume-boundary population provenance can reconstruct the day-zero founder state deterministically and derive/verify the same report. The read-only Explorer can surface the derived summary and M9 event family without changing residence maps or inventing transit locations.

See `docs/research/temporary-mobility-observability-v1.md` and `docs/research/m9-6-integration-audit.md`.
'''
text = replace_once(text, old, new, "roadmap M9.6 block")
path.write_text(text)


# Integration audit: replace the stale remaining-work section with completed downstream slices.
path = ROOT / "docs/research/m9-6-integration-audit.md"
text = path.read_text()
text = replace_once(
    text,
    "Status: M9.6 implementation in progress.",
    "Status: M9.6 capability implementation complete; protected final validation is required before closing the umbrella issue.",
    "audit status",
)
old = '''## Remaining work after the runner slice

M9.6 still requires the scientifically important downstream half of the capability:

- deterministic temporary-presence observability derived from authoritative event/checkpoint artifacts;
- explicit resident/visitor/transit accounting without assigning transit to arbitrary world cells;
- visit, arrival/return, duration, peak/mean presence, travel burden, origin-catchment and non-participation/unreachable observables;
- bundle validation and derived-analysis integration for the new machine-readable report;
- end-to-end M9.6 acceptance demonstrating exact regeneration for uninterrupted and resumed runs.

The temporary-mobility report should remain downstream/derived. It must not become a second authoritative movement state, and existing M4 permanent-migration observables must remain a separate category.
'''
new = '''## Fourth implementation slice: deterministic temporary-mobility observability

PR #137 establishes `TemporaryMobilityObservabilityReport` as a separate downstream artifact rather than changing M8 spatial-observability meaning. It replays authoritative event history household-by-household and reconciles the terminal result against the checkpoint.

The v1 report provides:

- exact persistent-residence and physical-presence person-day accounting;
- separate at-residence, visiting, outbound-transit and return-transit categories;
- trigger outcomes and explicit non-start reasons, including unreachable households;
- starts, arrivals, return departures, completions and journeys active at the report boundary;
- visit-duration distributions, visitor household/person-days and peak/mean visitor presence;
- travel duration/cost and origin catchment;
- derived route edge distance only when deterministic minimum-cost recomputation agrees with the authoritative M9.4 cost and destination.

Transit remains deliberately non-spatial because M9 v1 does not contain an authoritative per-day en-route cell. The resource subsystem's home-provisioning proxy for transit therefore does not become a claim of physical residence occupancy.

Focused acceptance proves exact physical person-day partitioning and exact report equality between uninterrupted execution and execution resumed from a checkpoint with an active temporary journey. Resume-lineage provenance may differ; the scientific state and derived report must not.

## Fifth implementation slice: report CLI, bundle validation and Explorer

PR #138 integrates the established report into ordinary downstream workflows:

- `anthrosim-temporary-observability run` derives or verifies one run;
- `anthrosim-temporary-observability tree` discovers and processes M9 runs beneath experiment/sweep roots;
- completed bundle validation treats `temporary-observability.json` as an optional derived artifact and requires exact deterministic regeneration when it is present;
- the existing deterministic bundle packer includes the report automatically through the normal optional-artifact path;
- a paused-run regression proves a report can be derived and verified when only `resume-start-population.json` is preserved, by validating that artifact and deterministically reconstructing day-zero founders from immutable experiment identity and the authoritative world;
- the read-only Explorer optionally surfaces the derived M9 summary, identities and authoritative temporary-event family while leaving its residence map semantics unchanged.

Explorer support uses the machine-readable report rather than independently re-deriving scientific quantities in JavaScript. It validates basic provenance and exact person-day identities for display, while Rust bundle/report validation remains the authority for deterministic regeneration.

## M9.6 acceptance reconciliation

The umbrella acceptance criteria are satisfied by the combined M9.3-M9.6 state and the M9.6 slices above:

- temporary transition events are separately versioned and remain distinct from `HouseholdMigration`;
- active journeys, resolved program, resource ledger and RNG/state continuity survive checkpoints and state digests;
- core and transformed-spatial uninterrupted/resumed execution reconcile exactly at the scientific-state boundary;
- ordinary run, ensemble and sweep identity preserve the immutable world-independent M9 definition, with routing re-derived per world;
- completed bundles validate the optional derived report and reject tampering;
- paused runs regenerate/verify the report from preserved provenance;
- resident/visitor/transit observables, journey outcomes, duration, peaks/means, travel burden, catchment and non-participation are machine-readable;
- transit is not assigned to arbitrary cells;
- M4 permanent-migration observables remain a separate category;
- read-only Explorer support is downstream from the established machine-readable semantics.

M9.6 therefore adds inspectability and experiment integration without creating an archaeological observation/preservation model or assigning a social motive to temporary aggregation.
'''
text = replace_once(text, old, new, "audit downstream completion block")
text = replace_once(
    text,
    "The first three M9.6 slices do not introduce a new temporary-mobility mechanism. They expose the existing M9 v1 semantics through the transformed-landscape host, immutable experiment/provenance machinery and ordinary runners using the same authoritative ordering, travel model, state and event vocabulary already represented by `anthrosim-model-semantics-v5`.",
    "The M9.6 slices do not introduce a new temporary-mobility mechanism. They expose the existing M9 v1 semantics through the transformed-landscape host, immutable experiment/provenance machinery, ordinary runners and downstream derived observability using the same authoritative ordering, travel model, state and event vocabulary already represented by `anthrosim-model-semantics-v5`.",
    "audit identity scope",
)
text = replace_once(
    text,
    "Accordingly, these slices do not by themselves require another `MODEL_SEMANTICS_ID` change. They also do not change the M8 landscape-to-model transformation semantics, so `SPATIAL_MODEL_SEMANTICS_ID` remains unchanged. The package version remains on the current released line during ordinary M9 implementation.\n\nThat identity decision must be reviewed again if later M9.6 work changes authoritative simulated meaning rather than experiment packaging, validation or downstream observability only.",
    "Accordingly, M9.6 does not require another `MODEL_SEMANTICS_ID` change. It also does not change the M8 landscape-to-model transformation semantics, so `SPATIAL_MODEL_SEMANTICS_ID` remains unchanged. The package version remains on the current released line during ordinary M9 implementation. M9.7 must review scientific identity again if its benchmark work reveals an authoritative semantic change rather than merely exercising the completed M9 capability.",
    "audit final identity decision",
)
path.write_text(text)

print("updated M9.6 roadmap and integration audit for closure")
