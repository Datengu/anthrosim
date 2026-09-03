#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:80]!r}")
    p.write_text(text.replace(old, new, 1), encoding="utf-8")


# M4 decision schedule: keep one shared pre-move snapshot, but assign sequential
# migration RNG draws by a HouseholdId-independent key derived from persisted person
# coupling identities.
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "/// Decisions are evaluated in stable household-ID order against one shared\n/// pre-move snapshot. Selected household moves are then applied simultaneously\n/// in a packed population pass. This prevents earlier households in a boundary\n/// from changing the information seen by later households.",
    "/// Decisions are evaluated against one shared pre-move snapshot in a schedule derived from\n/// persistent person-level stochastic coupling identities rather than arbitrary `HouseholdId`\n/// order. Selected household moves are then applied simultaneously in a packed population pass.\n/// This preserves simultaneous information while making sequential migration RNG assignment\n/// invariant to pure household relabelling.",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "    kin_locations: Vec<Vec<CellId>>,\n    planned_destinations: Vec<CellId>,",
    "    kin_locations: Vec<Vec<CellId>>,\n    /// Household decision key: minimum persistent stochastic-coupling rank among living members.\n    /// Person ranks are globally unique causal identities, so non-empty households have unique\n    /// keys without introducing a second persisted household identity.\n    household_coupling_keys: Vec<u64>,\n    /// Reusable HouseholdId-index schedule sorted by `household_coupling_keys` at each boundary.\n    decision_order: Vec<usize>,\n    planned_destinations: Vec<CellId>,",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "            kin_locations: vec![Vec::new(); households],\n            planned_destinations: vec![CellId::INVALID; households],",
    "            kin_locations: vec![Vec::new(); households],\n            household_coupling_keys: vec![u64::MAX; households],\n            decision_order: Vec::with_capacity(households),\n            planned_destinations: vec![CellId::INVALID; households],",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "        for household_index in 0..population.household_count() {\n            let members = self.living_members[household_index];",
    "        self.decision_order.clear();\n        self.decision_order.extend(\n            (0..population.household_count())\n                .filter(|&household_index| self.living_members[household_index] > 0),\n        );\n        self.decision_order\n            .sort_unstable_by_key(|&household_index| self.household_coupling_keys[household_index]);\n\n        for decision_ordinal in 0..self.decision_order.len() {\n            let household_index = self.decision_order[decision_ordinal];\n            let members = self.living_members[household_index];",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "        self.kin_locations.resize_with(household_count, Vec::new);\n        self.planned_destinations\n            .resize(household_count, CellId::INVALID);",
    "        self.kin_locations.resize_with(household_count, Vec::new);\n        self.household_coupling_keys\n            .resize(household_count, u64::MAX);\n        self.decision_order.clear();\n        self.planned_destinations\n            .resize(household_count, CellId::INVALID);",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "            || self.kin_locations.len() != household_count\n            || self.planned_destinations.len() != household_count",
    "            || self.kin_locations.len() != household_count\n            || self.household_coupling_keys.len() != household_count\n            || self.planned_destinations.len() != household_count",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "        for locations in &mut self.kin_locations {\n            locations.clear();\n        }\n        self.planned_destinations.fill(CellId::INVALID);",
    "        for locations in &mut self.kin_locations {\n            locations.clear();\n        }\n        self.household_coupling_keys.fill(u64::MAX);\n        self.planned_destinations.fill(CellId::INVALID);",
)
replace_once(
    "crates/anthrosim-core/src/migration.rs",
    "            let household_index = household_index(household, population.household_count())?;\n            let location = population.location_at_index(person_index).ok_or(",
    "            let household_index = household_index(household, population.household_count())?;\n            let coupling_rank = population\n                .stochastic_coupling_rank_at_index(person_index)\n                .ok_or(MigrationError::InternalInvariant(\n                    \"living person has no stochastic coupling rank\",\n                ))?;\n            self.household_coupling_keys[household_index] =\n                self.household_coupling_keys[household_index].min(coupling_rank);\n            let location = population.location_at_index(person_index).ok_or(",
)

# Causal continuation compatibility.
replace_once(
    "crates/anthrosim-core/src/checkpoint.rs",
    "    pub const PRE_BACKGROUND_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 14;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 15;",
    "    pub const PRE_BACKGROUND_MORTALITY_COUPLING_SCHEMA_VERSION: u32 = 14;\n    pub const PRE_MIGRATION_HOUSEHOLD_COUPLING_SCHEMA_VERSION: u32 = 15;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 16;",
)
replace_once(
    "crates/anthrosim-core/src/provenance.rs",
    "/// A v26 checkpoint must therefore not resume under v27 with unchanged mortality RNG positions.\npub const MODEL_SEMANTICS_ID: &str = \"anthrosim-model-semantics-v27\";",
    "/// A v26 checkpoint must therefore not resume under v27 with unchanged mortality RNG positions.\n///\n/// v28 removes arbitrary `HouseholdId` order from M4 sequential migration RNG assignment. Each\n/// non-empty household is scheduled by the minimum persistent stochastic-coupling rank among its\n/// living members while all decisions still observe one shared pre-move snapshot and relocations\n/// are still applied simultaneously. A v27 checkpoint must therefore not resume under v28 with\n/// unchanged migration RNG positions while silently changing which household receives each draw.\npub const MODEL_SEMANTICS_ID: &str = \"anthrosim-model-semantics-v28\";",
)
replace_once(
    "scripts/test-current-model-semantics-docs.py",
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v27"\nCURRENT_SHORT = "v27"',
    'CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v28"\nCURRENT_SHORT = "v28"',
)

# Living semantics documents. Preserve historical v27 evidence prose; change only current headers
# and add the new M4 decision-order contract.
for path in (
    "docs/scientific-model.md",
    "docs/research/odd.md",
    "docs/research/odd-d.md",
):
    replace_once(path, "current model semantics v27", "current model semantics v28")

replace_once(
    "docs/scientific-model.md",
    "M4 permanent relocation remains atomic at its decision boundary, but its opportunity clock is now independent of M3. For `D = migration.decisionPeriodsPerYear`, decision interval `j` uses `[floor(j*365/D), floor((j+1)*365/D))`; the synthetic default is `D = 4`. M4's resource-support term uses annual per-person need allocated over the current M4 decision interval using the same cumulative elapsed-day annual-allocation rule as M3. The runtime reconciles M4 decision index and actual decision day rather than assuming every resource boundary is a migration boundary.",
    "M4 permanent relocation remains atomic at its decision boundary, but its opportunity clock is now independent of M3. For `D = migration.decisionPeriodsPerYear`, decision interval `j` uses `[floor(j*365/D), floor((j+1)*365/D))`; the synthetic default is `D = 4`. M4's resource-support term uses annual per-person need allocated over the current M4 decision interval using the same cumulative elapsed-day annual-allocation rule as M3. The runtime reconciles M4 decision index and actual decision day rather than assuming every resource boundary is a migration boundary. Under v28, all households still evaluate one shared pre-move snapshot, but sequential `migration/choice` and `migration/uncertainty` draws are assigned in a HouseholdId-independent schedule keyed by the minimum persistent stochastic-coupling rank among each household's living members. Planned relocations are still applied simultaneously. Candidate enumeration within one household is unchanged by this repair and remains a separate scientific ordering surface.",
)
replace_once(
    "docs/research/odd.md",
    "- M4 resource support allocates annual per-person need over its own current decision interval using the same cumulative elapsed-day allocation rule rather than requiring an M3 resource boundary.",
    "- M4 resource support allocates annual per-person need over its own current decision interval using the same cumulative elapsed-day allocation rule rather than requiring an M3 resource boundary. Under v28, households still evaluate one shared pre-move snapshot, while sequential migration choice/uncertainty draws are assigned by a HouseholdId-independent schedule keyed by the minimum persistent stochastic-coupling rank among living household members; selected relocations remain simultaneous.",
)
replace_once(
    "docs/research/odd-d.md",
    "At each configured M4 decision boundary, households are first tested for relocation pressure. Pressured households compare the utility of staying with bounded local alternatives using a shared pre-move snapshot. The stay action evaluates residence-state terms only; candidate actions evaluate the same destination residence terms and then pay relocation-only travel, uncertainty and relocation-risk costs. Alternatives that exceed the configured minimum improvement over the stay action become eligible; one is selected through weighted deterministic stochastic choice. Selected moves are then applied simultaneously.",
    "At each configured M4 decision boundary, households are first tested for relocation pressure. Under v28 the household evaluation schedule is independent of arbitrary `HouseholdId`: non-empty households are ordered by the minimum persistent stochastic-coupling rank among their living members. Pressured households compare the utility of staying with bounded local alternatives using one shared pre-move snapshot. The stay action evaluates residence-state terms only; candidate actions evaluate the same destination residence terms and then pay relocation-only travel, uncertainty and relocation-risk costs. Alternatives that exceed the configured minimum improvement over the stay action become eligible; one is selected through weighted deterministic stochastic choice. Selected moves are then applied simultaneously.",
)
replace_once(
    "docs/architecture.md",
    "Selected household moves are evaluated against one shared pre-move snapshot and then applied simultaneously in one packed scan of the living population. This prevents household-ID evaluation order from changing the information available to later households and avoids scanning the whole population separately for every move.",
    "Selected household moves are evaluated against one shared pre-move snapshot and then applied simultaneously in one packed scan of the living population. Under v28, the sequential M4 RNG schedule is derived from the minimum persistent person stochastic-coupling rank among each non-empty household's living members instead of `HouseholdId`, so pure household relabelling cannot reassign migration draws while no second persisted household identity is introduced. The packed application still avoids scanning the whole population separately for every move.",
)
replace_once(
    "docs/architecture.md",
    "Migration candidates are enumerated in a stable geometric order and household decisions are evaluated in stable household-ID order. Stochastic destination selection is therefore replayable without requiring global optimization.",
    "Migration candidates are enumerated in a stable geometric order. Under v28, household decisions consume the separate `migration/choice` and `migration/uncertainty` streams in a schedule keyed by the minimum persistent stochastic-coupling rank among living household members rather than canonical `HouseholdId`. Stochastic destination selection remains exactly replayable without requiring global optimization.",
)

# Remove this helper from the production candidate; the workflow is removed separately by the
# repository connector because the Actions token cannot update workflow files.
Path("scripts/tmp_patch_av4_003.py").unlink()
