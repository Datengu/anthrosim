# Audit v5 Area A — M9 return completion × coincident M4 boundary adversary

Target: immutable `v0.3.5` / `e7667af52d48a1ffbae2bf7713a2388e65994b42` / `anthrosim-model-semantics-v33`.

This is fresh Audit-v5 Area-A evidence for **same-day state read/write ordering**. It complements, but does not replay, the permanent departure-side ordering regression already present in the v0.3.5 suite.

## Hypothesis

The declared scheduler applies due temporary-mobility transitions before M4 and excludes a household from permanent migration while it has an active temporary journey. The converse boundary is scientifically important: if a return completes exactly on an M4 decision boundary, the completed household should be visible as resident and M4-eligible on that same boundary.

If M4 instead observes the stale pre-completion state, the duration of temporary-mobility exclusion acquires a hidden one-boundary lag. That can alter permanent-relocation opportunities even when journey timing is otherwise identical.

## Controlled construction

- one-year core simulation, 16×16 world, 20 people in target household size 5;
- mortality and fertility disabled;
- resource need and condition/scarcity mortality disabled;
- ordinary synthetic-validation M4 retained;
- one synthetic unoccupied focal destination;
- every residential origin has a zero-day outbound and zero-day return path;
- one day-0 temporary departure, stay duration exactly 182 days.

Under the declared half-open journey timing:

- departure/arrival: day 0;
- household remains visiting through the first M4 boundary (day 91);
- return departure and completion: day 182, exactly the second M4 boundary;
- later M4 boundaries: days 273 and 365.

The baseline without M9 must evaluate every household at all four M4 boundaries. The M9 arm must evaluate none on day 91, then all households on days 182, 273 and 365, for exactly `3 × household_count` evaluations.

The adversary also requires one day-182 `TemporaryJourneyCompleted` event per household. If day-182 permanent migrations occur, every completion event must precede the first migration event in authoritative event sequence. Full run invariants must pass.

## Interpretation

A pass falsifies this specific stale-state/order hypothesis and supplies fresh v5 evidence that return completion is visible to coincident M4 evaluation. It does not establish all M9/M3/M4/M2 collision semantics and does not erase AV5-001.

A failure at the scientific assertions requires classification after excluding harness/configuration errors. The evidence PR is not production code and is intended to close unmerged after the result is recorded.
