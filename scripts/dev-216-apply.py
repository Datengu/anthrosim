from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1))

# ResourceConfig: make initial stock independent from storage capacity.
replace(
    "crates/anthrosim-core/src/config.rs",
    "    pub annual_regeneration_units_per_productivity: u32,\n    pub productivity_scale_permille: u16,",
    "    pub annual_regeneration_units_per_productivity: u32,\n    /// Day-zero stock units per cell productivity unit before the ordinary productivity scale.\n    /// This is an explicit initial-condition assumption, independent of storage capacity. Capacity\n    /// may cap an impossible starting stock but increasing capacity does not create historical stock.\n    pub initial_stock_units_per_productivity: u32,\n    pub productivity_scale_permille: u16,",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    /// Legacy wire name retained for input compatibility. Under schema v4 this is the maximum",
    "    /// Legacy wire name retained for input compatibility. Under schema v5 this is the maximum",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    /// Legacy wire name retained for input compatibility. Under schema v4 this is the maximum",
    "    /// Legacy wire name retained for input compatibility. Under schema v5 this is the maximum",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 4;",
    "    pub const CURRENT_SCHEMA_VERSION: u32 = 5;",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "            annual_regeneration_units_per_productivity: 1,\n            productivity_scale_permille: 1_000,",
    "            annual_regeneration_units_per_productivity: 1,\n            initial_stock_units_per_productivity: 10,\n            productivity_scale_permille: 1_000,",
)
replace(
    "crates/anthrosim-core/src/config.rs",
    "    pub const fn with_productivity_scale_permille(mut self, value: u16) -> Self {\n        self.productivity_scale_permille = value;\n        self\n    }",
    "    pub const fn with_initial_stock_units_per_productivity(mut self, value: u32) -> Self {\n        self.initial_stock_units_per_productivity = value;\n        self\n    }\n\n    #[must_use]\n    pub const fn with_productivity_scale_permille(mut self, value: u16) -> Self {\n        self.productivity_scale_permille = value;\n        self\n    }",
)

# Resource initialization: derive day-zero stock from the explicit resource assumption, not World.food_stock.
replace(
    "crates/anthrosim-core/src/resources.rs",
    "        for cell in world.cells() {\n            let scaled_initial = scale_permille(\n                u64::from(cell.food_stock),\n                config.productivity_scale_permille,\n            );\n            let capacity = cell_capacity(cell.base_productivity, config);",
    "        for cell in world.cells() {\n            let configured_initial = u64::from(cell.base_productivity)\n                .checked_mul(u64::from(config.initial_stock_units_per_productivity))\n                .ok_or(ResourceError::AccountingOverflow)?;\n            let scaled_initial =\n                scale_permille(configured_initial, config.productivity_scale_permille);\n            let capacity = cell_capacity(cell.base_productivity, config);",
)

# This adds a new causal initialization degree of freedom, so checkpoint continuation semantics advance.
replace(
    "crates/anthrosim-core/src/provenance.rs",
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v15";',
    'pub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v16";',
)
