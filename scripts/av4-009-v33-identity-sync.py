from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f'{path}: expected exactly one target, found {count}: {old[:100]}')
    p.write_text(text.replace(old, new, 1))

replace_once(
    'crates/anthrosim-core/src/provenance.rs',
    '/// silently changing which represented household receives an indivisible scarce-resource unit.\npub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v32";',
    '''/// silently changing which represented household receives an indivisible scarce-resource unit.\n///\n/// v33 removes canonical spatial candidate order from M4 uncertainty and proportional-choice\n/// assignment. Deterministically eligible candidates are coupled by their active deterministic\n/// M4 utility and movement distance rather than CellId/container position; exact scientifically\n/// indistinguishable eligible destination orbits are left unresolved instead of inventing an\n/// unmodelled orientation. A v32 checkpoint must therefore not resume under v33 with unchanged\n/// migration RNG positions while silently reassigning candidate uncertainty/choice draws.\npub const MODEL_SEMANTICS_ID: &str = "anthrosim-model-semantics-v33";''',
)

replace_once(
    'crates/anthrosim-core/src/checkpoint.rs',
    '    pub const PRE_RESOURCE_REMAINDER_COUPLING_SCHEMA_VERSION: u32 = 19;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 20;',
    '    pub const PRE_RESOURCE_REMAINDER_COUPLING_SCHEMA_VERSION: u32 = 19;\n    pub const PRE_M4_SPATIAL_CANDIDATE_COUPLING_SCHEMA_VERSION: u32 = 20;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 21;',
)

for path in (
    'docs/scientific-model.md',
    'docs/research/odd.md',
    'docs/research/odd-d.md',
):
    replace_once(path, 'current model semantics v32', 'current model semantics v33')

replace_once(
    'scripts/test-current-model-semantics-docs.py',
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v32"\nCURRENT_SHORT = "v32"',
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v33"\nCURRENT_SHORT = "v33"',
)
