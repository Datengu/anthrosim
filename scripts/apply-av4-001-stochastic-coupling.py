from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def replace_n(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


# ---------------------------------------------------------------------------
# Population: persist a relabelling-invariant stochastic coupling identity.
# ---------------------------------------------------------------------------
p = Path("crates/anthrosim-core/src/population.rs")
text = p.read_text(encoding="utf-8")

marker = "#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub(crate) struct HouseholdFissionOutcome {\n    pub households_created: u64,\n    pub people_reassigned: u64,\n    pub fissions: Vec<HouseholdFissionRecord>,\n}\n"
insert = marker + r'''

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct InitialStochasticBaseKey {
    birth_day: i64,
    reproductive_sex_rank: u8,
    location: CellId,
    condition_permille: u16,
    condition_loss_remainder_thousandths: u16,
    declared_last_birth_day: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct InitialStochasticSignature {
    prior_rank: u64,
    female_parent_rank: Option<u64>,
    male_parent_rank: Option<u64>,
    children: Vec<(u8, u64)>,
    household_members: Vec<u64>,
}
'''
text = replace_once(text, marker, insert, "stochastic key structs")

text = replace_once(
    text,
    "    condition_loss_remainder_thousandths: Vec<u16>,\n    household_locations: Vec<CellId>,",
    "    condition_loss_remainder_thousandths: Vec<u16>,\n    /// Persistent person-level coupling identity for sequential scientific RNG streams.\n    /// The initial assignment is canonicalized from represented scientific state rather than\n    /// canonical PersonId/packed-record order; model births receive subsequent ranks.\n    #[serde(default)]\n    stochastic_coupling_ranks: Vec<u64>,\n    household_locations: Vec<CellId>,",
    "population rank column",
)
text = replace_once(
    text,
    "    /// v4 binds per-person fractional M3 condition-loss remainder into causal population state.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 4;",
    "    /// v5 binds persistent person-level stochastic coupling identity into causal population\n    /// state so checkpoint/resume preserves relabelling-invariant RNG assignment.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 5;",
    "population schema v5",
)

# Both declared and synthetic initializers build a zeroed column, then canonicalize it before
# validation. The canonicalizer sees fully materialized day-zero scientific state.
text = replace_n(
    text,
    "        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;",
    "        let stochastic_coupling_ranks = vec![0_u64; person_count];\n        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;",
    2,
    "initializer rank allocation",
)
text = replace_n(
    text,
    "        let population = Self {",
    "        let mut population = Self {",
    2,
    "initializer mutability",
)
text = replace_n(
    text,
    "            condition_loss_remainder_thousandths,\n            household_locations,",
    "            condition_loss_remainder_thousandths,\n            stochastic_coupling_ranks,\n            household_locations,",
    2,
    "initializer rank field",
)
finish = "            occupancy,\n        };\n        population.validate(world)?;\n        Ok(population)"
if text.count(finish) != 2:
    raise SystemExit(f"initializer finalization: expected 2 matches, found {text.count(finish)}")
text = text.replace(
    finish,
    "            occupancy,\n        };\n        population.assign_initial_stochastic_coupling_ranks(Some(definition));\n        population.validate(world)?;\n        Ok(population)",
    1,
)
text = text.replace(
    finish,
    "            occupancy,\n        };\n        population.assign_initial_stochastic_coupling_ranks(None);\n        population.validate(world)?;\n        Ok(population)",
    1,
)

text = replace_once(
    text,
    "    #[must_use]\n    pub(crate) fn condition_loss_remainder_thousandths_at_index(\n        &self,\n        index: usize,\n    ) -> Option<u16> {\n        self.condition_loss_remainder_thousandths\n            .get(index)\n            .copied()\n    }\n",
    "    #[must_use]\n    pub(crate) fn condition_loss_remainder_thousandths_at_index(\n        &self,\n        index: usize,\n    ) -> Option<u16> {\n        self.condition_loss_remainder_thousandths\n            .get(index)\n            .copied()\n    }\n\n    #[must_use]\n    pub(crate) fn stochastic_coupling_rank_at_index(&self, index: usize) -> Option<u64> {\n        self.stochastic_coupling_ranks.get(index).copied()\n    }\n",
    "rank accessor",
)

text = replace_once(
    text,
    "        let id = person_id_from_index(self.person_count());\n        self.birth_days.push(birth_day);",
    "        let id = person_id_from_index(self.person_count());\n        let stochastic_coupling_rank = u64::try_from(self.person_count())\n            .map_err(|_| PopulationError::InternalInvariant {\n                reason: \"person count does not fit stochastic coupling rank space\",\n            })?\n            .checked_add(1)\n            .ok_or(PopulationError::InternalInvariant {\n                reason: \"stochastic coupling rank space is exhausted\",\n            })?;\n        self.birth_days.push(birth_day);",
    "newborn rank allocation",
)
text = replace_once(
    text,
    "        self.condition_permille.push(PERMILLE_MAX);\n        self.condition_loss_remainder_thousandths.push(0);\n        self.births_since_start = self.births_since_start.saturating_add(1);",
    "        self.condition_permille.push(PERMILLE_MAX);\n        self.condition_loss_remainder_thousandths.push(0);\n        self.stochastic_coupling_ranks.push(stochastic_coupling_rank);\n        self.births_since_start = self.births_since_start.saturating_add(1);",
    "newborn rank push",
)

method_marker = "    /// Derives deterministic, PersonId-independent relationship-role classes for the living\n"
methods = r'''    fn initial_stochastic_base_key(
        &self,
        index: usize,
        founder_population: Option<&FounderPopulationDefinition>,
    ) -> InitialStochasticBaseKey {
        let person = person_id_from_index(index);
        let declared_last_birth_day = founder_population.and_then(|definition| {
            definition
                .people
                .iter()
                .find(|founder| founder.id == person)
                .and_then(|founder| founder.last_birth_day)
        });
        InitialStochasticBaseKey {
            birth_day: self.birth_days[index],
            reproductive_sex_rank: match self.reproductive_sexes[index] {
                ReproductiveSex::Female => 0,
                ReproductiveSex::Male => 1,
            },
            location: self.locations[index],
            condition_permille: self.condition_permille[index],
            condition_loss_remainder_thousandths: self.condition_loss_remainder_thousandths[index],
            declared_last_birth_day,
        }
    }

    /// Canonicalize day-zero represented people into persistent coupling ranks without using
    /// canonical PersonId as a scientific distinction. Refinement incorporates scalar state,
    /// genealogy, and household-role multisets globally. PersonId is used only to order records
    /// that remain scientifically indistinguishable after the refinement; swapping such records
    /// leaves the unlabeled scientific state unchanged.
    fn assign_initial_stochastic_coupling_ranks(
        &mut self,
        founder_population: Option<&FounderPopulationDefinition>,
    ) {
        let person_count = self.person_count();
        if person_count == 0 {
            return;
        }

        let base_keys = (0..person_count)
            .map(|index| self.initial_stochastic_base_key(index, founder_population))
            .collect::<Vec<_>>();
        let mut unique_base_keys = base_keys.clone();
        unique_base_keys.sort();
        unique_base_keys.dedup();
        let mut ranks = base_keys
            .iter()
            .map(|key| {
                u64::try_from(
                    unique_base_keys
                        .binary_search(key)
                        .expect("initial stochastic base key must be represented"),
                )
                .expect("initial stochastic base-rank space must fit u64")
            })
            .collect::<Vec<_>>();

        let mut child_links = vec![Vec::<(u8, usize)>::new(); person_count];
        for child_index in 0..person_count {
            for (role, parent) in [
                (0_u8, self.female_parents[child_index]),
                (1_u8, self.male_parents[child_index]),
            ] {
                if let Some(parent_index) = person_index(parent, person_count) {
                    child_links[parent_index].push((role, child_index));
                }
            }
        }

        for _ in 0..person_count {
            let mut household_members = vec![Vec::<u64>::new(); self.household_count()];
            for index in 0..person_count {
                let household_index = usize::try_from(self.households[index].0 - 1)
                    .expect("validated initial household ID must fit usize");
                household_members[household_index].push(ranks[index]);
            }
            for members in &mut household_members {
                members.sort_unstable();
            }

            let signatures = (0..person_count)
                .map(|index| {
                    let parent_rank = |parent: PersonId| {
                        person_index(parent, person_count).map(|parent_index| ranks[parent_index])
                    };
                    let mut children = child_links[index]
                        .iter()
                        .map(|&(role, child_index)| (role, ranks[child_index]))
                        .collect::<Vec<_>>();
                    children.sort_unstable();
                    let household_index = usize::try_from(self.households[index].0 - 1)
                        .expect("validated initial household ID must fit usize");
                    InitialStochasticSignature {
                        prior_rank: ranks[index],
                        female_parent_rank: parent_rank(self.female_parents[index]),
                        male_parent_rank: parent_rank(self.male_parents[index]),
                        children,
                        household_members: household_members[household_index].clone(),
                    }
                })
                .collect::<Vec<_>>();
            let mut unique_signatures = signatures.clone();
            unique_signatures.sort();
            unique_signatures.dedup();
            let next_ranks = signatures
                .iter()
                .map(|signature| {
                    u64::try_from(
                        unique_signatures
                            .binary_search(signature)
                            .expect("initial stochastic signature must be represented"),
                    )
                    .expect("initial stochastic signature-rank space must fit u64")
                })
                .collect::<Vec<_>>();
            if next_ranks == ranks {
                break;
            }
            ranks = next_ranks;
        }

        let mut order = (0..person_count).collect::<Vec<_>>();
        order.sort_by_key(|&index| (ranks[index], person_id_from_index(index).0));
        for (ordinal, index) in order.into_iter().enumerate() {
            self.stochastic_coupling_ranks[index] = u64::try_from(ordinal)
                .expect("initial stochastic coupling ordinal must fit u64")
                + 1;
        }
    }

'''
if text.count(method_marker) != 1:
    raise SystemExit(f"rank method insertion marker: found {text.count(method_marker)}")
text = text.replace(method_marker, methods + method_marker, 1)

text = replace_once(
    text,
    "            self.condition_permille.len(),\n            self.condition_loss_remainder_thousandths.len(),\n        ];",
    "            self.condition_permille.len(),\n            self.condition_loss_remainder_thousandths.len(),\n            self.stochastic_coupling_ranks.len(),\n        ];",
    "rank validation column length",
)
text = replace_once(
    text,
    "        let records = person_count as u64;\n        let expected_records = u64::from(self.initial_population)",
    r'''        let records = person_count as u64;
        let mut seen_stochastic_coupling_ranks = vec![false; person_count];
        for (index, &rank) in self.stochastic_coupling_ranks.iter().enumerate() {
            let Some(rank_index) = rank
                .checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())
                .filter(|&value| value < person_count)
            else {
                return Err(PopulationValidationError::InvalidStochasticCouplingRank {
                    person: person_id_from_index(index),
                    rank,
                    records,
                });
            };
            if seen_stochastic_coupling_ranks[rank_index] {
                return Err(PopulationValidationError::DuplicateStochasticCouplingRank { rank });
            }
            seen_stochastic_coupling_ranks[rank_index] = true;
        }
        let expected_records = u64::from(self.initial_population)''',
    "rank validation values",
)
text = replace_once(
    text,
    "            digest_u64(\n                &mut hash,\n                u64::from(self.condition_loss_remainder_thousandths[index]),\n            );\n        }",
    "            digest_u64(\n                &mut hash,\n                u64::from(self.condition_loss_remainder_thousandths[index]),\n            );\n            digest_u64(&mut hash, self.stochastic_coupling_ranks[index]);\n        }",
    "rank digest binding",
)
text = replace_once(
    text,
    "    #[error(\"population accounting overflowed\")]\n    PopulationAccountingOverflow,",
    "    #[error(\"population accounting overflowed\")]\n    PopulationAccountingOverflow,\n    #[error(\"person {person:?} has stochastic coupling rank {rank} outside 1..={records}\")]\n    InvalidStochasticCouplingRank { person: PersonId, rank: u64, records: u64 },\n    #[error(\"stochastic coupling rank {rank} is assigned to multiple person records\")]\n    DuplicateStochasticCouplingRank { rank: u64 },",
    "rank validation errors",
)
p.write_text(text, encoding="utf-8")


# ---------------------------------------------------------------------------
# Demography: consume the existing fertility stream in persistent coupling-rank order.
# ---------------------------------------------------------------------------
p = Path("crates/anthrosim-core/src/demography.rs")
text = p.read_text(encoding="utf-8")
role_call = r'''    let role_ranks = demographic_role_ranks(
        population,
        records_at_boundary_start,
        day,
        &same_day_migration_origins,
        founder_population,
    )?;

'''
text = replace_once(text, role_call, "", "remove dynamic role-rank call")
text = replace_once(
    text,
    "    // Freeze the eligible-female set before any birth is appended, matching the historical annual\n    // boundary semantics, but assign the shared fertility stream in a scientific-state order rather\n    // than packed-record/PersonId order. The final PersonId tie-break is reached only for records\n    // that remain indistinguishable under the complete relabelling-invariant role refinement.",
    "    // Freeze the eligible-female set before any birth is appended, matching the historical annual\n    // boundary semantics, but assign the shared fertility stream by persistent scientific coupling\n    // identity rather than packed-record/PersonId order. The rank is canonicalized from day-zero\n    // represented state and persisted through checkpoints, so later condition/migration differences\n    // cannot make independent replay lose the RNG-to-person coupling.",
    "fertility ordering comment",
)
text = replace_once(
    text,
    "        fertility_candidates.push((\n            role_ranks[female_index],\n            female,",
    "        let stochastic_coupling_rank = population\n            .stochastic_coupling_rank_at_index(female_index)\n            .ok_or(PopulationError::InternalInvariant {\n                reason: \"living female is missing stochastic coupling identity\",\n            })?;\n        fertility_candidates.push((\n            stochastic_coupling_rank,\n            female,",
    "fertility candidate rank",
)
start_marker = "#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]\nstruct DemographicRoleBaseKey"
end_marker = "fn same_day_migration_origins("
start = text.find(start_marker)
end = text.find(end_marker, start)
if start < 0 or end < 0:
    raise SystemExit(f"dynamic role helper removal markers missing: start={start}, end={end}")
text = text[:start] + text[end:]
p.write_text(text, encoding="utf-8")


# ---------------------------------------------------------------------------
# Independent demography observability: replay the same persisted coupling order.
# ---------------------------------------------------------------------------
p = Path("crates/anthrosim-core/src/demography_observability.rs")
text = p.read_text(encoding="utf-8")
text = replace_once(
    text,
    "struct ReplayPerson {\n    id: PersonId,\n    birth_day: i64,",
    "struct ReplayPerson {\n    id: PersonId,\n    stochastic_coupling_rank: u64,\n    birth_day: i64,",
    "replay rank field",
)
text = replace_once(
    text,
    "        people.push(ReplayPerson {\n            id,\n            birth_day: person.birth_day,",
    "        let stochastic_coupling_rank = population\n            .stochastic_coupling_rank_at_index(index)\n            .ok_or_else(|| invalid(format!(\"initial population is missing stochastic coupling rank for {id:?}\")))?;\n        people.push(ReplayPerson {\n            id,\n            stochastic_coupling_rank,\n            birth_day: person.birth_day,",
    "replay initial rank",
)
old_loop = r'''        for female_index in 0..records_at_boundary_start {
            if !people[female_index].alive()
                || people[female_index].reproductive_sex != ReproductiveSex::Female
            {
                continue;
            }'''
new_loop = r'''        let mut fertility_order = (0..records_at_boundary_start)
            .filter(|&index| {
                people[index].alive()
                    && people[index].reproductive_sex == ReproductiveSex::Female
            })
            .map(|index| (people[index].stochastic_coupling_rank, index))
            .collect::<Vec<_>>();
        fertility_order.sort_unstable_by_key(|&(rank, _)| rank);

        for (_, female_index) in fertility_order {'''
text = replace_once(text, old_loop, new_loop, "observability fertility order")
text = replace_once(
    text,
    "    people.push(ReplayPerson {\n        id: birth.person,\n        birth_day,",
    "    let stochastic_coupling_rank = u64::try_from(people.len())\n        .map_err(|_| invalid(\"replay person count does not fit stochastic coupling rank space\".to_owned()))?\n        .checked_add(1)\n        .ok_or_else(|| invalid(\"replay stochastic coupling rank space is exhausted\".to_owned()))?;\n    people.push(ReplayPerson {\n        id: birth.person,\n        stochastic_coupling_rank,\n        birth_day,",
    "replay newborn rank",
)
reconcile = "        let checks = [\n            (final_person.birth_day == person.birth_day, \"birthDay\"),"
reconcile_new = r'''        let final_index = replay_index(person.id, final_population.person_count())
            .ok_or_else(|| invalid(format!("final population is missing index for {:?}", person.id)))?;
        let final_stochastic_coupling_rank = final_population
            .stochastic_coupling_rank_at_index(final_index)
            .ok_or_else(|| {
                invalid(format!(
                    "final population is missing stochastic coupling rank for {:?}",
                    person.id
                ))
            })?;
        if final_stochastic_coupling_rank != person.stochastic_coupling_rank {
            return Err(invalid(format!(
                "replayed stochasticCouplingRank does not match final population for {:?}",
                person.id
            )));
        }
        let checks = [
            (final_person.birth_day == person.birth_day, "birthDay"),'''
text = replace_once(text, reconcile, reconcile_new, "reconcile rank")
p.write_text(text, encoding="utf-8")


# ---------------------------------------------------------------------------
# Checkpoint wire contract: nested Population shape/causal state changed.
# ---------------------------------------------------------------------------
p = Path("crates/anthrosim-core/src/checkpoint.rs")
text = p.read_text(encoding="utf-8")
text = replace_once(
    text,
    "    pub const PRE_M4_CHOICE_WEIGHT_TRACE_SCHEMA_VERSION: u32 = 12;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 13;",
    "    pub const PRE_M4_CHOICE_WEIGHT_TRACE_SCHEMA_VERSION: u32 = 12;\n    pub const PRE_STOCHASTIC_COUPLING_SCHEMA_VERSION: u32 = 13;\n    pub const CURRENT_SCHEMA_VERSION: u32 = 14;",
    "checkpoint schema v14",
)
p.write_text(text, encoding="utf-8")


# ---------------------------------------------------------------------------
# Observability contract documentation.
# ---------------------------------------------------------------------------
p = Path("docs/research/m2-demography-observability-v1.md")
text = p.read_text(encoding="utf-8")
text = replace_once(
    text,
    "8. replay the independent `demography/fertility` RNG stream for the exact attempted draws;",
    "8. replay the independent `demography/fertility` RNG stream in persisted scientific stochastic-coupling rank order for the exact attempted draws, so canonical `PersonId` labels do not assign fertility realizations;",
    "observability contract rank order",
)
p.write_text(text, encoding="utf-8")
