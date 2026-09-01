use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{DemographyConfig, PopulationConfig, PopulationInitialization},
    founder_initialization::{FounderPopulationDefinition, FounderPopulationError},
    ids::{CellId, HouseholdId, PersonId},
    rng::RngFactory,
    time::{DAYS_PER_YEAR, SimTime},
    world::{PERMILLE_MAX, World},
};

const NO_EVENT_DAY: u64 = u64::MAX;
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const CONDITION_RESPONSE_DENOMINATOR: u16 = 1_000;

/// Biological state used by the v0.1 reproduction mechanism.
///
/// This does not model social gender or gendered social roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReproductiveSex {
    Female,
    Male,
}

/// Read-only materialized view of one person stored in the packed population.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonSnapshot {
    pub id: PersonId,
    /// Epoch-relative day of birth. Founders born before the run use negative days.
    pub birth_day: i64,
    pub death_day: Option<u64>,
    /// Latest birth created during model execution. Declared pre-run reproductive-history timing
    /// remains in the immutable founder definition rather than being forged into model-time state.
    pub last_birth_day: Option<u64>,
    pub reproductive_sex: ReproductiveSex,
    /// Persistent residence cell. M9 temporary physical presence is stored separately.
    pub location: CellId,
    pub household: HouseholdId,
    pub female_parent: PersonId,
    pub male_parent: PersonId,
    pub condition_permille: u16,
}

impl PersonSnapshot {
    #[must_use]
    pub fn age_days_at(self, time: SimTime) -> Option<u64> {
        let now = i64::try_from(time.days()).ok()?;
        let age = now.checked_sub(self.birth_day)?;
        u64::try_from(age).ok()
    }

    #[must_use]
    pub const fn is_alive(self) -> bool {
        self.death_day.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulationSummary {
    pub schema_version: u32,
    pub initial_population: u32,
    pub person_records: u64,
    pub living_population: u64,
    pub births_since_start: u64,
    pub deaths_since_start: u64,
    pub household_count: u64,
    /// Number of cells containing at least one living persistent resident.
    /// M9 temporary visitors/transit are intentionally excluded.
    pub living_occupied_cell_count: u64,
    /// Mean condition among living people, or `None` when no living people exist.
    ///
    /// Zero remains a valid defined mean when living people have zero condition.
    pub mean_living_condition_permille: Option<u16>,
    pub living_below_half_condition: u64,
    pub digest64: u64,
}

/// Compact persistent-residence cell-to-person index rebuilt from authoritative person locations.
///
/// It indexes persistent person records, including the dead. Consumers that need M2/M4
/// residence-based interaction candidates must additionally filter for alive state. M9
/// temporary visitor/transit presence is authoritative elsewhere and is not indexed here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellOccupancy {
    offsets: Vec<u64>,
    people: Vec<PersonId>,
}

impl CellOccupancy {
    fn build(locations: &[CellId], cell_count: usize) -> Result<Self, PopulationValidationError> {
        let mut counts = vec![0_u64; cell_count];
        for (index, &location) in locations.iter().enumerate() {
            let cell_index = location_index(location, cell_count).ok_or(
                PopulationValidationError::InvalidPersonLocation {
                    person: person_id_from_index(index),
                    location,
                },
            )?;
            counts[cell_index] = counts[cell_index].saturating_add(1);
        }

        let mut offsets = Vec::with_capacity(cell_count.saturating_add(1));
        offsets.push(0);
        for count in counts {
            let next = offsets
                .last()
                .copied()
                .unwrap_or(0_u64)
                .saturating_add(count);
            offsets.push(next);
        }

        let mut cursor = offsets[..cell_count].to_vec();
        let mut people = vec![PersonId::INVALID; locations.len()];
        for (index, &location) in locations.iter().enumerate() {
            let cell_index =
                location_index(location, cell_count).expect("locations were validated");
            let write_index = usize::try_from(cursor[cell_index]).map_err(|_| {
                PopulationValidationError::OccupancyShape {
                    reason: "occupancy offset does not fit usize",
                }
            })?;
            people[write_index] = person_id_from_index(index);
            cursor[cell_index] = cursor[cell_index].saturating_add(1);
        }

        Ok(Self { offsets, people })
    }

    #[must_use]
    pub fn people_in_cell(&self, cell: CellId) -> Option<&[PersonId]> {
        let index = usize::try_from(cell.0.checked_sub(1)?).ok()?;
        let end_index = index.checked_add(1)?;
        if end_index >= self.offsets.len() {
            return None;
        }
        let start = usize::try_from(self.offsets[index]).ok()?;
        let end = usize::try_from(self.offsets[end_index]).ok()?;
        self.people.get(start..end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HouseholdRelocationOutcome {
    pub people_moved: u64,
    pub condition_loss_total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HouseholdFissionRecord {
    pub source_household: HouseholdId,
    pub new_household: HouseholdId,
    pub residence: CellId,
    pub people_reassigned: Vec<PersonId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct HouseholdFissionOutcome {
    pub households_created: u64,
    pub people_reassigned: u64,
    pub fissions: Vec<HouseholdFissionRecord>,
}

/// Authoritative persistent person/household state.
///
/// Hot per-person fields are stored as parallel contiguous arrays rather than
/// allocation-heavy person objects. Stable IDs are one-based indices into these
/// arrays; records are retained after death so genealogy can remain stable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Population {
    schema_version: u32,
    initial_population: u32,
    births_since_start: u64,
    deaths_since_start: u64,
    max_person_records: u64,
    birth_days: Vec<i64>,
    death_days: Vec<u64>,
    last_birth_days: Vec<u64>,
    reproductive_sexes: Vec<ReproductiveSex>,
    locations: Vec<CellId>,
    households: Vec<HouseholdId>,
    female_parents: Vec<PersonId>,
    male_parents: Vec<PersonId>,
    condition_permille: Vec<u16>,
    /// Unmaterialized M3 under-supply deterioration in thousandths of one condition point.
    /// This causal remainder is carried across resource boundaries and bound into checkpoints.
    condition_loss_remainder_thousandths: Vec<u16>,
    household_locations: Vec<CellId>,
    occupancy: CellOccupancy,
}

impl Population {
    /// v4 binds per-person fractional M3 condition-loss remainder into causal population state.
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;

    /// Initialize the frozen synthetic-validation founder preset.
    ///
    /// Declared founder state requires the separate definition-bearing constructor so a caller
    /// cannot accidentally request declared initialization while omitting the state that gives it
    /// scientific meaning.
    pub fn initialize(
        config: PopulationConfig,
        world: &World,
        rng_factory: RngFactory,
    ) -> Result<Self, PopulationError> {
        validate_config(config)?;
        if world.cell_count() == 0 {
            return Err(PopulationError::WorldHasNoCells);
        }

        match config.initialization {
            PopulationInitialization::SyntheticValidationV1 => {
                Self::initialize_synthetic_validation_v1(config, world, rng_factory)
            }
            PopulationInitialization::DeclaredFounderStateV1 => {
                Err(PopulationError::DeclaredFounderDefinitionRequired)
            }
        }
    }

    /// Materialize one exact declared founder state at simulation day 0.
    ///
    /// Synthetic-only population parameters such as the uniform age ceiling, synthetic sex ratio,
    /// and target synthetic household size are intentionally ignored. Every authoritative founder
    /// age, sex, household, residence, direct-parent link and condition comes from `definition`.
    pub fn initialize_declared_founder_state_v1(
        config: PopulationConfig,
        definition: &FounderPopulationDefinition,
        world: &World,
        demography: &DemographyConfig,
    ) -> Result<Self, PopulationError> {
        validate_config(config)?;
        if config.initialization != PopulationInitialization::DeclaredFounderStateV1 {
            return Err(PopulationError::DeclaredFounderModeNotSelected);
        }
        if world.cell_count() == 0 {
            return Err(PopulationError::WorldHasNoCells);
        }
        definition.validate(
            config.initial_population,
            config.max_person_records,
            world,
            demography,
        )?;

        let person_count = definition.people.len();
        let mut birth_days = Vec::with_capacity(person_count);
        let death_days = vec![NO_EVENT_DAY; person_count];
        // Pre-run reproductive-history timing is intentionally not forged into this u64 model-time
        // field. M2 consults the immutable founder definition until a model-period birth supersedes
        // that history.
        let last_birth_days = vec![NO_EVENT_DAY; person_count];
        let mut reproductive_sexes = Vec::with_capacity(person_count);
        let mut locations = Vec::with_capacity(person_count);
        let mut households = Vec::with_capacity(person_count);
        let mut female_parents = Vec::with_capacity(person_count);
        let mut male_parents = Vec::with_capacity(person_count);
        let mut condition_permille = Vec::with_capacity(person_count);
        let condition_loss_remainder_thousandths = vec![0; person_count];
        let household_locations: Vec<_> = definition
            .households
            .iter()
            .map(|household| household.location)
            .collect();

        for person in &definition.people {
            let household_index = usize::try_from(person.household.0 - 1)
                .expect("validated founder household ID must fit usize");
            let location = household_locations[household_index];
            birth_days.push(person.birth_day);
            reproductive_sexes.push(person.reproductive_sex);
            locations.push(location);
            households.push(person.household);
            female_parents.push(person.female_parent.unwrap_or(PersonId::INVALID));
            male_parents.push(person.male_parent.unwrap_or(PersonId::INVALID));
            condition_permille.push(person.condition_permille);
        }

        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;
        let population = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initial_population: config.initial_population,
            births_since_start: 0,
            deaths_since_start: 0,
            max_person_records: config.max_person_records,
            birth_days,
            death_days,
            last_birth_days,
            reproductive_sexes,
            locations,
            households,
            female_parents,
            male_parents,
            condition_permille,
            condition_loss_remainder_thousandths,
            household_locations,
            occupancy,
        };
        population.validate(world)?;
        Ok(population)
    }

    fn initialize_synthetic_validation_v1(
        config: PopulationConfig,
        world: &World,
        rng_factory: RngFactory,
    ) -> Result<Self, PopulationError> {
        let person_count = usize::try_from(config.initial_population)
            .expect("u32 population count must fit supported usize targets");
        let household_size = usize::from(config.target_household_size);
        let household_count = if person_count == 0 {
            0
        } else {
            person_count.div_ceil(household_size)
        };

        let mut household_rng = rng_factory.stream("demography/init/household_location");
        let mut age_rng = rng_factory.stream("demography/init/age");
        let mut sex_rng = rng_factory.stream("demography/init/sex");

        let world_cell_count =
            u64::try_from(world.cell_count()).expect("supported world cell count must fit u64");
        let mut household_locations = Vec::with_capacity(household_count);
        for _ in 0..household_count {
            let location = CellId::new(
                synthetic_validation_modulo_draw(household_rng.next_u64(), world_cell_count) + 1,
            );
            household_locations.push(location);
        }

        let mut birth_days = Vec::with_capacity(person_count);
        let mut death_days = Vec::with_capacity(person_count);
        let mut last_birth_days = Vec::with_capacity(person_count);
        let mut reproductive_sexes = Vec::with_capacity(person_count);
        let mut locations = Vec::with_capacity(person_count);
        let mut households = Vec::with_capacity(person_count);
        let mut female_parents = Vec::with_capacity(person_count);
        let mut male_parents = Vec::with_capacity(person_count);
        let mut condition_permille = Vec::with_capacity(person_count);
        let mut condition_loss_remainder_thousandths = Vec::with_capacity(person_count);

        let max_age_days = u64::from(config.synthetic_max_age_years) * DAYS_PER_YEAR;
        for index in 0..person_count {
            let household_index = index / household_size;
            let household = HouseholdId::new(
                u64::try_from(household_index).expect("household index must fit u64") + 1,
            );
            let location = household_locations[household_index];
            let age_days = if max_age_days == 0 {
                0
            } else {
                synthetic_validation_modulo_draw(age_rng.next_u64(), max_age_days)
            };
            let birth_day = -i64::try_from(age_days).expect("synthetic age range must fit i64");
            let sex_draw =
                synthetic_validation_modulo_draw(sex_rng.next_u64(), u64::from(PERMILLE_MAX))
                    as u16;
            let reproductive_sex = if sex_draw < config.synthetic_male_permille {
                ReproductiveSex::Male
            } else {
                ReproductiveSex::Female
            };

            birth_days.push(birth_day);
            death_days.push(NO_EVENT_DAY);
            last_birth_days.push(NO_EVENT_DAY);
            reproductive_sexes.push(reproductive_sex);
            locations.push(location);
            households.push(household);
            female_parents.push(PersonId::INVALID);
            male_parents.push(PersonId::INVALID);
            condition_permille.push(PERMILLE_MAX);
            condition_loss_remainder_thousandths.push(0);
        }

        let occupancy = CellOccupancy::build(&locations, world.cell_count())?;
        let population = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initial_population: config.initial_population,
            births_since_start: 0,
            deaths_since_start: 0,
            max_person_records: config.max_person_records,
            birth_days,
            death_days,
            last_birth_days,
            reproductive_sexes,
            locations,
            households,
            female_parents,
            male_parents,
            condition_permille,
            condition_loss_remainder_thousandths,
            household_locations,
            occupancy,
        };
        population.validate(world)?;
        Ok(population)
    }

    #[must_use]
    pub fn person_count(&self) -> usize {
        self.birth_days.len()
    }

    #[must_use]
    pub fn living_count(&self) -> u64 {
        self.death_days
            .iter()
            .filter(|&&death_day| death_day == NO_EVENT_DAY)
            .count() as u64
    }

    #[must_use]
    pub fn household_count(&self) -> usize {
        self.household_locations.len()
    }

    #[must_use]
    pub const fn occupancy(&self) -> &CellOccupancy {
        &self.occupancy
    }

    #[must_use]
    pub fn person(&self, id: PersonId) -> Option<PersonSnapshot> {
        let index = person_index(id, self.person_count())?;
        Some(PersonSnapshot {
            id,
            birth_day: self.birth_days[index],
            death_day: optional_event_day(self.death_days[index]),
            last_birth_day: optional_event_day(self.last_birth_days[index]),
            reproductive_sex: self.reproductive_sexes[index],
            location: self.locations[index],
            household: self.households[index],
            female_parent: self.female_parents[index],
            male_parent: self.male_parents[index],
            condition_permille: self.condition_permille[index],
        })
    }

    #[must_use]
    pub fn household_location(&self, id: HouseholdId) -> Option<CellId> {
        let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
        self.household_locations.get(index).copied()
    }

    #[must_use]
    pub fn summary(&self) -> PopulationSummary {
        PopulationSummary {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initial_population: self.initial_population,
            person_records: self.person_count() as u64,
            living_population: self.living_count(),
            births_since_start: self.births_since_start,
            deaths_since_start: self.deaths_since_start,
            household_count: self.household_count() as u64,
            living_occupied_cell_count: self.living_occupied_cell_count(),
            mean_living_condition_permille: self.mean_living_condition_permille(),
            living_below_half_condition: self.living_below_condition(500),
            digest64: self.digest64(),
        }
    }

    #[must_use]
    pub fn record_limit_reached(&self) -> bool {
        self.person_count() as u64 >= self.max_person_records
    }

    #[must_use]
    pub(crate) fn person_id_at_index(&self, index: usize) -> Option<PersonId> {
        (index < self.person_count()).then(|| person_id_from_index(index))
    }

    #[must_use]
    pub(crate) fn is_alive_index(&self, index: usize) -> bool {
        self.death_days.get(index).copied() == Some(NO_EVENT_DAY)
    }

    #[must_use]
    pub(crate) fn reproductive_sex_at_index(&self, index: usize) -> Option<ReproductiveSex> {
        self.reproductive_sexes.get(index).copied()
    }

    #[must_use]
    pub(crate) fn location_at_index(&self, index: usize) -> Option<CellId> {
        self.locations.get(index).copied()
    }

    #[must_use]
    pub(crate) fn household_at_index(&self, index: usize) -> Option<HouseholdId> {
        self.households.get(index).copied()
    }

    #[must_use]
    pub(crate) fn female_parent_at_index(&self, index: usize) -> Option<PersonId> {
        self.female_parents.get(index).copied()
    }

    #[must_use]
    pub(crate) fn male_parent_at_index(&self, index: usize) -> Option<PersonId> {
        self.male_parents.get(index).copied()
    }

    #[must_use]
    pub(crate) fn condition_at_index(&self, index: usize) -> Option<u16> {
        self.condition_permille.get(index).copied()
    }

    #[must_use]
    pub(crate) fn condition_loss_remainder_thousandths_at_index(
        &self,
        index: usize,
    ) -> Option<u16> {
        self.condition_loss_remainder_thousandths
            .get(index)
            .copied()
    }

    pub(crate) fn set_condition_with_loss_remainder_at_index(
        &mut self,
        index: usize,
        condition: u16,
        remainder_thousandths: u16,
    ) -> bool {
        if condition > PERMILLE_MAX
            || remainder_thousandths >= CONDITION_RESPONSE_DENOMINATOR
            || (condition == 0 && remainder_thousandths != 0)
        {
            return false;
        }
        let Some(condition_slot) = self.condition_permille.get_mut(index) else {
            return false;
        };
        let Some(remainder_slot) = self.condition_loss_remainder_thousandths.get_mut(index) else {
            return false;
        };
        *condition_slot = condition;
        *remainder_slot = remainder_thousandths;
        true
    }

    pub(crate) fn set_condition_at_index(&mut self, index: usize, condition: u16) -> bool {
        self.set_condition_with_loss_remainder_at_index(index, condition, 0)
    }

    #[must_use]
    pub(crate) fn last_birth_day_at_index(&self, index: usize) -> Option<u64> {
        self.last_birth_days
            .get(index)
            .copied()
            .and_then(optional_event_day)
    }

    #[must_use]
    pub(crate) fn age_days_at_index(&self, index: usize, current_day: u64) -> Option<u64> {
        let birth_day = *self.birth_days.get(index)?;
        let current_day = i64::try_from(current_day).ok()?;
        u64::try_from(current_day.checked_sub(birth_day)?).ok()
    }

    pub(crate) fn mark_death(&mut self, index: usize, day: u64) -> bool {
        let Some(death_day) = self.death_days.get_mut(index) else {
            return false;
        };
        if *death_day != NO_EVENT_DAY {
            return false;
        }
        *death_day = day;
        self.deaths_since_start = self.deaths_since_start.saturating_add(1);
        true
    }

    pub(crate) fn note_successful_birth(&mut self, female_parent_index: usize, day: u64) {
        if let Some(last_birth_day) = self.last_birth_days.get_mut(female_parent_index) {
            *last_birth_day = day;
        }
    }

    pub(crate) fn append_birth(
        &mut self,
        day: u64,
        reproductive_sex: ReproductiveSex,
        location: CellId,
        household: HouseholdId,
        female_parent: PersonId,
        male_parent: PersonId,
    ) -> Result<PersonId, PopulationError> {
        if self.record_limit_reached() {
            return Err(PopulationError::PersonRecordLimitReached {
                limit: self.max_person_records,
            });
        }
        let birth_day =
            i64::try_from(day).map_err(|_| PopulationError::SimulationDayTooLarge { day })?;
        let id = person_id_from_index(self.person_count());
        self.birth_days.push(birth_day);
        self.death_days.push(NO_EVENT_DAY);
        self.last_birth_days.push(NO_EVENT_DAY);
        self.reproductive_sexes.push(reproductive_sex);
        self.locations.push(location);
        self.households.push(household);
        self.female_parents.push(female_parent);
        self.male_parents.push(male_parent);
        self.condition_permille.push(PERMILLE_MAX);
        self.condition_loss_remainder_thousandths.push(0);
        self.births_since_start = self.births_since_start.saturating_add(1);
        Ok(id)
    }

    /// Derives deterministic, PersonId-independent relationship-role classes for the living
    /// members of one source household. Refinement starts from age/sex state and repeatedly
    /// incorporates parent state plus the multiset of living in-household child roles until the
    /// partition is stable. PersonId can therefore remain a final deterministic tie-break only
    /// after the declared relationship rule can no longer distinguish two records.
    fn household_relationship_ranks(&self, living_members: &[usize]) -> Vec<u64> {
        let person_count = self.person_count();
        let mut member_positions = vec![usize::MAX; person_count];
        for (position, &member_index) in living_members.iter().enumerate() {
            member_positions[member_index] = position;
        }

        let mut child_links = vec![Vec::<(u8, usize)>::new(); living_members.len()];
        for &child_index in living_members {
            for (role, parent) in [
                (0_u8, self.female_parents[child_index]),
                (1_u8, self.male_parents[child_index]),
            ] {
                if parent == PersonId::INVALID {
                    continue;
                }
                if let Some(parent_index) = person_index(parent, person_count) {
                    let position = member_positions[parent_index];
                    if position != usize::MAX {
                        child_links[position].push((role, child_index));
                    }
                }
            }
        }

        let sex_rank = |index: usize| match self.reproductive_sexes[index] {
            ReproductiveSex::Female => 0_u8,
            ReproductiveSex::Male => 1_u8,
        };
        let mut base_keys = living_members
            .iter()
            .map(|&index| (self.birth_days[index], sex_rank(index)))
            .collect::<Vec<_>>();
        base_keys.sort_unstable();
        base_keys.dedup();

        let mut ranks = vec![u64::MAX; person_count];
        for &index in living_members {
            let position = base_keys
                .binary_search(&(self.birth_days[index], sex_rank(index)))
                .expect("living household member base role must be represented");
            ranks[index] = u64::try_from(position)
                .expect("living household relationship-rank space must fit u64");
        }

        for _ in 0..living_members.len() {
            let signatures = living_members
                .iter()
                .enumerate()
                .map(|(member_position, &index)| {
                    let parent_signature = |parent: PersonId| -> (u8, u64) {
                        if parent == PersonId::INVALID {
                            return (0, 0);
                        }
                        let Some(parent_index) = person_index(parent, person_count) else {
                            return (1, 0);
                        };
                        let position = member_positions[parent_index];
                        if position != usize::MAX {
                            (3, ranks[parent_index])
                        } else if self.is_alive_index(parent_index) {
                            (2, 0)
                        } else {
                            (1, 0)
                        }
                    };
                    let mut children = child_links[member_position]
                        .iter()
                        .map(|&(role, child_index)| (role, ranks[child_index]))
                        .collect::<Vec<_>>();
                    children.sort_unstable();
                    (
                        index,
                        (
                            ranks[index],
                            parent_signature(self.female_parents[index]),
                            parent_signature(self.male_parents[index]),
                            children,
                        ),
                    )
                })
                .collect::<Vec<_>>();

            let mut unique_signatures = signatures
                .iter()
                .map(|(_, signature)| signature.clone())
                .collect::<Vec<_>>();
            unique_signatures.sort();
            unique_signatures.dedup();

            let mut next_ranks = ranks.clone();
            for (index, signature) in signatures {
                let position = unique_signatures
                    .binary_search(&signature)
                    .expect("relationship signature must be represented");
                next_ranks[index] = u64::try_from(position)
                    .expect("living household relationship-rank space must fit u64");
            }
            if living_members
                .iter()
                .all(|&index| next_ranks[index] == ranks[index])
            {
                return ranks;
            }
            ranks = next_ranks;
        }
        ranks
    }

    pub(crate) fn fission_oversized_households(
        &mut self,
        max_living_members: u16,
        minimum_independent_age_years: u16,
        current_day: u64,
        eligible_households: &[bool],
    ) -> Result<HouseholdFissionOutcome, PopulationError> {
        if max_living_members == 0 {
            return Err(PopulationError::ZeroLifecycleHouseholdSize);
        }
        if minimum_independent_age_years == 0 {
            return Err(PopulationError::ZeroLifecycleIndependentAge);
        }
        let original_household_count = self.household_count();
        if eligible_households.len() != original_household_count {
            return Err(PopulationError::HouseholdLifecycleShapeMismatch);
        }
        let minimum_independent_age_days =
            u64::from(minimum_independent_age_years).saturating_mul(365);
        let ceiling = usize::from(max_living_members);
        let mut outcome = HouseholdFissionOutcome::default();

        for (household_index, &is_eligible) in eligible_households.iter().enumerate() {
            if !is_eligible {
                continue;
            }
            let household = HouseholdId::new(
                u64::try_from(household_index)
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?,
            );
            let living_members = (0..self.person_count())
                .filter(|&person_index| {
                    self.is_alive_index(person_index) && self.households[person_index] == household
                })
                .collect::<Vec<_>>();
            if living_members.len() <= ceiling {
                continue;
            }

            let required_groups = living_members.len().div_ceil(ceiling);
            let mut independent = living_members
                .iter()
                .copied()
                .filter(|&person_index| {
                    self.age_days_at_index(person_index, current_day)
                        .is_some_and(|age| age >= minimum_independent_age_days)
                })
                .collect::<Vec<_>>();
            let relationship_ranks = self.household_relationship_ranks(&living_members);
            independent.sort_by_key(|&index| {
                (
                    self.birth_days[index],
                    match self.reproductive_sexes[index] {
                        ReproductiveSex::Female => 0_u8,
                        ReproductiveSex::Male => 1_u8,
                    },
                    relationship_ranks[index],
                    person_id_from_index(index).0,
                )
            });
            let group_count = required_groups.min(independent.len());
            if group_count < 2 {
                continue;
            }

            let base_group_size = living_members.len() / group_count;
            let larger_group_count = living_members.len() % group_count;
            let target_sizes = (0..group_count)
                .map(|group_index| base_group_size + usize::from(group_index < larger_group_count))
                .collect::<Vec<_>>();
            let mut groups = vec![Vec::<usize>::new(); group_count];
            let mut assigned_group = vec![None; self.person_count()];

            for (ordinal, &person_index) in independent.iter().enumerate() {
                let group_index = ordinal % group_count;
                groups[group_index].push(person_index);
                assigned_group[person_index] = Some(group_index);
            }

            let mut dependents = living_members
                .iter()
                .copied()
                .filter(|&index| assigned_group[index].is_none())
                .collect::<Vec<_>>();
            dependents.sort_by_key(|&index| {
                (
                    self.birth_days[index],
                    match self.reproductive_sexes[index] {
                        ReproductiveSex::Female => 0_u8,
                        ReproductiveSex::Male => 1_u8,
                    },
                    relationship_ranks[index],
                    person_id_from_index(index).0,
                )
            });

            for member_index in dependents {
                let mut parent_groups = Vec::with_capacity(2);
                for parent in [
                    self.female_parents[member_index],
                    self.male_parents[member_index],
                ] {
                    if parent == PersonId::INVALID {
                        continue;
                    }
                    if let Some(parent_index_value) = person_index(parent, self.person_count())
                        && self.is_alive_index(parent_index_value)
                        && self.households[parent_index_value] == household
                        && let Some(group_index) = assigned_group[parent_index_value]
                        && !parent_groups.contains(&group_index)
                    {
                        parent_groups.push(group_index);
                    }
                }
                let candidates = if parent_groups.is_empty() {
                    (0..group_count).collect::<Vec<_>>()
                } else {
                    parent_groups
                };
                let group_index = candidates
                    .into_iter()
                    .max_by_key(|&candidate| {
                        (
                            target_sizes[candidate].saturating_sub(groups[candidate].len()),
                            std::cmp::Reverse(candidate),
                        )
                    })
                    .expect("dependency-safe fission must have at least one anchored group");
                groups[group_index].push(member_index);
                assigned_group[member_index] = Some(group_index);
            }

            debug_assert_eq!(
                groups.iter().map(Vec::len).sum::<usize>(),
                living_members.len()
            );
            debug_assert!(groups.iter().all(|group| group.iter().any(|&member_index| {
                self.age_days_at_index(member_index, current_day)
                    .is_some_and(|age| age >= minimum_independent_age_days)
            })));

            let residence = self.household_locations[household_index];
            for group in groups.iter().skip(1) {
                let new_household_raw = u64::try_from(self.household_locations.len())
                    .map_err(|_| PopulationError::HouseholdIdSpaceExhausted)?
                    .checked_add(1)
                    .ok_or(PopulationError::HouseholdIdSpaceExhausted)?;
                let new_household = HouseholdId::new(new_household_raw);
                self.household_locations.push(residence);
                let mut reassigned = Vec::with_capacity(group.len());
                for &member_index in group {
                    self.households[member_index] = new_household;
                    reassigned.push(person_id_from_index(member_index));
                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);
                }
                outcome.households_created = outcome.households_created.saturating_add(1);
                outcome.fissions.push(HouseholdFissionRecord {
                    source_household: household,
                    new_household,
                    residence,
                    people_reassigned: reassigned,
                });
            }
        }
        Ok(outcome)
    }

    pub(crate) fn apply_household_relocations(
        &mut self,
        destinations: &[CellId],
        condition_costs: &[u16],
        world: &World,
    ) -> Result<HouseholdRelocationOutcome, PopulationError> {
        if destinations.len() != self.household_count()
            || condition_costs.len() != self.household_count()
        {
            return Err(PopulationError::RelocationShapeMismatch);
        }
        for (household_index, &destination) in destinations.iter().enumerate() {
            if destination == CellId::INVALID {
                continue;
            }
            if world.cell(destination).is_none() {
                return Err(PopulationError::InvalidRelocationDestination { destination });
            }
            self.household_locations[household_index] = destination;
        }

        let mut people_moved = 0_u64;
        let mut condition_loss_total = 0_u64;
        for index in 0..self.person_count() {
            if !self.is_alive_index(index) {
                continue;
            }
            let household = self.households[index];
            let household_index = usize::try_from(
                household
                    .0
                    .checked_sub(1)
                    .ok_or(PopulationError::RelocationShapeMismatch)?,
            )
            .map_err(|_| PopulationError::RelocationShapeMismatch)?;
            let destination = destinations
                .get(household_index)
                .copied()
                .ok_or(PopulationError::RelocationShapeMismatch)?;
            if destination == CellId::INVALID {
                continue;
            }
            self.locations[index] = destination;
            let before = self.condition_permille[index];
            let after = before.saturating_sub(condition_costs[household_index]);
            self.condition_permille[index] = after;
            if after == 0 {
                self.condition_loss_remainder_thousandths[index] = 0;
            }
            people_moved = people_moved.saturating_add(1);
            condition_loss_total =
                condition_loss_total.saturating_add(u64::from(before.saturating_sub(after)));
        }
        self.rebuild_occupancy(world)?;
        Ok(HouseholdRelocationOutcome {
            people_moved,
            condition_loss_total,
        })
    }

    pub(crate) fn rebuild_occupancy(&mut self, world: &World) -> Result<(), PopulationError> {
        self.occupancy = CellOccupancy::build(&self.locations, world.cell_count())?;
        Ok(())
    }

    pub fn validate(&self, world: &World) -> Result<(), PopulationValidationError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(PopulationValidationError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }

        let person_count = self.person_count();
        let lengths = [
            self.death_days.len(),
            self.last_birth_days.len(),
            self.reproductive_sexes.len(),
            self.locations.len(),
            self.households.len(),
            self.female_parents.len(),
            self.male_parents.len(),
            self.condition_permille.len(),
            self.condition_loss_remainder_thousandths.len(),
        ];
        if lengths.iter().any(|&length| length != person_count) {
            return Err(PopulationValidationError::ColumnLengthMismatch);
        }

        let records = person_count as u64;
        let expected_records = u64::from(self.initial_population)
            .checked_add(self.births_since_start)
            .ok_or(PopulationValidationError::PopulationAccountingOverflow)?;
        if records != expected_records {
            return Err(PopulationValidationError::PersonRecordAccountingMismatch {
                records,
                expected: expected_records,
            });
        }
        let expected_living = expected_records
            .checked_sub(self.deaths_since_start)
            .ok_or(PopulationValidationError::PopulationAccountingOverflow)?;
        let living = self.living_count();
        if living != expected_living {
            return Err(
                PopulationValidationError::LivingPopulationAccountingMismatch {
                    living,
                    expected: expected_living,
                },
            );
        }
        if records > self.max_person_records {
            return Err(PopulationValidationError::PersonRecordLimitExceeded {
                records,
                limit: self.max_person_records,
            });
        }

        for index in 0..person_count {
            let person = person_id_from_index(index);
            let location = self.locations[index];
            if world.cell(location).is_none() {
                return Err(PopulationValidationError::InvalidPersonLocation { person, location });
            }

            let household = self.households[index];
            let household_location = self
                .household_location(household)
                .ok_or(PopulationValidationError::InvalidPersonHousehold { person, household })?;
            if self.is_alive_index(index) && household_location != location {
                return Err(PopulationValidationError::HouseholdLocationMismatch {
                    person,
                    household,
                    person_location: location,
                    household_location,
                });
            }

            if self.condition_permille[index] > PERMILLE_MAX {
                return Err(PopulationValidationError::InvalidCondition {
                    person,
                    condition: self.condition_permille[index],
                });
            }
            let remainder = self.condition_loss_remainder_thousandths[index];
            if remainder >= CONDITION_RESPONSE_DENOMINATOR
                || (self.condition_permille[index] == 0 && remainder != 0)
            {
                return Err(PopulationValidationError::InvalidConditionLossRemainder {
                    person,
                    condition: self.condition_permille[index],
                    remainder_thousandths: remainder,
                });
            }

            let female_parent = self.female_parents[index];
            let male_parent = self.male_parents[index];
            if female_parent == person || male_parent == person {
                return Err(PopulationValidationError::SelfParent { person });
            }
            if female_parent != PersonId::INVALID && female_parent == male_parent {
                return Err(PopulationValidationError::DuplicateParents { person });
            }
            self.validate_parent(person, index, female_parent, ReproductiveSex::Female)?;
            self.validate_parent(person, index, male_parent, ReproductiveSex::Male)?;

            let death_day = self.death_days[index];
            if death_day != NO_EVENT_DAY && self.birth_days[index] >= 0 {
                let birth_day = u64::try_from(self.birth_days[index])
                    .expect("non-negative birth day must fit u64");
                // Newborns are not exposed to mortality until a later annual boundary.
                if death_day <= birth_day {
                    return Err(PopulationValidationError::DeathBeforeBirth {
                        person,
                        birth_day: self.birth_days[index],
                        death_day,
                    });
                }
            }

            let last_birth_day = self.last_birth_days[index];
            if last_birth_day != NO_EVENT_DAY {
                if self.reproductive_sexes[index] != ReproductiveSex::Female {
                    return Err(PopulationValidationError::BirthHistoryOnNonFemale { person });
                }
                if self.birth_days[index] >= 0 {
                    let own_birth_day = u64::try_from(self.birth_days[index])
                        .expect("non-negative birth day must fit u64");
                    if last_birth_day <= own_birth_day {
                        return Err(PopulationValidationError::BirthHistoryBeforeOwnBirth {
                            person,
                            own_birth_day,
                            recorded_birth_day: last_birth_day,
                        });
                    }
                }
                // Mortality precedes fertility on the same boundary, so equality is impossible.
                if death_day != NO_EVENT_DAY && last_birth_day >= death_day {
                    return Err(PopulationValidationError::BirthAfterDeath {
                        person,
                        birth_day: last_birth_day,
                        death_day,
                    });
                }
            }
        }

        // `last_birth_days` is persisted fertility state used by future spacing decisions. It
        // must therefore agree with the latest post-run child record, not merely be locally
        // plausible in isolation.
        let mut latest_recorded_child_birth_days = vec![NO_EVENT_DAY; person_count];
        for child_index in 0..person_count {
            let female_parent = self.female_parents[child_index];
            if female_parent == PersonId::INVALID || self.birth_days[child_index] < 0 {
                continue;
            }
            let parent_index = person_index(female_parent, person_count).ok_or(
                PopulationValidationError::InvalidParent {
                    person: person_id_from_index(child_index),
                    parent: female_parent,
                },
            )?;
            let child_birth_day = u64::try_from(self.birth_days[child_index])
                .expect("non-negative child birth day must fit u64");
            let latest = &mut latest_recorded_child_birth_days[parent_index];
            if *latest == NO_EVENT_DAY || child_birth_day > *latest {
                *latest = child_birth_day;
            }
        }
        for (index, &expected) in latest_recorded_child_birth_days.iter().enumerate() {
            let recorded = self.last_birth_days[index];
            if recorded != expected {
                return Err(PopulationValidationError::BirthHistoryMismatch {
                    person: person_id_from_index(index),
                    recorded_birth_day: optional_event_day(recorded),
                    latest_child_birth_day: optional_event_day(expected),
                });
            }
        }

        self.validate_occupancy(world)
    }

    fn validate_parent(
        &self,
        person: PersonId,
        child_index: usize,
        parent: PersonId,
        expected_sex: ReproductiveSex,
    ) -> Result<(), PopulationValidationError> {
        if parent == PersonId::INVALID {
            return Ok(());
        }
        let parent_index = person_index(parent, self.person_count())
            .ok_or(PopulationValidationError::InvalidParent { person, parent })?;
        if self.reproductive_sexes[parent_index] != expected_sex {
            return Err(PopulationValidationError::ParentSexMismatch {
                person,
                parent,
                expected: expected_sex,
                actual: self.reproductive_sexes[parent_index],
            });
        }
        if self.birth_days[parent_index] >= self.birth_days[child_index] {
            return Err(PopulationValidationError::ParentNotOlder { person, parent });
        }
        let child_birth_day = self.birth_days[child_index];
        let parent_death_day = self.death_days[parent_index];
        if child_birth_day >= 0 && parent_death_day != NO_EVENT_DAY {
            let child_birth_day =
                u64::try_from(child_birth_day).expect("non-negative birth day must fit u64");
            // Mortality is processed before fertility at an annual boundary, so a parent who
            // dies on the child's birth day is already ineligible for that birth.
            if parent_death_day <= child_birth_day {
                return Err(PopulationValidationError::ParentDeadBeforeBirth {
                    person,
                    parent,
                    parent_death_day,
                    child_birth_day,
                });
            }
        }
        Ok(())
    }

    fn validate_occupancy(&self, world: &World) -> Result<(), PopulationValidationError> {
        let expected_offsets = world.cell_count().saturating_add(1);
        if self.occupancy.offsets.len() != expected_offsets
            || self.occupancy.offsets.first().copied() != Some(0)
            || self.occupancy.offsets.last().copied() != Some(self.person_count() as u64)
            || self.occupancy.people.len() != self.person_count()
            || self
                .occupancy
                .offsets
                .windows(2)
                .any(|window| window[0] > window[1])
        {
            return Err(PopulationValidationError::OccupancyShape {
                reason: "offset/person arrays are inconsistent",
            });
        }

        let mut seen = vec![false; self.person_count()];
        for cell_index in 0..world.cell_count() {
            let cell = CellId::new(cell_index as u64 + 1);
            let people = self.occupancy.people_in_cell(cell).ok_or(
                PopulationValidationError::OccupancyShape {
                    reason: "valid cell is absent from occupancy index",
                },
            )?;
            for &person in people {
                let person_index = person_index(person, self.person_count())
                    .ok_or(PopulationValidationError::InvalidOccupancyPerson { cell, person })?;
                if seen[person_index] {
                    return Err(PopulationValidationError::DuplicateOccupancyPerson { person });
                }
                seen[person_index] = true;
                if self.locations[person_index] != cell {
                    return Err(PopulationValidationError::OccupancyLocationMismatch {
                        person,
                        indexed_cell: cell,
                        actual_cell: self.locations[person_index],
                    });
                }
            }
        }

        if let Some(index) = seen.iter().position(|&was_seen| !was_seen) {
            return Err(PopulationValidationError::MissingOccupancyPerson {
                person: person_id_from_index(index),
            });
        }
        Ok(())
    }

    #[must_use]
    fn living_occupied_cell_count(&self) -> u64 {
        let mut count = 0_u64;
        for cell_index in 0..self.occupancy.offsets.len().saturating_sub(1) {
            let cell = CellId::new(cell_index as u64 + 1);
            if self.occupancy.people_in_cell(cell).is_some_and(|people| {
                people.iter().any(|&person| {
                    person_index(person, self.person_count())
                        .is_some_and(|index| self.is_alive_index(index))
                })
            }) {
                count = count.saturating_add(1);
            }
        }
        count
    }

    #[must_use]
    pub fn mean_living_condition_permille(&self) -> Option<u16> {
        let mut total = 0_u64;
        let mut count = 0_u64;
        for index in 0..self.person_count() {
            if self.is_alive_index(index) {
                total = total.saturating_add(u64::from(self.condition_permille[index]));
                count = count.saturating_add(1);
            }
        }
        let mean = total.checked_div(count)?;
        Some(u16::try_from(mean).unwrap_or(PERMILLE_MAX))
    }

    #[must_use]
    pub fn living_below_condition(&self, threshold_permille: u16) -> u64 {
        self.condition_permille
            .iter()
            .enumerate()
            .filter(|(index, condition)| {
                self.is_alive_index(*index) && **condition < threshold_permille
            })
            .count() as u64
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_u64(&mut hash, u64::from(self.initial_population));
        digest_u64(&mut hash, self.births_since_start);
        digest_u64(&mut hash, self.deaths_since_start);
        digest_u64(&mut hash, self.max_person_records);
        digest_u64(&mut hash, self.person_count() as u64);
        for index in 0..self.person_count() {
            digest_i64(&mut hash, self.birth_days[index]);
            digest_u64(&mut hash, self.death_days[index]);
            digest_u64(&mut hash, self.last_birth_days[index]);
            digest_u64(
                &mut hash,
                match self.reproductive_sexes[index] {
                    ReproductiveSex::Female => 0,
                    ReproductiveSex::Male => 1,
                },
            );
            digest_u64(&mut hash, self.locations[index].0);
            digest_u64(&mut hash, self.households[index].0);
            digest_u64(&mut hash, self.female_parents[index].0);
            digest_u64(&mut hash, self.male_parents[index].0);
            digest_u64(&mut hash, u64::from(self.condition_permille[index]));
            digest_u64(
                &mut hash,
                u64::from(self.condition_loss_remainder_thousandths[index]),
            );
        }
        digest_u64(&mut hash, self.household_count() as u64);
        for &location in &self.household_locations {
            digest_u64(&mut hash, location.0);
        }
        hash
    }
}

fn validate_config(config: PopulationConfig) -> Result<(), PopulationError> {
    if config.schema_version != PopulationConfig::CURRENT_SCHEMA_VERSION {
        return Err(PopulationError::UnsupportedPopulationSchema {
            found: config.schema_version,
            supported: PopulationConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.initialization == PopulationInitialization::SyntheticValidationV1 {
        if config.target_household_size == 0 {
            return Err(PopulationError::ZeroHouseholdSize);
        }
        if config.synthetic_male_permille > PERMILLE_MAX {
            return Err(PopulationError::InvalidMalePermille {
                value: config.synthetic_male_permille,
            });
        }
    }
    if u64::from(config.initial_population) > config.max_person_records {
        return Err(PopulationError::InitialPopulationExceedsRecordLimit {
            initial_population: config.initial_population,
            limit: config.max_person_records,
        });
    }
    Ok(())
}

/// Frozen synthetic-validation mapping used by the founder preset.
///
/// Modulo reduction can differ by one source value between buckets when `upper_exclusive` does
/// not divide the 2^64 RNG domain. That negligible bias is not an empirical sampling claim and
/// changing the mapping would churn historical deterministic reference outputs, so it is kept
/// explicit and regression-tested rather than silently replaced.
fn synthetic_validation_modulo_draw(raw: u64, upper_exclusive: u64) -> u64 {
    debug_assert!(upper_exclusive > 0);
    raw % upper_exclusive
}

fn person_id_from_index(index: usize) -> PersonId {
    PersonId::new(u64::try_from(index).expect("person index must fit u64") + 1)
}

fn person_index(id: PersonId, person_count: usize) -> Option<usize> {
    let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
    (index < person_count).then_some(index)
}

fn location_index(id: CellId, cell_count: usize) -> Option<usize> {
    let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
    (index < cell_count).then_some(index)
}

fn optional_event_day(day: u64) -> Option<u64> {
    (day != NO_EVENT_DAY).then_some(day)
}

fn digest_u64(hash: &mut u64, value: u64) {
    digest_bytes(hash, &value.to_le_bytes());
}

fn digest_i64(hash: &mut u64, value: i64) {
    digest_bytes(hash, &value.to_le_bytes());
}

fn digest_bytes(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

#[derive(Debug, Error)]
pub enum PopulationError {
    #[error("population schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedPopulationSchema { found: u32, supported: u32 },
    #[error("target household size must be greater than zero")]
    ZeroHouseholdSize,
    #[error("synthetic male share {value} permille is outside 0..=1000")]
    InvalidMalePermille { value: u16 },
    #[error("initial population {initial_population} exceeds persistent record limit {limit}")]
    InitialPopulationExceedsRecordLimit { initial_population: u32, limit: u64 },
    #[error("declared founder initialization requires an explicit founder definition")]
    DeclaredFounderDefinitionRequired,
    #[error("declared founder definition supplied without declared founder initialization mode")]
    DeclaredFounderModeNotSelected,
    #[error("persistent person record limit {limit} has been reached")]
    PersonRecordLimitReached { limit: u64 },
    #[error("household relocation arrays do not match the population household layout")]
    RelocationShapeMismatch,
    #[error("household lifecycle eligibility does not match the population household layout")]
    HouseholdLifecycleShapeMismatch,
    #[error("household lifecycle maximum living size must be greater than zero")]
    ZeroLifecycleHouseholdSize,
    #[error("household lifecycle minimum independent age must be greater than zero")]
    ZeroLifecycleIndependentAge,
    #[error("household identity space is exhausted")]
    HouseholdIdSpaceExhausted,
    #[error("household relocation destination {destination:?} is outside the world")]
    InvalidRelocationDestination { destination: CellId },
    #[error("simulation day {day} cannot be represented as an epoch-relative signed birth day")]
    SimulationDayTooLarge { day: u64 },
    #[error("internal population invariant failed: {reason}")]
    InternalInvariant { reason: &'static str },
    #[error("cannot initialize a population into a world with no cells")]
    WorldHasNoCells,
    #[error(transparent)]
    FounderPopulation(#[from] FounderPopulationError),
    #[error(transparent)]
    Validation(#[from] PopulationValidationError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PopulationValidationError {
    #[error("population schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("population structure-of-arrays columns have different lengths")]
    ColumnLengthMismatch,
    #[error("population accounting overflowed")]
    PopulationAccountingOverflow,
    #[error("persistent person records {records} do not match expected {expected}")]
    PersonRecordAccountingMismatch { records: u64, expected: u64 },
    #[error("living population {living} does not match expected {expected}")]
    LivingPopulationAccountingMismatch { living: u64, expected: u64 },
    #[error("persistent person records {records} exceed operational limit {limit}")]
    PersonRecordLimitExceeded { records: u64, limit: u64 },
    #[error("person {person:?} references invalid location {location:?}")]
    InvalidPersonLocation { person: PersonId, location: CellId },
    #[error("person {person:?} references invalid household {household:?}")]
    InvalidPersonHousehold {
        person: PersonId,
        household: HouseholdId,
    },
    #[error(
        "person {person:?} is in {person_location:?} but household {household:?} is in {household_location:?}"
    )]
    HouseholdLocationMismatch {
        person: PersonId,
        household: HouseholdId,
        person_location: CellId,
        household_location: CellId,
    },
    #[error("person {person:?} has invalid condition {condition} permille")]
    InvalidCondition { person: PersonId, condition: u16 },
    #[error(
        "person {person:?} has invalid M3 condition-loss remainder {remainder_thousandths}/1000 at condition {condition}"
    )]
    InvalidConditionLossRemainder {
        person: PersonId,
        condition: u16,
        remainder_thousandths: u16,
    },
    #[error("person {person:?} is their own parent")]
    SelfParent { person: PersonId },
    #[error("person {person:?} has the same non-null parent in both parent roles")]
    DuplicateParents { person: PersonId },
    #[error("person {person:?} references invalid parent {parent:?}")]
    InvalidParent { person: PersonId, parent: PersonId },
    #[error(
        "person {person:?} parent {parent:?} has incompatible reproductive sex: expected {expected:?}, found {actual:?}"
    )]
    ParentSexMismatch {
        person: PersonId,
        parent: PersonId,
        expected: ReproductiveSex,
        actual: ReproductiveSex,
    },
    #[error("person {person:?} parent {parent:?} is not older than the child")]
    ParentNotOlder { person: PersonId, parent: PersonId },
    #[error(
        "person {person:?} parent {parent:?} died on day {parent_death_day} before or on child birth day {child_birth_day}"
    )]
    ParentDeadBeforeBirth {
        person: PersonId,
        parent: PersonId,
        parent_death_day: u64,
        child_birth_day: u64,
    },
    #[error("person {person:?} dies on day {death_day} before or on birth day {birth_day}")]
    DeathBeforeBirth {
        person: PersonId,
        birth_day: i64,
        death_day: u64,
    },
    #[error("person {person:?} has a birth-history event despite non-female reproductive sex")]
    BirthHistoryOnNonFemale { person: PersonId },
    #[error(
        "person {person:?} records a birth on day {recorded_birth_day} before or on own birth day {own_birth_day}"
    )]
    BirthHistoryBeforeOwnBirth {
        person: PersonId,
        own_birth_day: u64,
        recorded_birth_day: u64,
    },
    #[error("person {person:?} has birth on day {birth_day} on or after death on day {death_day}")]
    BirthAfterDeath {
        person: PersonId,
        birth_day: u64,
        death_day: u64,
    },
    #[error(
        "person {person:?} birth-history day {recorded_birth_day:?} does not match latest recorded child birth day {latest_child_birth_day:?}"
    )]
    BirthHistoryMismatch {
        person: PersonId,
        recorded_birth_day: Option<u64>,
        latest_child_birth_day: Option<u64>,
    },
    #[error("invalid occupancy index: {reason}")]
    OccupancyShape { reason: &'static str },
    #[error("cell {cell:?} occupancy references invalid person {person:?}")]
    InvalidOccupancyPerson { cell: CellId, person: PersonId },
    #[error("person {person:?} occurs more than once in occupancy index")]
    DuplicateOccupancyPerson { person: PersonId },
    #[error(
        "person {person:?} is indexed in {indexed_cell:?} but authoritative location is {actual_cell:?}"
    )]
    OccupancyLocationMismatch {
        person: PersonId,
        indexed_cell: CellId,
        actual_cell: CellId,
    },
    #[error("person {person:?} is absent from occupancy index")]
    MissingOccupancyPerson { person: PersonId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{ParameterProvenance, WorldConfig},
        founder_initialization::{
            FounderGenealogyStatus, FounderHousehold, FounderPerson, FounderPopulationDefinition,
        },
    };

    fn test_world(seed: u64) -> World {
        World::generate(WorldConfig::new(32, 24), RngFactory::new(seed)).unwrap()
    }

    fn declared_definition() -> FounderPopulationDefinition {
        FounderPopulationDefinition::new(
            "population-materialization-test-v1",
            ParameterProvenance::SyntheticValidation,
            FounderGenealogyStatus::CompleteLivingDirectParents,
            vec![
                FounderHousehold {
                    id: HouseholdId::new(1),
                    location: CellId::new(1),
                },
                FounderHousehold {
                    id: HouseholdId::new(2),
                    location: CellId::new(2),
                },
            ],
            vec![
                FounderPerson {
                    id: PersonId::new(1),
                    birth_day: -18_250,
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(2),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 800,
                },
                FounderPerson {
                    id: PersonId::new(2),
                    birth_day: -17_885,
                    reproductive_sex: ReproductiveSex::Male,
                    household: HouseholdId::new(2),
                    female_parent: None,
                    male_parent: None,
                    last_birth_day: None,
                    condition_permille: 900,
                },
                FounderPerson {
                    id: PersonId::new(3),
                    birth_day: -9_125,
                    reproductive_sex: ReproductiveSex::Female,
                    household: HouseholdId::new(1),
                    female_parent: Some(PersonId::new(1)),
                    male_parent: Some(PersonId::new(2)),
                    last_birth_day: Some(-100),
                    condition_permille: 700,
                },
            ],
        )
    }

    #[test]
    fn initialization_has_exact_population_and_valid_references() {
        let world = test_world(7);
        let config = PopulationConfig::new(10_000).with_target_household_size(5);
        let population = Population::initialize(config, &world, RngFactory::new(7)).unwrap();

        assert_eq!(population.person_count(), 10_000);
        assert_eq!(population.living_count(), 10_000);
        assert_eq!(population.household_count(), 2_000);
        assert_eq!(population.occupancy.people.len(), 10_000);
        population.validate(&world).unwrap();
    }

    #[test]
    fn deserialized_population_rejects_unsupported_schema_versions() {
        let world = test_world(7);
        let population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(7)).unwrap();
        let baseline = serde_json::to_value(population).unwrap();

        for found in [0, Population::CURRENT_SCHEMA_VERSION + 1] {
            let mut value = baseline.clone();
            value["schemaVersion"] = serde_json::json!(found);
            let restored: Population = serde_json::from_value(value).unwrap();

            assert_eq!(
                restored.validate(&world),
                Err(PopulationValidationError::UnsupportedSchema {
                    found,
                    supported: Population::CURRENT_SCHEMA_VERSION,
                })
            );
        }
    }

    #[test]
    fn declared_founder_state_materializes_exactly_and_is_seed_independent() {
        let world = test_world(8);
        let config = PopulationConfig::new(3)
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
        let definition = declared_definition();
        let population = Population::initialize_declared_founder_state_v1(
            config,
            &definition,
            &world,
            &crate::config::DemographyConfig::synthetic_validation_v1(),
        )
        .unwrap();

        assert_eq!(population.person_count(), 3);
        assert_eq!(population.household_count(), 2);
        let founder = population.person(PersonId::new(3)).unwrap();
        assert_eq!(founder.birth_day, -9_125);
        assert_eq!(founder.location, CellId::new(1));
        assert_eq!(founder.household, HouseholdId::new(1));
        assert_eq!(founder.female_parent, PersonId::new(1));
        assert_eq!(founder.male_parent, PersonId::new(2));
        assert_eq!(founder.condition_permille, 700);
        assert_eq!(founder.last_birth_day, None);
        population.validate(&world).unwrap();
    }

    #[test]
    fn declared_mode_cannot_fall_back_to_synthetic_initialization() {
        let world = test_world(9);
        let config = PopulationConfig::new(3)
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1);
        assert!(matches!(
            Population::initialize(config, &world, RngFactory::new(9)),
            Err(PopulationError::DeclaredFounderDefinitionRequired)
        ));
    }

    #[test]
    fn stable_ids_round_trip_to_snapshots() {
        let world = test_world(11);
        let population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(11))
                .unwrap();

        let id = PersonId::new(42);
        let person = population.person(id).unwrap();
        assert_eq!(person.id, id);
        assert!(person.age_days_at(SimTime::ZERO).is_some());
        assert_eq!(person.female_parent, PersonId::INVALID);
        assert_eq!(person.male_parent, PersonId::INVALID);
        assert_eq!(
            population.household_location(person.household),
            Some(person.location)
        );
    }

    #[test]
    fn same_seed_and_config_produce_identical_population() {
        let world = test_world(17);
        let config = PopulationConfig::new(4_096);
        let a = Population::initialize(config, &world, RngFactory::new(17)).unwrap();
        let b = Population::initialize(config, &world, RngFactory::new(17)).unwrap();

        assert_eq!(a, b);
        assert_eq!(a.digest64(), b.digest64());
    }

    #[test]
    fn changing_seed_changes_population_digest() {
        let world = test_world(19);
        let config = PopulationConfig::new(4_096);
        let a = Population::initialize(config, &world, RngFactory::new(19)).unwrap();
        let b = Population::initialize(config, &world, RngFactory::new(20)).unwrap();

        assert_ne!(a.digest64(), b.digest64());
    }

    #[test]
    fn occupancy_reconciles_to_authoritative_locations() {
        let world = test_world(23);
        let population =
            Population::initialize(PopulationConfig::new(1_000), &world, RngFactory::new(23))
                .unwrap();

        let indexed: usize = (1..=world.cell_count())
            .map(|cell| {
                population
                    .occupancy()
                    .people_in_cell(CellId::new(cell as u64))
                    .unwrap()
                    .len()
            })
            .sum();
        assert_eq!(indexed, population.person_count());
    }

    #[test]
    fn death_and_birth_accounting_remain_exact() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(31)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(10), &world, RngFactory::new(31)).unwrap();
        assert!(population.mark_death(0, 365));
        let female = population
            .person_id_at_index(1)
            .expect("founder ID should exist");
        let male = population
            .person_id_at_index(2)
            .expect("founder ID should exist");
        population
            .append_birth(
                365,
                ReproductiveSex::Female,
                CellId::new(1),
                population.household_at_index(1).unwrap(),
                female,
                male,
            )
            .unwrap();
        population.rebuild_occupancy(&world).unwrap();

        assert_eq!(population.person_count(), 11);
        assert_eq!(population.living_count(), 10);
        assert_eq!(population.births_since_start, 1);
        assert_eq!(population.deaths_since_start, 1);
        assert_eq!(
            population.condition_loss_remainder_thousandths.last(),
            Some(&0)
        );
    }

    #[test]
    fn condition_updates_are_reflected_in_summary_and_digest() {
        let world = test_world(41);
        let mut population =
            Population::initialize(PopulationConfig::new(10), &world, RngFactory::new(41)).unwrap();
        let before = population.digest64();
        assert!(population.set_condition_at_index(0, 250));
        assert_eq!(population.condition_at_index(0), Some(250));
        assert!(
            population
                .mean_living_condition_permille()
                .is_some_and(|mean| mean < 1_000)
        );
        assert_eq!(population.living_below_condition(500), 1);
        assert_ne!(before, population.digest64());
    }

    #[test]
    fn fractional_condition_loss_remainder_is_causal_and_validated() {
        let world = test_world(42);
        let mut population =
            Population::initialize(PopulationConfig::new(1), &world, RngFactory::new(42)).unwrap();
        let baseline = population.digest64();
        assert!(population.set_condition_with_loss_remainder_at_index(0, 900, 400));
        assert_eq!(
            population.condition_loss_remainder_thousandths_at_index(0),
            Some(400)
        );
        assert_ne!(population.digest64(), baseline);
        population.validate(&world).unwrap();

        assert!(population.set_condition_at_index(0, 900));
        assert_eq!(
            population.condition_loss_remainder_thousandths_at_index(0),
            Some(0)
        );

        population.condition_loss_remainder_thousandths[0] = 1_000;
        assert!(matches!(
            population.validate(&world),
            Err(PopulationValidationError::InvalidConditionLossRemainder { .. })
        ));
    }

    #[test]
    fn living_condition_mean_distinguishes_undefined_from_true_zero() {
        let world = test_world(43);
        let mut population =
            Population::initialize(PopulationConfig::new(3), &world, RngFactory::new(43)).unwrap();

        for index in 0..population.person_count() {
            assert!(population.set_condition_at_index(index, 0));
        }
        assert_eq!(population.mean_living_condition_permille(), Some(0));
        assert_eq!(population.summary().mean_living_condition_permille, Some(0));

        for index in 0..population.person_count() {
            assert!(population.mark_death(index, 1));
        }
        assert_eq!(population.living_count(), 0);
        assert_eq!(population.mean_living_condition_permille(), None);
        let summary = population.summary();
        assert_eq!(summary.mean_living_condition_permille, None);
        assert_eq!(
            serde_json::to_value(summary).unwrap()["meanLivingConditionPermille"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn synthetic_modulo_mapping_is_frozen_and_has_at_most_one_count_bucket_imbalance() {
        assert_eq!(synthetic_validation_modulo_draw(0, 3), 0);
        assert_eq!(synthetic_validation_modulo_draw(u64::MAX, 3), u64::MAX % 3);

        let domain = u128::from(u64::MAX) + 1;
        for upper_exclusive in [3_u64, 365, u64::from(PERMILLE_MAX), 10_003] {
            let smaller_bucket = domain / u128::from(upper_exclusive);
            let remainder = domain % u128::from(upper_exclusive);
            assert!(remainder < u128::from(upper_exclusive));
            if remainder != 0 {
                let larger_bucket = smaller_bucket + 1;
                assert_eq!(larger_bucket - smaller_bucket, 1);
            }
        }
    }

    #[test]
    fn persisted_parent_death_on_child_birth_day_is_rejected() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(53)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(53))
                .unwrap();
        let female_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Female)
            })
            .unwrap();
        let male_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Male)
            })
            .unwrap();
        let female = population.person_id_at_index(female_index).unwrap();
        let male = population.person_id_at_index(male_index).unwrap();
        let location = population.location_at_index(female_index).unwrap();
        let household = population.household_at_index(female_index).unwrap();
        population
            .append_birth(
                365,
                ReproductiveSex::Female,
                location,
                household,
                female,
                male,
            )
            .unwrap();
        population.rebuild_occupancy(&world).unwrap();
        assert!(population.mark_death(male_index, 365));

        assert!(matches!(
            population.validate(&world),
            Err(PopulationValidationError::ParentDeadBeforeBirth {
                parent_death_day: 365,
                child_birth_day: 365,
                ..
            })
        ));
    }

    #[test]
    fn persisted_last_birth_not_after_own_birth_is_rejected() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(59)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(59))
                .unwrap();
        let female_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Female)
            })
            .unwrap();
        let male_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Male)
            })
            .unwrap();
        let female = population.person_id_at_index(female_index).unwrap();
        let male = population.person_id_at_index(male_index).unwrap();
        let location = population.location_at_index(female_index).unwrap();
        let household = population.household_at_index(female_index).unwrap();
        population
            .append_birth(
                365,
                ReproductiveSex::Female,
                location,
                household,
                female,
                male,
            )
            .unwrap();
        population.rebuild_occupancy(&world).unwrap();
        let child_index = population.person_count() - 1;
        population.last_birth_days[child_index] = 365;

        assert!(matches!(
            population.validate(&world),
            Err(PopulationValidationError::BirthHistoryBeforeOwnBirth {
                own_birth_day: 365,
                recorded_birth_day: 365,
                ..
            })
        ));
    }

    #[test]
    fn persisted_last_birth_day_must_match_latest_recorded_child() {
        let world = World::generate(WorldConfig::new(1, 1), RngFactory::new(61)).unwrap();
        let mut population =
            Population::initialize(PopulationConfig::new(100), &world, RngFactory::new(61))
                .unwrap();
        let female_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Female)
            })
            .unwrap();
        let male_index = (0..population.person_count())
            .find(|&index| {
                population.reproductive_sex_at_index(index) == Some(ReproductiveSex::Male)
            })
            .unwrap();
        let female = population.person_id_at_index(female_index).unwrap();
        let male = population.person_id_at_index(male_index).unwrap();
        let location = population.location_at_index(female_index).unwrap();
        let household = population.household_at_index(female_index).unwrap();
        population
            .append_birth(
                365,
                ReproductiveSex::Female,
                location,
                household,
                female,
                male,
            )
            .unwrap();
        population.note_successful_birth(female_index, 365);
        population.rebuild_occupancy(&world).unwrap();
        population.validate(&world).unwrap();

        population.last_birth_days[female_index] = 364;
        assert!(matches!(
            population.validate(&world),
            Err(PopulationValidationError::BirthHistoryMismatch {
                recorded_birth_day: Some(364),
                latest_child_birth_day: Some(365),
                ..
            })
        ));

        population.last_birth_days[female_index] = NO_EVENT_DAY;
        assert!(matches!(
            population.validate(&world),
            Err(PopulationValidationError::BirthHistoryMismatch {
                recorded_birth_day: None,
                latest_child_birth_day: Some(365),
                ..
            })
        ));
    }

    #[test]
    fn rejects_zero_household_size() {
        let world = test_world(29);
        let config = PopulationConfig::new(100).with_target_household_size(0);
        assert!(matches!(
            Population::initialize(config, &world, RngFactory::new(29)),
            Err(PopulationError::ZeroHouseholdSize)
        ));
    }

    #[test]
    fn declared_founder_state_does_not_reuse_synthetic_only_household_or_sex_validation() {
        let world = test_world(30);
        let mut config = PopulationConfig::new(3)
            .with_initialization(PopulationInitialization::DeclaredFounderStateV1)
            .with_target_household_size(0);
        config.synthetic_male_permille = 1_001;
        Population::initialize_declared_founder_state_v1(
            config,
            &declared_definition(),
            &world,
            &crate::config::DemographyConfig::synthetic_validation_v1(),
        )
        .unwrap();
    }

    #[test]
    fn rejects_initial_population_above_record_limit() {
        let world = test_world(37);
        let config = PopulationConfig::new(101).with_max_person_records(100);
        assert!(matches!(
            Population::initialize(config, &world, RngFactory::new(37)),
            Err(PopulationError::InitialPopulationExceedsRecordLimit { .. })
        ));
    }
}
