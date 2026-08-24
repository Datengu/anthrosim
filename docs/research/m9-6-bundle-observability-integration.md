# M9.6 bundle observability integration

This slice makes temporary-mobility observability consumable through the ordinary artifact workflow after the core replay schema is established.

It adds a dedicated downstream CLI that derives or verifies `temporary-observability.json` for one run or a nested experiment/sweep tree, using only preserved authoritative artifacts and deterministic founder reconstruction where a resumed bundle lacks `initial-population.json`.

Completed bundle validation treats the report as an optional derived artifact. If present, its provenance must match the checkpoint/world/program and deterministic regeneration must be exactly equal. The existing bundle packer therefore includes the report automatically without inventing an M9-specific archive format.

This work is downstream analysis/validation only and does not change authoritative model semantics or package version.
