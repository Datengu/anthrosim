# M9.6 temporary observability implementation slice

This slice implements the downstream replay contract in `temporary-mobility-observability-v1.md` without changing authoritative simulation semantics.

The core report is deliberately separate from M8 spatial observability. It reconstructs household residence, living-member counts and temporary presence from authoritative events, accrues half-open person-day intervals, and reconciles the result against the terminal checkpoint.

Acceptance requires exact physical person-day partitioning, persistent-residence accounting, non-spatial transit, machine-readable journey/trigger outcomes, visitor peaks/means, duration bins, origin catchments, travel burden, derived route-edge distance reconciled to authoritative M9.4 routing, and exact uninterrupted/resumed regeneration.

This slice is downstream only. It does not change `MODEL_SEMANTICS_ID`, M8 spatial transformation semantics or the package version.
