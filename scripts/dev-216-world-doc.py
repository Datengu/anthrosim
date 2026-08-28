from pathlib import Path
p = Path('crates/anthrosim-core/src/world.rs')
s = p.read_text()
s = s.replace(
    'pub const INITIAL_FOOD_STOCK_MULTIPLIER: u32 = 10;\n',
    '/// Legacy synthetic-world stock marker multiplier. Authoritative M3 day-zero stock is declared\n/// by `ResourceConfig.initial_stock_units_per_productivity`; this constant remains in the frozen\n/// synthetic world representation and world digest for compatibility.\npub const INITIAL_FOOD_STOCK_MULTIPLIER: u32 = 10;\n',
    1,
)
s = s.replace(
    '    /// Abstract resource units available at initialization.\n    pub food_stock: u32,',
    '    /// Legacy synthetic stock marker retained in world identity/compatibility. Authoritative\n    /// dynamic M3 day-zero stock is derived from `ResourceConfig`, not from this field.\n    pub food_stock: u32,',
    1,
)
s = s.replace(
    '    /// exactly one value per world cell in row-major order. Productivity replacement also resets\n    /// the derived initial food stock to the same relationship used by synthetic world generation.',
    '    /// exactly one value per world cell in row-major order. Productivity replacement also resets\n    /// the legacy synthetic world stock marker. M3 dynamic starting stock remains independently\n    /// declared by `ResourceConfig.initial_stock_units_per_productivity`.',
    1,
)
p.write_text(s)
