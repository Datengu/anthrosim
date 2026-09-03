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


population_path = Path("crates/anthrosim-core/src/population.rs")
population = population_path.read_text()

population = replace_once(
    population,
    "    condition_loss_remainder_thousandths: Vec<u16>,\n    household_locations: Vec<CellId>,",
    "    condition_loss_remainder_thousandths: Vec<u16>,\n    /// Stable stochastic-coupling order independent of canonical PersonId labels.\n    /// Founder ranks are canonicalized from declared scientific state; model births receive the\n    /// next rank in the already-canonical demographic birth sequence.\n    stochastic_ranks: Vec<u64>,\n    household_locations: Vec<CellId>,",
    "population stochastic-rank column",
)

population = replace_once(
    population,
    "    /// v4 binds per-person fractional M3 condition-loss remainder into causal population state.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 4;",
    "    /// v5 binds a stable per-person stochastic-coupling rank into causal population state so\n    /// arbitrary canonical PersonId relabelling cannot reassign sequential scientific RNG draws.\n    pub const CURRENT_SCHEMA_VERSION: u32 = 5;",
    "population schema",
)

population = replace_n(
    population,
    "        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;",
    "        let stochastic_ranks = vec![0_u64; person_count];\n        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;",
    2,
    "initializer stochastic-rank allocation",
)

population = replace_n(
    population,
    "            condition_loss_remainder_thousandths,\n            household_locations,",
    "            condition_loss_remainder_thousandths,\n            stochastic_ranks,\n            household_locations,",
    2,
    "initializer stochastic-rank field",
)

population = population.replace("        let population = Self {", "        let mut population = Self {", 2)
if population.count("        let mut population = Self {") < 2:
    raise SystemExit("initializer mutability: failed to find both population constructions")

first_finalize = "            occupancy,\n        };\n        population.validate(world)?;\n        Ok(population)"
if population.count(first_finalize) != 2:
    raise SystemExit(f"initializer finalize: expected 2 matches, found {population.count(first_finalize)}")
population = population.replace(
    first_finalize,
    "            occupancy,\n        };\n        population.assign_initial_stochastic_ranks(Some(definition));\n        population.validate(world)?;\n        Ok(population)",
    1,
)
population = population.replace(
    first_finalize,
    "            occupancy,\n        };\n        population.assign_initial_stochastic_ranks(None);\n        population.validate(world)?;\n        Ok(population)",
    1,
)

population = replace_once(
    population,
    "    #[must_use]\n    pub(crate) fn condition_loss_remainder_thousandths_at_index(\n        &self,\n        index: usize,\n    ) -> Option<u16> {\n        self.condition_loss_remainder_thousandths\n            .get(index)\n            .copied()\n    }\n",
    "    #[must_use]\n    pub(crate) fn condition_loss_remainder_thousandths_at_index(\n        &self,\n        index: usize,\n    ) -> Option<u16> {\n        self.condition_loss_remainder_thousandths\n            .get(index)\n            .copied()\n    }\n\n    #[must_use]\n    pub(crate) fn stochastic_rank_at_index(&self, index: usize) -> Option<u64> {\n        self.stochastic_ranks.get(index).copied()\n    }\n",
    "stochastic-rank accessor",
)

population = replace_once(
    population,
    "        let id = person_id_from_index(self.person_count());\n        self.birth_days.push(birth_day);",
    "        let id = person_id_from_index(self.person_count());\n        let stochastic_rank = u64::try_from(self.person_count())\n            .map_err(|_| PopulationError::InternalInvariant {\n                reason: \"person count does not fit stochastic-rank space\",\n            })?\n            .checked_add(1)\n            .ok_or(PopulationError::InternalInvariant {\n                reason: \"stochastic-rank space is exhausted\",\n            })?;\n        self.birth_days.push(birth_day);",
    "newborn stochastic rank",
)

population = replace_once(
    population,
    "        self.condition_permille.push(PERMILLE_MAX);\n        self.condition_loss_remainder_thousandths.push(0);\n        self.births_since_start = self.births_since_start.saturating_add(1);",
    "        self.condition_permille.push(PERMILLE_MAX);\n        self.condition_loss_remainder_thousandths.push(0);\n        self.stochastic_ranks.push(stochastic_rank);\n        self.births_since_start = self.births_since_start.saturating_add(1);",
    "newborn stochastic-rank push",
)

marker = "#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub(crate) struct HouseholdFissionOutcome {\n    pub households_created: u64,\n    pub people_reassigned: u64,\n    pub fissions: Vec<HouseholdFissionRecord>,\n}\n"
insertion = marker + "\n#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]\nstruct InitialStochasticPersonKey {\n    location: CellId,\n    birth_day: i64,\n    reproductive_sex_rank: u8,\n    condition_permille: u16,\n    condition_loss_remainder_thousandths: u16,\n    declared_last_birth_day: Option<i64>,\n    relationship_rank: u64,\n}\n"
population = replace_once(population, marker, insertion, "initial stochastic key type")

method_marker = "    /// Derives deterministic, PersonId-independent relationship-role classes for the living\n"
methods = r'''    fn initial_stochastic_person_key(
        &self,
        index: usize,
        relationship_rank: u64,
        founder_population: Option<&FounderPopulationDefinition>,
    ) -> InitialStochasticPersonKey {
        let person = person_id_from_index(index);
        InitialStochasticPersonKey {
            location: self.locations[index],
            birth_day: self.birth_days[index],
            reproductive_sex_rank: match self.reproductive_sexes[index] {
                ReproductiveSex::Female => 0,
                ReproductiveSex::Male => 1,
            },
            condition_permille: self.condition_permille[index],
            condition_loss_remainder_thousandths: self.condition_loss_remainder_thousandths[index],
            declared_last_birth_day: founder_population
                .and_then(|definition| definition.last_birth_day(person)),
            relationship_rank,
        }
    }

    /// Assign a stable scientific stochastic order to the initial population. Household numeric
    /// labels are used only to recover membership; ordering is derived from residence plus the
    /// multiset of member scientific states and relationship roles. PersonId is a final tie-break
    /// only inside an equivalence class that this declared scientific state cannot distinguish.
    fn assign_initial_stochastic_ranks(
        &mut self,
        founder_population: Option<&FounderPopulationDefinition>,
    ) {
        let person_count = self.person_count();
        if person_count == 0 {
            return;
        }

        let mut relationship_ranks = vec![u64::MAX; person_count];
        let mut household_keys = Vec::with_capacity(self.household_count());
        for household_index in 0..self.household_count() {
            let household = HouseholdId::new(
                u64::try_from(household_index).expect("household index must fit u64") + 1,
            );
            let members = (0..person_count)
                .filter(|&index| self.households[index] == household)
                .collect::<Vec<_>>();
            let ranks = self.household_relationship_ranks(&members);
            let mut member_keys = Vec::with_capacity(members.len());
            for &index in &members {
                relationship_ranks[index] = ranks[index];
                member_keys.push(self.initial_stochastic_person_key(
                    index,
                    ranks[index],
                    founder_population,
                ));
            }
            member_keys.sort_unstable();
            household_keys.push((self.household_locations[household_index], member_keys));
        }

        let mut unique_household_keys = household_keys.clone();
        unique_household_keys.sort();
        unique_household_keys.dedup();
        let mut household_ranks = vec![u64::MAX; self.household_count()];
        for (household_index, key) in household_keys.iter().enumerate() {
            let rank = unique_household_keys
                .binary_search(key)
                .expect("initial household scientific key must be represented");
            household_ranks[household_index] =
                u64::try_from(rank).expect("household stochastic-rank space must fit u64");
        }

        let mut order = (0..person_count).collect::<Vec<_>>();
        order.sort_by_key(|&index| {
            let household_index = usize::try_from(self.households[index].0 - 1)
                .expect("validated household ID must fit usize");
            (
                household_ranks[household_index],
                self.initial_stochastic_person_key(
                    index,
                    relationship_ranks[index],
                    founder_population,
                ),
                person_id_from_index(index).0,
            )
        });
        for (ordinal, index) in order.into_iter().enumerate() {
            self.stochastic_ranks[index] = u64::try_from(ordinal)
                .expect("initial stochastic-rank ordinal must fit u64")
                + 1;
        }
    }

'''
if population.count(method_marker) != 1:
    raise SystemExit(f"stochastic-rank methods insertion: expected 1 marker, found {population.count(method_marker)}")
population = population.replace(method_marker, methods + method_marker, 1)

population = replace_once(
    population,
    "            self.condition_permille.len(),\n            self.condition_loss_remainder_thousandths.len(),\n        ];",
    "            self.condition_permille.len(),\n            self.condition_loss_remainder_thousandths.len(),\n            self.stochastic_ranks.len(),\n        ];",
    "validation stochastic column length",
)

length_check = "        if lengths.iter().any(|&length| length != person_count) {\n            return Err(PopulationValidationError::ColumnLengthMismatch);\n        }\n"
rank_validation = length_check + r'''
        let mut seen_stochastic_ranks = vec![false; person_count];
        for (index, &rank) in self.stochastic_ranks.iter().enumerate() {
            let Some(rank_index) = rank
                .checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())
                .filter(|&value| value < person_count)
            else {
                return Err(PopulationValidationError::InvalidStochasticRank {
                    person: person_id_from_index(index),
                    rank,
                    records: person_count as u64,
                });
            };
            if seen_stochastic_ranks[rank_index] {
                return Err(PopulationValidationError::DuplicateStochasticRank { rank });
            }
            seen_stochastic_ranks[rank_index] = true;
        }
'''
population = replace_once(population, length_check, rank_validation, "stochastic-rank validation")

population = replace_once(
    population,
    "            digest_u64(\n                &mut hash,\n                u64::from(self.condition_loss_remainder_thousandths[index]),\n            );\n        }",
    "            digest_u64(\n                &mut hash,\n                u64::from(self.condition_loss_remainder_thousandths[index]),\n            );\n            digest_u64(&mut hash, self.stochastic_ranks[index]);\n        }",
    "stochastic-rank digest",
)

error_marker = "    #[error(\"population accounting overflowed\")]\n    PopulationAccountingOverflow,\n"
error_insert = error_marker + "    #[error(\"person {person:?} has stochastic rank {rank} outside 1..={records}\")]\n    InvalidStochasticRank { person: PersonId, rank: u64, records: u64 },\n    #[error(\"stochastic rank {rank} is assigned to multiple person records\")]\n    DuplicateStochasticRank { rank: u64 },\n"
population = replace_once(population, error_marker, error_insert, "stochastic-rank validation errors")

population_path.write_text(population)


demography_path = Path("crates/anthrosim-core/src/demography.rs")
demography = demography_path.read_text()
old_loop = "    let mut births_added = false;\n    for female_index in 0..records_at_boundary_start {"
new_loop = r'''    // Canonical PersonId is bookkeeping identity. Consume the existing sequential fertility
    // stream in stable scientific stochastic-rank order so a pure founder relabelling cannot move
    // a same-seed fertility realization between otherwise fixed people/households/cells.
    let mut fertility_order = Vec::new();
    for index in 0..records_at_boundary_start {
        if !population.is_alive_index(index)
            || population.reproductive_sex_at_index(index) != Some(ReproductiveSex::Female)
        {
            continue;
        }
        let rank = population.stochastic_rank_at_index(index).ok_or(
            PopulationError::InternalInvariant {
                reason: "living female is missing a stochastic coupling rank",
            },
        )?;
        fertility_order.push((rank, index));
    }
    fertility_order.sort_unstable_by_key(|&(rank, _)| rank);

    let mut births_added = false;
    for (_, female_index) in fertility_order {'''
demography = replace_once(demography, old_loop, new_loop, "fertility stochastic ordering")
demography_path.write_text(demography)


observability_path = Path("crates/anthrosim-core/src/demography_observability.rs")
observability = observability_path.read_text()
observability = replace_once(
    observability,
    "struct ReplayPerson {\n    id: PersonId,\n    birth_day: i64,",
    "struct ReplayPerson {\n    id: PersonId,\n    stochastic_rank: u64,\n    birth_day: i64,",
    "replay stochastic rank field",
)
observability = replace_once(
    observability,
    "        people.push(ReplayPerson {\n            id,\n            birth_day: person.birth_day,",
    "        let stochastic_rank = population\n            .stochastic_rank_at_index(index)\n            .ok_or_else(|| invalid(format!(\"initial population is missing stochastic rank for {id:?}\")))?;\n        people.push(ReplayPerson {\n            id,\n            stochastic_rank,\n            birth_day: person.birth_day,",
    "initial replay stochastic rank",
)
old_replay_loop = "        for female_index in 0..records_at_boundary_start {\n            if !people[female_index].alive()\n                || people[female_index].reproductive_sex != ReproductiveSex::Female\n            {\n                continue;\n            }"
new_replay_loop = r'''        let mut fertility_order = (0..records_at_boundary_start)
            .filter(|&index| {
                people[index].alive()
                    && people[index].reproductive_sex == ReproductiveSex::Female
            })
            .map(|index| (people[index].stochastic_rank, index))
            .collect::<Vec<_>>();
        fertility_order.sort_unstable_by_key(|&(rank, _)| rank);

        for (_, female_index) in fertility_order {'''
observability = replace_once(observability, old_replay_loop, new_replay_loop, "observability fertility ordering")
observability = replace_once(
    observability,
    "    people.push(ReplayPerson {\n        id: birth.person,\n        birth_day,",
    "    let stochastic_rank = u64::try_from(people.len())\n        .map_err(|_| invalid(\"replay person count does not fit stochastic-rank space\".to_owned()))?\n        .checked_add(1)\n        .ok_or_else(|| invalid(\"replay stochastic-rank space is exhausted\".to_owned()))?;\n    people.push(ReplayPerson {\n        id: birth.person,\n        stochastic_rank,\n        birth_day,",
    "replay newborn stochastic rank",
)
reconcile_marker = "        let checks = [\n            (final_person.birth_day == person.birth_day, \"birthDay\"),"
reconcile_insert = r'''        let final_index = replay_index(person.id, final_population.person_count())
            .ok_or_else(|| invalid(format!("final population is missing index for {:?}", person.id)))?;
        let final_stochastic_rank = final_population
            .stochastic_rank_at_index(final_index)
            .ok_or_else(|| invalid(format!("final population is missing stochastic rank for {:?}", person.id)))?;
        if final_stochastic_rank != person.stochastic_rank {
            return Err(invalid(format!(
                "replayed stochasticRank does not match final population for {:?}",
                person.id
            )));
        }
        let checks = [
            (final_person.birth_day == person.birth_day, "birthDay"),'''
observability = replace_once(observability, reconcile_marker, reconcile_insert, "reconcile stochastic rank")
observability_path.write_text(observability)


provenance_path = Path("crates/anthrosim-core/src/provenance.rs")
provenance = provenance_path.read_text()
provenance = replace_once(
    provenance,
    "/// v25 makes dependency-aware household fission relationship-aware before arbitrary identity\n/// tie-breaking, so scientifically equivalent PersonId relabellings preserve unlabelled kin\n/// composition when age and sex are equal but parent/child roles differ.\npub const MODEL_SEMANTICS_ID: &str = \"anthrosim-model-semantics-v25\";",
    "/// v25 makes dependency-aware household fission relationship-aware before arbitrary identity\n/// tie-breaking, so scientifically equivalent PersonId relabellings preserve unlabelled kin\n/// composition when age and sex are equal but parent/child roles differ.\n///\n/// v26 binds a canonical scientific stochastic rank to each person and consumes annual fertility\n/// draws in that rank order. Canonical PersonId therefore remains bookkeeping identity rather than\n/// deciding which fixed person/household/cell receives a same-seed fertility realization.\npub const MODEL_SEMANTICS_ID: &str = \"anthrosim-model-semantics-v26\";",
    "model semantics v26",
)
provenance_path.write_text(provenance)


doc_path = Path("docs/research/m2-demography-observability-v1.md")
doc = doc_path.read_text()
doc = replace_once(
    doc,
    "8. replay the independent `demography/fertility` RNG stream for the exact attempted draws;",
    "8. replay the independent `demography/fertility` RNG stream in persisted scientific stochastic-rank order for the exact attempted draws, so canonical `PersonId` labels do not assign fertility realizations;",
    "observability contract fertility ordering",
)
doc_path.write_text(doc)
