from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parents[1]


def replace(path: str, old: str, new: str, count: int = 1) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"anchor not found in {path}: {old[:120]!r}")
    target.write_text(text.replace(old, new, count), encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")


# ---------------------------------------------------------------------------
# Authoritative household-topology events: required so derived M9 replay and
# household-age observability can reconstruct dynamic household creation.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/events.rs",
    "use crate::{\n    ids::{CellId, HouseholdId, PersonId, TemporaryJourneyId},",
    "use crate::{\n    ids::{CellId, HouseholdId, PersonId, TemporaryJourneyId},",
)
replace(
    "crates/anthrosim-core/src/events.rs",
    "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(tag = \"type\", rename_all = \"camelCase\")]\npub enum EventKind {",
    "pub(crate) const HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION: u32 = 1;\n\n#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n#[serde(tag = \"type\", rename_all = \"camelCase\")]\npub enum EventKind {",
)
replace(
    "crates/anthrosim-core/src/events.rs",
    """        realized_travel_condition_loss_total: u64,\n    },\n    TemporaryJourneyNotStarted {""",
    """        realized_travel_condition_loss_total: u64,\n    },\n    /// Annual-boundary structural household split. This event exists so derived observability can\n    /// replay household creation without inferring it from terminal state. The alternative is a\n    /// synthetic sensitivity treatment, not a claim about historical household formation.\n    HouseholdFission {\n        event_schema_version: u32,\n        source_household: HouseholdId,\n        new_household: HouseholdId,\n        residence: CellId,\n        people_reassigned: Vec<PersonId>,\n    },\n    TemporaryJourneyNotStarted {""",
)

# Population returns the exact authoritative fission records used by the event log.
replace(
    "crates/anthrosim-core/src/population.rs",
    """#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]\npub(crate) struct HouseholdFissionOutcome {\n    pub households_created: u64,\n    pub people_reassigned: u64,\n}""",
    """#[derive(Debug, Clone, PartialEq, Eq)]\npub(crate) struct HouseholdFissionRecord {\n    pub source_household: HouseholdId,\n    pub new_household: HouseholdId,\n    pub residence: CellId,\n    pub people_reassigned: Vec<PersonId>,\n}\n\n#[derive(Debug, Clone, PartialEq, Eq, Default)]\npub(crate) struct HouseholdFissionOutcome {\n    pub households_created: u64,\n    pub people_reassigned: u64,\n    pub fissions: Vec<HouseholdFissionRecord>,\n}""",
)
replace(
    "crates/anthrosim-core/src/population.rs",
    """                let new_household = HouseholdId::new(new_household_raw);\n                self.household_locations.push(residence);\n                for &person_index in &living_members[cursor..cursor + group_size] {\n                    self.households[person_index] = new_household;\n                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);\n                }\n                outcome.households_created = outcome.households_created.saturating_add(1);\n                cursor += group_size;""",
    """                let new_household = HouseholdId::new(new_household_raw);\n                self.household_locations.push(residence);\n                let mut reassigned = Vec::with_capacity(group_size);\n                for &person_index in &living_members[cursor..cursor + group_size] {\n                    self.households[person_index] = new_household;\n                    reassigned.push(person_id_from_index(person_index));\n                    outcome.people_reassigned = outcome.people_reassigned.saturating_add(1);\n                }\n                outcome.households_created = outcome.households_created.saturating_add(1);\n                outcome.fissions.push(HouseholdFissionRecord {\n                    source_household: household,\n                    new_household,\n                    residence,\n                    people_reassigned: reassigned,\n                });\n                cursor += group_size;""",
)

# Lifecycle integration emits events only after dependent topology state reconciles.
replace(
    "crates/anthrosim-core/src/household_lifecycle.rs",
    """    config::{\n        DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID,\n        HouseholdLifecycleConfig,\n    },\n    ids::HouseholdId,""",
    """    config::{\n        DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID,\n        HouseholdLifecycleConfig,\n    },\n    events::{EventKind, EventLog, HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION},\n    ids::HouseholdId,""",
)
replace(
    "crates/anthrosim-core/src/household_lifecycle.rs",
    """pub(crate) fn apply_household_lifecycle_at_annual_boundary(\n    population: &mut Population,\n    temporary_mobility: &mut TemporaryMobilityState,\n    config: &HouseholdLifecycleConfig,""",
    """pub(crate) fn apply_household_lifecycle_at_annual_boundary(\n    population: &mut Population,\n    temporary_mobility: &mut TemporaryMobilityState,\n    events: &mut EventLog,\n    config: &HouseholdLifecycleConfig,""",
)
replace(
    "crates/anthrosim-core/src/household_lifecycle.rs",
    """    let HouseholdFissionOutcome {\n        households_created,\n        people_reassigned,\n    } = population.fission_oversized_households(config.max_living_members, &eligible)?;\n    temporary_mobility.reconcile_household_topology_at_boundary(population, day)?;\n    Ok(HouseholdLifecycleOutcome {\n        households_created,\n        people_reassigned,\n    })""",
    """    let HouseholdFissionOutcome {\n        households_created,\n        people_reassigned,\n        fissions,\n    } = population.fission_oversized_households(config.max_living_members, &eligible)?;\n    temporary_mobility.reconcile_household_topology_at_boundary(population, day)?;\n    for fission in fissions {\n        events.push_authoritative(\n            day,\n            EventKind::HouseholdFission {\n                event_schema_version: HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,\n                source_household: fission.source_household,\n                new_household: fission.new_household,\n                residence: fission.residence,\n                people_reassigned: fission.people_reassigned,\n            },\n        );\n    }\n    Ok(HouseholdLifecycleOutcome {\n        households_created,\n        people_reassigned,\n    })""",
)
for path in [
    "crates/anthrosim-core/src/simulation.rs",
    "crates/anthrosim-core/src/spatial_simulation.rs",
]:
    replace(
        path,
        """                    &mut self.population,\n                    &mut self.temporary_mobility,\n                    &household_lifecycle,""",
        """                    &mut self.population,\n                    &mut self.temporary_mobility,\n                    &mut self.events,\n                    &household_lifecycle,""",
    )

# ---------------------------------------------------------------------------
# Household observability v2: age distributions are reconstructed from the
# authoritative fission history, while the historical fixed baseline remains
# exactly day-old for every founder household.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/src/household_observability.rs",
    r'''
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    use crate::{
        EventKind, EventLog, EventProvenance,
        config::{ExperimentConfig, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID},
        events::HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,
        household_lifecycle::household_lifecycle_model_id,
        ids::{HouseholdId, PersonId},
        population::Population,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdSizeBin {
        pub living_members: u32,
        pub household_count: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdAgeBin {
        pub age_days: u64,
        pub household_count: u64,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdGenerationSpanBin {
        /// Number of genealogical generations represented among living members of one household.
        pub generations: u32,
        pub household_count: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct HouseholdObservabilityReport {
        pub schema_version: u32,
        pub day: u64,
        pub lifecycle_model_id: String,
        pub total_household_records: u64,
        pub active_households: u64,
        pub extinct_households: u64,
        /// Under the historical fixed-founder baseline every household record was created at day
        /// zero, so its age is exactly the run day. Dynamic treatments retain this compatibility
        /// field as null and expose the complete distribution below instead.
        pub uniform_founder_household_age_days: Option<u64>,
        pub oldest_living_household_age_days: u64,
        pub living_household_age_distribution: Vec<HouseholdAgeBin>,
        pub largest_living_household_size: u32,
        pub living_household_size_distribution: Vec<HouseholdSizeBin>,
        pub maximum_living_generation_span: u32,
        pub multigenerational_households: u64,
        pub living_household_generation_span_distribution: Vec<HouseholdGenerationSpanBin>,
    }

    impl HouseholdObservabilityReport {
        pub const CURRENT_SCHEMA_VERSION: u32 = 2;
    }

    pub fn derive_household_observability(
        population: &Population,
        experiment: &ExperimentConfig,
        events: &EventLog,
        day: u64,
    ) -> Result<HouseholdObservabilityReport, HouseholdObservabilityError> {
        let household_count = population.household_count();
        let creation_days = household_creation_days(experiment, events, household_count, day)?;
        let mut living_sizes = vec![0_u32; household_count];
        let mut minimum_generation = vec![u32::MAX; household_count];
        let mut maximum_generation = vec![0_u32; household_count];
        let mut has_living_member = vec![false; household_count];
        let mut memo = vec![None; population.person_count()];
        let mut visiting = vec![false; population.person_count()];

        for person_index in 0..population.person_count() {
            let person = population
                .person(PersonId::new(
                    u64::try_from(person_index)
                        .map_err(|_| HouseholdObservabilityError::PersonIdOverflow)?
                        .checked_add(1)
                        .ok_or(HouseholdObservabilityError::PersonIdOverflow)?,
                ))
                .ok_or(HouseholdObservabilityError::MissingPerson { person_index })?;
            if !person.is_alive() {
                continue;
            }
            let household_index = usize::try_from(person.household.0.checked_sub(1).ok_or(
                HouseholdObservabilityError::InvalidHousehold {
                    household: person.household,
                },
            )?)
            .map_err(|_| HouseholdObservabilityError::InvalidHousehold {
                household: person.household,
            })?;
            if household_index >= household_count {
                return Err(HouseholdObservabilityError::InvalidHousehold {
                    household: person.household,
                });
            }
            let generation = generation_depth(population, person.id, &mut memo, &mut visiting)?;
            living_sizes[household_index] = living_sizes[household_index].saturating_add(1);
            has_living_member[household_index] = true;
            minimum_generation[household_index] = minimum_generation[household_index].min(generation);
            maximum_generation[household_index] = maximum_generation[household_index].max(generation);
        }

        let mut age_bins = BTreeMap::<u64, u64>::new();
        let mut size_bins = BTreeMap::<u32, u64>::new();
        let mut span_bins = BTreeMap::<u32, u64>::new();
        let mut active_households = 0_u64;
        let mut oldest_age = 0_u64;
        let mut largest = 0_u32;
        let mut max_span = 0_u32;
        let mut multigenerational = 0_u64;
        for index in 0..household_count {
            if !has_living_member[index] {
                continue;
            }
            active_households = active_households.saturating_add(1);
            let age = day.checked_sub(creation_days[index]).ok_or(
                HouseholdObservabilityError::CreationDayAfterObservation {
                    household: HouseholdId::new(index as u64 + 1),
                    creation_day: creation_days[index],
                    observation_day: day,
                },
            )?;
            oldest_age = oldest_age.max(age);
            *age_bins.entry(age).or_default() += 1;
            let size = living_sizes[index];
            largest = largest.max(size);
            *size_bins.entry(size).or_default() += 1;
            let span = maximum_generation[index]
                .saturating_sub(minimum_generation[index])
                .saturating_add(1);
            max_span = max_span.max(span);
            if span >= 2 {
                multigenerational = multigenerational.saturating_add(1);
            }
            *span_bins.entry(span).or_default() += 1;
        }

        let total_household_records = u64::try_from(household_count)
            .map_err(|_| HouseholdObservabilityError::HouseholdCountOverflow)?;
        Ok(HouseholdObservabilityReport {
            schema_version: HouseholdObservabilityReport::CURRENT_SCHEMA_VERSION,
            day,
            lifecycle_model_id: household_lifecycle_model_id(experiment.household_lifecycle.as_ref())
                .to_owned(),
            total_household_records,
            active_households,
            extinct_households: total_household_records.saturating_sub(active_households),
            uniform_founder_household_age_days: (experiment.household_lifecycle.is_none()
                && household_lifecycle_model_id(None) == FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID)
                .then_some(day),
            oldest_living_household_age_days: oldest_age,
            living_household_age_distribution: age_bins
                .into_iter()
                .map(|(age_days, household_count)| HouseholdAgeBin {
                    age_days,
                    household_count,
                })
                .collect(),
            largest_living_household_size: largest,
            living_household_size_distribution: size_bins
                .into_iter()
                .map(|(living_members, household_count)| HouseholdSizeBin {
                    living_members,
                    household_count,
                })
                .collect(),
            maximum_living_generation_span: max_span,
            multigenerational_households: multigenerational,
            living_household_generation_span_distribution: span_bins
                .into_iter()
                .map(
                    |(generations, household_count)| HouseholdGenerationSpanBin {
                        generations,
                        household_count,
                    },
                )
                .collect(),
        })
    }

    fn household_creation_days(
        experiment: &ExperimentConfig,
        events: &EventLog,
        household_count: usize,
        observation_day: u64,
    ) -> Result<Vec<u64>, HouseholdObservabilityError> {
        let mut creation_days = vec![0_u64; household_count];
        let mut first_dynamic_household = None::<u64>;
        let mut dynamic_count = 0_u64;
        for record in &events.events {
            let EventKind::HouseholdFission {
                event_schema_version,
                new_household,
                ..
            } = &record.event
            else {
                continue;
            };
            if experiment.household_lifecycle.is_none() {
                return Err(HouseholdObservabilityError::UnexpectedFissionEvent);
            }
            if record.provenance != EventProvenance::Authoritative {
                return Err(HouseholdObservabilityError::NonAuthoritativeFissionEvent);
            }
            if *event_schema_version != HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION {
                return Err(HouseholdObservabilityError::UnsupportedFissionEventSchema {
                    found: *event_schema_version,
                    supported: HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,
                });
            }
            if record.day > observation_day {
                return Err(HouseholdObservabilityError::CreationDayAfterObservation {
                    household: *new_household,
                    creation_day: record.day,
                    observation_day,
                });
            }
            let raw = new_household.0;
            let first = *first_dynamic_household.get_or_insert(raw);
            let expected = first
                .checked_add(dynamic_count)
                .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
            if raw != expected {
                return Err(HouseholdObservabilityError::NonCanonicalFissionHousehold {
                    expected: HouseholdId::new(expected),
                    found: *new_household,
                });
            }
            let index = usize::try_from(
                raw.checked_sub(1)
                    .ok_or(HouseholdObservabilityError::InvalidHousehold {
                        household: *new_household,
                    })?,
            )
            .map_err(|_| HouseholdObservabilityError::InvalidHousehold {
                household: *new_household,
            })?;
            if index >= household_count {
                return Err(HouseholdObservabilityError::InvalidHousehold {
                    household: *new_household,
                });
            }
            creation_days[index] = record.day;
            dynamic_count = dynamic_count.saturating_add(1);
        }
        if let Some(first) = first_dynamic_household {
            let founder_count = first
                .checked_sub(1)
                .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
            let observed_total = founder_count
                .checked_add(dynamic_count)
                .ok_or(HouseholdObservabilityError::HouseholdCountOverflow)?;
            if observed_total
                != u64::try_from(household_count)
                    .map_err(|_| HouseholdObservabilityError::HouseholdCountOverflow)?
            {
                return Err(HouseholdObservabilityError::IncompleteFissionHistory {
                    expected_households: household_count,
                    history_households: observed_total,
                });
            }
        }
        Ok(creation_days)
    }

    fn generation_depth(
        population: &Population,
        person: PersonId,
        memo: &mut [Option<u32>],
        visiting: &mut [bool],
    ) -> Result<u32, HouseholdObservabilityError> {
        let index = usize::try_from(
            person
                .0
                .checked_sub(1)
                .ok_or(HouseholdObservabilityError::InvalidPerson { person })?,
        )
        .map_err(|_| HouseholdObservabilityError::InvalidPerson { person })?;
        if index >= memo.len() {
            return Err(HouseholdObservabilityError::InvalidPerson { person });
        }
        if let Some(value) = memo[index] {
            return Ok(value);
        }
        if visiting[index] {
            return Err(HouseholdObservabilityError::GenealogyCycle { person });
        }
        visiting[index] = true;
        let snapshot = population
            .person(person)
            .ok_or(HouseholdObservabilityError::InvalidPerson { person })?;
        let mut has_parent = false;
        let mut parent_depth = 0_u32;
        for parent in [snapshot.female_parent, snapshot.male_parent] {
            if parent == PersonId::INVALID {
                continue;
            }
            if population.person(parent).is_none() {
                return Err(HouseholdObservabilityError::InvalidParent { person, parent });
            }
            has_parent = true;
            parent_depth = parent_depth.max(generation_depth(population, parent, memo, visiting)?);
        }
        visiting[index] = false;
        let depth = if has_parent {
            parent_depth
                .checked_add(1)
                .ok_or(HouseholdObservabilityError::GenerationDepthOverflow)?
        } else {
            0
        };
        memo[index] = Some(depth);
        Ok(depth)
    }

    #[derive(Debug, Error, PartialEq, Eq)]
    pub enum HouseholdObservabilityError {
        #[error("person index {person_index} has no persistent record")]
        MissingPerson { person_index: usize },
        #[error("person identity space overflowed while deriving household observability")]
        PersonIdOverflow,
        #[error("household count does not fit u64")]
        HouseholdCountOverflow,
        #[error("invalid household identity {household:?}")]
        InvalidHousehold { household: HouseholdId },
        #[error("fixed-founder experiment unexpectedly contains a household-fission event")]
        UnexpectedFissionEvent,
        #[error("household-fission event is not authoritative")]
        NonAuthoritativeFissionEvent,
        #[error("household-fission event schema {found} is unsupported; supported schema is {supported}")]
        UnsupportedFissionEventSchema { found: u32, supported: u32 },
        #[error("non-canonical household-fission identity: expected {expected:?}, found {found:?}")]
        NonCanonicalFissionHousehold {
            expected: HouseholdId,
            found: HouseholdId,
        },
        #[error("household-fission history accounts for {history_households} households but terminal state has {expected_households}")]
        IncompleteFissionHistory {
            expected_households: usize,
            history_households: u64,
        },
        #[error("household {household:?} was created on day {creation_day}, after observation day {observation_day}")]
        CreationDayAfterObservation {
            household: HouseholdId,
            creation_day: u64,
            observation_day: u64,
        },
        #[error("invalid person identity {person:?}")]
        InvalidPerson { person: PersonId },
        #[error("person {person:?} references invalid parent {parent:?}")]
        InvalidParent { person: PersonId, parent: PersonId },
        #[error("genealogy cycle encountered while deriving generation depth at {person:?}")]
        GenealogyCycle { person: PersonId },
        #[error("genealogical generation depth overflowed u32")]
        GenerationDepthOverflow,
    }
    ''',
)

# Public export includes the new age bin.
replace(
    "crates/anthrosim-core/src/lib.rs",
    """pub use household_observability::{\n    HouseholdGenerationSpanBin, HouseholdObservabilityError, HouseholdObservabilityReport,\n    HouseholdSizeBin, derive_household_observability,\n};""",
    """pub use household_observability::{\n    HouseholdAgeBin, HouseholdGenerationSpanBin, HouseholdObservabilityError,\n    HouseholdObservabilityReport, HouseholdSizeBin, derive_household_observability,\n};""",
)

# CLI now gives observability the authoritative event history.
replace(
    "crates/anthrosim-cli/src/bin/anthrosim-household-observability.rs",
    """        &checkpoint.population,\n        &checkpoint.experiment,\n        checkpoint.time.days(),""",
    """        &checkpoint.population,\n        &checkpoint.experiment,\n        &checkpoint.events,\n        checkpoint.time.days(),""",
)

# ---------------------------------------------------------------------------
# M9 observability replay learns household creation from authoritative fission
# events rather than requiring founder and terminal household counts to match.
# ---------------------------------------------------------------------------
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """    EventKind, EventProvenance, HouseholdPresence, MetricProvenance, Population,\n    SimulationCheckpoint, TemporaryJourneyIneligibility, TemporaryMobilityProgram,""",
    """    EventKind, EventProvenance, HouseholdPresence, MetricProvenance, Population,\n    SimulationCheckpoint, TemporaryJourneyIneligibility, TemporaryMobilityProgram,\n    events::HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,""",
)
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """    if initial_population.household_count() != checkpoint.population.household_count() {\n        return Err(invalid(\"initial and checkpoint household counts differ\"));\n    }""",
    """    if initial_population.household_count() > checkpoint.population.household_count() {\n        return Err(invalid(\n            \"terminal household count is smaller than the founder household count\",\n        ));\n    }""",
)
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """        previous_sequence = record.sequence;\n        previous_day = record.day;\n\n        let household = event_household(&record.event);""",
    """        previous_sequence = record.sequence;\n        previous_day = record.day;\n\n        if let EventKind::HouseholdFission {\n            event_schema_version,\n            source_household,\n            new_household,\n            residence,\n            people_reassigned,\n        } = &record.event\n        {\n            replay_household_fission(\n                replay,\n                record.day,\n                *event_schema_version,\n                *source_household,\n                *new_household,\n                *residence,\n                people_reassigned.len(),\n            )?;\n            continue;\n        }\n\n        let household = event_household(&record.event);""",
)
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """#[derive(Debug, Clone, Copy)]\nenum HouseholdPresenceKind {""",
    """fn replay_household_fission(\n    replay: &mut Replay<'_>,\n    day: u64,\n    event_schema_version: u32,\n    source_household: HouseholdId,\n    new_household: HouseholdId,\n    residence: CellId,\n    people_reassigned: usize,\n) -> Result<(), TemporaryMobilityObservabilityError> {\n    if event_schema_version != HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION {\n        return Err(invalid(format!(\n            \"household-fission event schema {event_schema_version} is unsupported; expected {HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION}\"\n        )));\n    }\n    let source_index = household_index(source_household, replay.households.len())?;\n    accrue_household(replay, source_index, day)?;\n    if !replay.households[source_index].presence.is_at_residence()\n        || replay.households[source_index].active_journey.is_some()\n    {\n        return Err(invalid(\n            \"household fission occurred while the source household was temporarily away\",\n        ));\n    }\n    if replay.households[source_index].residence != residence {\n        return Err(invalid(\n            \"household fission residence does not match replay source residence\",\n        ));\n    }\n    let expected_new = u64::try_from(replay.households.len())\n        .map_err(|_| invalid(\"household replay count exceeds u64\"))?\n        .checked_add(1)\n        .ok_or_else(|| invalid(\"household replay identity overflow\"))?;\n    if new_household != HouseholdId::new(expected_new) {\n        return Err(invalid(format!(\n            \"household fission created {:?}, expected next canonical household {:?}\",\n            new_household,\n            HouseholdId::new(expected_new)\n        )));\n    }\n    let moved = u64::try_from(people_reassigned)\n        .map_err(|_| invalid(\"household fission reassignment count exceeds u64\"))?;\n    if moved == 0 || moved >= replay.households[source_index].living {\n        return Err(invalid(\n            \"household fission must move a nonzero proper subset of living source members\",\n        ));\n    }\n    replay.households[source_index].living -= moved;\n    replay.households.push(HouseholdReplay {\n        residence,\n        living: moved,\n        presence: HouseholdPresence::AtResidence,\n        active_journey: None,\n        last_day: day,\n    });\n    Ok(())\n}\n\n#[derive(Debug, Clone, Copy)]\nenum HouseholdPresenceKind {""",
)
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """fn reconcile_terminal_state(\n    replay: &mut Replay<'_>,\n    checkpoint: &SimulationCheckpoint,\n) -> Result<(), TemporaryMobilityObservabilityError> {\n    let mut final_living = vec![0_u64; replay.households.len()];""",
    """fn reconcile_terminal_state(\n    replay: &mut Replay<'_>,\n    checkpoint: &SimulationCheckpoint,\n) -> Result<(), TemporaryMobilityObservabilityError> {\n    if replay.households.len() != checkpoint.population.household_count() {\n        return Err(invalid(format!(\n            \"household replay reconstructed {} households but terminal population has {}\",\n            replay.households.len(),\n            checkpoint.population.household_count()\n        )));\n    }\n    let mut final_living = vec![0_u64; replay.households.len()];""",
)
replace(
    "crates/anthrosim-core/src/temporary_observability.rs",
    """        EventKind::Birth { household, .. }\n        | EventKind::Death { household, .. }\n        | EventKind::HouseholdMigration { household, .. }""",
    """        EventKind::Birth { household, .. }\n        | EventKind::Death { household, .. }\n        | EventKind::HouseholdMigration { household, .. }\n        | EventKind::HouseholdFission {\n            source_household: household,\n            ..\n        }""",
)

# Temporary-history completeness exempts triggers at/before a dynamic household's creation boundary.
replace(
    "crates/anthrosim-core/src/temporary_history.rs",
    """    let report = derive_temporary_mobility_observability(world, &initial_population, checkpoint)\n        .map_err(|error| invalid(format!(\"temporary event replay failed: {error}\")))?;\n\n    let mut journeys""",
    """    let report = derive_temporary_mobility_observability(world, &initial_population, checkpoint)\n        .map_err(|error| invalid(format!(\"temporary event replay failed: {error}\")))?;\n    let mut household_creation_days = BTreeMap::<u64, u64>::new();\n    let mut first_dynamic_household = None::<u64>;\n    for record in &checkpoint.events.events {\n        if let EventKind::HouseholdFission { new_household, .. } = record.event {\n            if household_creation_days.insert(new_household.0, record.day).is_some() {\n                return Err(invalid(format!(\n                    \"duplicate household-fission creation event for household {}\",\n                    new_household.0\n                )));\n            }\n            first_dynamic_household = Some(\n                first_dynamic_household\n                    .map_or(new_household.0, |current| current.min(new_household.0)),\n            );\n        }\n    }\n    let founder_household_count = first_dynamic_household\n        .map(|raw| raw.saturating_sub(1))\n        .unwrap_or_else(|| initial_population.household_count() as u64);\n\n    let mut journeys""",
)
replace(
    "crates/anthrosim-core/src/temporary_history.rs",
    """            EventKind::Birth { .. }\n            | EventKind::Death { .. }\n            | EventKind::HouseholdMigration { .. } => {}""",
    """            EventKind::Birth { .. }\n            | EventKind::Death { .. }\n            | EventKind::HouseholdMigration { .. }\n            | EventKind::HouseholdFission { .. } => {}""",
)
replace(
    "crates/anthrosim-core/src/temporary_history.rs",
    """        for raw in 1..=checkpoint.population.household_count() as u64 {\n            if !trigger_outcomes.contains(&(trigger_index, raw)) {\n                return Err(invalid(format!(\n                    \"missing temporary trigger outcome for trigger {trigger_index}, household {raw}\"\n                )));\n            }\n        }""",
    """        for raw in 1..=checkpoint.population.household_count() as u64 {\n            let existed_for_trigger = if raw <= founder_household_count {\n                true\n            } else {\n                let creation_day = household_creation_days.get(&raw).ok_or_else(|| {\n                    invalid(format!(\n                        \"dynamic household {raw} has no household-fission creation event\"\n                    ))\n                })?;\n                trigger_day > *creation_day\n            };\n            if existed_for_trigger && !trigger_outcomes.contains(&(trigger_index, raw)) {\n                return Err(invalid(format!(\n                    \"missing temporary trigger outcome for trigger {trigger_index}, household {raw}\"\n                )));\n            }\n        }""",
)

# ---------------------------------------------------------------------------
# Stronger issue acceptance tests: distributions, auditable fission events,
# M9 observability replay, and M9 history validation must all work together.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/tests/household_lifecycle_sensitivity.rs",
    r'''
    use anthrosim_core::{
        DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource,
        HouseholdLifecycleConfig, MigrationConfig, Population, PopulationConfig, ResourceConfig,
        Simulation, TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming, World, WorldConfig, derive_household_observability,
        derive_temporary_mobility_observability, ids::{CellId, HouseholdId},
        rng::RngFactory, validate_temporary_mobility_history,
    };

    fn no_event_demography() -> DemographyConfig {
        let mut config = DemographyConfig::synthetic_validation_v1();
        for band in &mut config.mortality_bands {
            band.annual_probability_per_million = 0;
        }
        for band in &mut config.fertility_bands {
            band.annual_probability_per_million = 0;
        }
        config
    }

    fn no_pressure_resources() -> ResourceConfig {
        let mut config = ResourceConfig::synthetic_validation_v1();
        config.annual_need_units_per_person = 0;
        config.max_scarcity_mortality_probability_per_million = 0;
        config
    }

    fn base_config(seed: u64, duration_years: u64) -> ExperimentConfig {
        ExperimentConfig::new(seed, duration_years)
            .with_world(WorldConfig::new(4, 4))
            .with_population(PopulationConfig::new(12).with_target_household_size(12))
            .with_demography(no_event_demography())
            .with_resources(no_pressure_resources())
            .with_migration(MigrationConfig::synthetic_validation_v1().with_enabled(false))
    }

    #[test]
    fn deterministic_size_fission_balances_caps_and_records_household_ages() {
        let config = base_config(20701, 1)
            .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_size_fission_v1(5));
        let run = Simulation::new(config).unwrap().run_recorded().unwrap();
        let report = derive_household_observability(
            &run.checkpoint.population,
            &run.checkpoint.experiment,
            &run.checkpoint.events,
            run.checkpoint.time.days(),
        )
        .unwrap();
        assert_eq!(report.active_households, 3);
        assert_eq!(report.largest_living_household_size, 4);
        assert_eq!(
            report
                .living_household_size_distribution
                .iter()
                .map(|bin| (bin.living_members, bin.household_count))
                .collect::<Vec<_>>(),
            vec![(4, 3)]
        );
        assert_eq!(
            report
                .living_household_age_distribution
                .iter()
                .map(|bin| (bin.age_days, bin.household_count))
                .collect::<Vec<_>>(),
            vec![(0, 2), (365, 1)]
        );
        assert_eq!(run.manifest.population.living_population, 12);
        assert_eq!(run.manifest.population.births_since_start, 0);
        assert_eq!(run.manifest.population.deaths_since_start, 0);
        assert_eq!(
            run.checkpoint
                .events
                .events
                .iter()
                .filter(|record| matches!(record.event, EventKind::HouseholdFission { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn lifecycle_is_exactly_deterministic_and_checkpoint_resumable() {
        let config = base_config(20702, 3)
            .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_size_fission_v1(5));
        let first = Simulation::new(config.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        let duplicate = Simulation::new(config.clone())
            .unwrap()
            .run_recorded()
            .unwrap();
        assert_eq!(
            first.checkpoint.state_digest64,
            duplicate.checkpoint.state_digest64
        );
        assert_eq!(first.checkpoint.population, duplicate.checkpoint.population);
        assert_eq!(first.checkpoint.events, duplicate.checkpoint.events);

        let checkpoint = Simulation::new(config)
            .unwrap()
            .checkpoint_at_year(1)
            .unwrap();
        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();
        assert_eq!(
            first.checkpoint.state_digest64,
            resumed.checkpoint.state_digest64
        );
        assert_eq!(first.checkpoint.population, resumed.checkpoint.population);
        assert_eq!(first.checkpoint.events, resumed.checkpoint.events);
    }

    #[test]
    fn fissioned_households_are_auditable_independent_future_m9_participants() {
        let seed = 20703;
        let base = base_config(seed, 2);
        let factory = RngFactory::new(seed);
        let world = World::generate(base.world, factory).unwrap();
        let initial_population = Population::initialize(base.population, &world, factory).unwrap();
        let residence = initial_population
            .household_location(HouseholdId::new(1))
            .unwrap();
        let destination = (1..=world.cell_count() as u64)
            .map(CellId::new)
            .find(|&cell| cell != residence)
            .unwrap();
        let mobility = TemporaryMobilityConfig::new(
            FocalRegion::new(
                "issue-207-test-region",
                FocalRegionSource::Synthetic,
                vec![destination],
            )
            .unwrap(),
            TemporaryMobilitySchedule::new(
                "issue-207-two-year-schedule",
                TemporaryTriggerTiming::DepartureDay,
                vec![100, 465],
                3,
            )
            .unwrap(),
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .unwrap();

        let baseline = Simulation::new(base.clone().with_temporary_mobility(mobility.clone()))
            .unwrap()
            .run_recorded()
            .unwrap();
        let fission = Simulation::new(
            base.with_temporary_mobility(mobility)
                .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_size_fission_v1(5)),
        )
        .unwrap()
        .run_recorded()
        .unwrap();

        let departures = |events: &anthrosim_core::EventLog| {
            events
                .events
                .iter()
                .filter(|record| matches!(record.event, EventKind::TemporaryJourneyDeparted { .. }))
                .count()
        };
        assert_eq!(departures(&baseline.checkpoint.events), 2);
        assert_eq!(departures(&fission.checkpoint.events), 4);
        assert_eq!(fission.checkpoint.population.household_count(), 3);

        let observability = derive_temporary_mobility_observability(
            &world,
            &initial_population,
            &fission.checkpoint,
        )
        .unwrap();
        assert_eq!(observability.summary.journeys_started, 4);
        assert!(observability.summary.visitor_person_days > 0);
        assert!(observability.summary.peak_visitors > 0);
        validate_temporary_mobility_history(&world, &fission.checkpoint).unwrap();
    }
    ''',
)

# ---------------------------------------------------------------------------
# Paired experiment now preserves pooled terminal distributions and derives
# M9 aggregation from the ordinary authoritative replay path.
# ---------------------------------------------------------------------------
write(
    "crates/anthrosim-core/examples/household_lifecycle_sensitivity.rs",
    r'''
    use std::collections::BTreeMap;

    use anthrosim_core::{
        DemographyConfig, ExperimentConfig, FocalRegion, FocalRegionSource, HouseholdLifecycleConfig,
        MigrationConfig, Population, PopulationConfig, ResourceConfig, Simulation,
        TemporaryMobilityConfig, TemporaryMobilitySchedule, TemporaryTravelModel,
        TemporaryTriggerTiming, World, WorldConfig, derive_household_observability,
        derive_temporary_mobility_observability, ids::CellId, rng::RngFactory,
    };
    use serde::Serialize;

    const DURATION_YEARS: u64 = 40;
    const SEEDS: [u64; 8] = [20701, 20702, 20703, 20704, 20705, 20706, 20707, 20708];

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ArmAggregate {
        lifecycle_model_id: String,
        completed_runs: u64,
        population_extinct_runs: u64,
        terminal_living_population_total: u64,
        terminal_active_households_total: u64,
        terminal_largest_household_size_total: u64,
        terminal_multigenerational_households_total: u64,
        terminal_living_occupied_cells_total: u64,
        terminal_household_size_distribution: BTreeMap<u32, u64>,
        terminal_household_age_days_distribution: BTreeMap<u64, u64>,
        terminal_household_generation_span_distribution: BTreeMap<u32, u64>,
        mean_living_condition_permille_sum: u64,
        mean_living_condition_defined_runs: u64,
        unmet_need_total: u64,
        migration_moves_total: u64,
        migration_people_moved_total: u64,
        temporary_departures_total: u64,
        temporary_visitor_person_days_total: u64,
        temporary_visitor_household_days_total: u64,
        temporary_peak_visitors_max: u64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Comparison {
        schema_version: u32,
        purpose: &'static str,
        scientific_status: &'static str,
        seeds: Vec<u64>,
        duration_years: u64,
        founder_population: u32,
        founder_target_household_size: u16,
        alternative_max_living_members: u16,
        baseline: ArmAggregate,
        deterministic_size_fission: ArmAggregate,
    }

    fn replacement_demography() -> DemographyConfig {
        serde_json::from_str(include_str!(
            "../../../research/demography-controls-v1/replacement-control.json"
        ))
        .unwrap()
    }

    fn config(seed: u64, fission: bool) -> ExperimentConfig {
        let trigger_days = (0..DURATION_YEARS)
            .map(|year| year * 365 + 180)
            .collect::<Vec<_>>();
        let region = FocalRegion::new(
            "issue-207-structural-sensitivity-region",
            FocalRegionSource::Synthetic,
            vec![
                CellId::new(1),
                CellId::new(2),
                CellId::new(3),
                CellId::new(4),
            ],
        )
        .unwrap();
        let temporary_mobility = TemporaryMobilityConfig::new(
            region,
            TemporaryMobilitySchedule::new(
                "issue-207-annual-midyear",
                TemporaryTriggerTiming::DepartureDay,
                trigger_days,
                7,
            )
            .unwrap(),
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .unwrap();
        let resources = ResourceConfig::synthetic_validation_v1();
        let mut config = ExperimentConfig::new(seed, DURATION_YEARS)
            .with_world(WorldConfig::new(12, 12))
            .with_population(PopulationConfig::new(120).with_target_household_size(5))
            .with_demography(replacement_demography())
            .with_resources(resources)
            .with_migration(MigrationConfig::synthetic_validation_v1())
            .with_temporary_mobility(temporary_mobility);
        if fission {
            config = config
                .with_household_lifecycle(HouseholdLifecycleConfig::deterministic_size_fission_v1(8));
        }
        config
    }

    fn add_bin<K: Ord + Copy>(target: &mut BTreeMap<K, u64>, key: K, count: u64) {
        *target.entry(key).or_default() += count;
    }

    fn aggregate(fission: bool) -> ArmAggregate {
        let mut aggregate = ArmAggregate {
            lifecycle_model_id: String::new(),
            completed_runs: 0,
            population_extinct_runs: 0,
            terminal_living_population_total: 0,
            terminal_active_households_total: 0,
            terminal_largest_household_size_total: 0,
            terminal_multigenerational_households_total: 0,
            terminal_living_occupied_cells_total: 0,
            terminal_household_size_distribution: BTreeMap::new(),
            terminal_household_age_days_distribution: BTreeMap::new(),
            terminal_household_generation_span_distribution: BTreeMap::new(),
            mean_living_condition_permille_sum: 0,
            mean_living_condition_defined_runs: 0,
            unmet_need_total: 0,
            migration_moves_total: 0,
            migration_people_moved_total: 0,
            temporary_departures_total: 0,
            temporary_visitor_person_days_total: 0,
            temporary_visitor_household_days_total: 0,
            temporary_peak_visitors_max: 0,
        };
        for seed in SEEDS {
            let run_config = config(seed, fission);
            let factory = RngFactory::new(seed);
            let world = World::generate(run_config.world, factory).unwrap();
            let initial_population =
                Population::initialize(run_config.population, &world, factory).unwrap();
            let run = Simulation::new(run_config).unwrap().run_recorded().unwrap();
            let household = derive_household_observability(
                &run.checkpoint.population,
                &run.checkpoint.experiment,
                &run.checkpoint.events,
                run.checkpoint.time.days(),
            )
            .unwrap();
            let temporary = derive_temporary_mobility_observability(
                &world,
                &initial_population,
                &run.checkpoint,
            )
            .unwrap();
            aggregate.lifecycle_model_id = household.lifecycle_model_id.clone();
            aggregate.completed_runs +=
                u64::from(run.checkpoint.completed_years == run.checkpoint.experiment.duration_years);
            aggregate.population_extinct_runs +=
                u64::from(run.checkpoint.population.living_count() == 0);
            aggregate.terminal_living_population_total += run.checkpoint.population.living_count();
            aggregate.terminal_active_households_total += household.active_households;
            aggregate.terminal_largest_household_size_total +=
                u64::from(household.largest_living_household_size);
            aggregate.terminal_multigenerational_households_total +=
                household.multigenerational_households;
            aggregate.terminal_living_occupied_cells_total += run
                .checkpoint
                .population
                .summary()
                .living_occupied_cell_count;
            for bin in &household.living_household_size_distribution {
                add_bin(
                    &mut aggregate.terminal_household_size_distribution,
                    bin.living_members,
                    bin.household_count,
                );
            }
            for bin in &household.living_household_age_distribution {
                add_bin(
                    &mut aggregate.terminal_household_age_days_distribution,
                    bin.age_days,
                    bin.household_count,
                );
            }
            for bin in &household.living_household_generation_span_distribution {
                add_bin(
                    &mut aggregate.terminal_household_generation_span_distribution,
                    bin.generations,
                    bin.household_count,
                );
            }
            if let Some(condition) = run.checkpoint.population.mean_living_condition_permille() {
                aggregate.mean_living_condition_permille_sum += u64::from(condition);
                aggregate.mean_living_condition_defined_runs += 1;
            }
            aggregate.unmet_need_total += run.manifest.resources.unmet_need;
            aggregate.migration_moves_total += run.manifest.migration.moves_completed;
            aggregate.migration_people_moved_total += run.manifest.migration.people_moved;
            aggregate.temporary_departures_total += temporary.summary.journeys_started;
            aggregate.temporary_visitor_person_days_total += temporary.summary.visitor_person_days;
            aggregate.temporary_visitor_household_days_total +=
                temporary.summary.visitor_household_days;
            aggregate.temporary_peak_visitors_max = aggregate
                .temporary_peak_visitors_max
                .max(temporary.summary.peak_visitors);
        }
        aggregate
    }

    fn main() {
        let comparison = Comparison {
            schema_version: 2,
            purpose: "TRACE structural sensitivity to founder-defined versus deterministic size-fission household lifecycles",
            scientific_status: "synthetic structural sensitivity; not empirical household validation",
            seeds: SEEDS.to_vec(),
            duration_years: DURATION_YEARS,
            founder_population: 120,
            founder_target_household_size: 5,
            alternative_max_living_members: 8,
            baseline: aggregate(false),
            deterministic_size_fission: aggregate(true),
        };
        println!("{}", serde_json::to_string_pretty(&comparison).unwrap());
    }
    ''',
)

write(
    "scripts/render-household-lifecycle-result.py",
    r'''
    import json
    import sys
    from pathlib import Path

    source = Path(sys.argv[1])
    target = Path(sys.argv[2])
    data = json.loads(source.read_text(encoding="utf-8"))
    b = data["baseline"]
    f = data["deterministicSizeFission"]
    n = len(data["seeds"])

    def mean(total):
        return total / n

    def distribution(values, suffix=""):
        return ", ".join(
            f"{key}{suffix}: {value}"
            for key, value in sorted(values.items(), key=lambda item: int(item[0]))
        )

    lines = [
        "# Household lifecycle structural sensitivity — first result",
        "",
        "**Scientific status:** synthetic structural sensitivity; not empirical household validation.",
        "",
        f"Eight paired seeds were run for {data['durationYears']} years. The arms differ only in household lifecycle: `fixed_founder_v1` versus `deterministic_size_fission_v1` with a maximum of {data['alternativeMaxLivingMembers']} living members per eligible household after an annual boundary.",
        "",
        "| Observable | Fixed founder | Size fission |",
        "| --- | ---: | ---: |",
        f"| Completed runs | {b['completedRuns']}/{n} | {f['completedRuns']}/{n} |",
        f"| Extinct runs | {b['populationExtinctRuns']}/{n} | {f['populationExtinctRuns']}/{n} |",
        f"| Mean terminal living population | {mean(b['terminalLivingPopulationTotal']):.2f} | {mean(f['terminalLivingPopulationTotal']):.2f} |",
        f"| Mean terminal active households | {mean(b['terminalActiveHouseholdsTotal']):.2f} | {mean(f['terminalActiveHouseholdsTotal']):.2f} |",
        f"| Mean terminal largest household | {mean(b['terminalLargestHouseholdSizeTotal']):.2f} | {mean(f['terminalLargestHouseholdSizeTotal']):.2f} |",
        f"| Mean terminal multi-generational households | {mean(b['terminalMultigenerationalHouseholdsTotal']):.2f} | {mean(f['terminalMultigenerationalHouseholdsTotal']):.2f} |",
        f"| Mean terminal occupied residence cells | {mean(b['terminalLivingOccupiedCellsTotal']):.2f} | {mean(f['terminalLivingOccupiedCellsTotal']):.2f} |",
        f"| Total unmet resource need | {b['unmetNeedTotal']} | {f['unmetNeedTotal']} |",
        f"| Total M4 moves | {b['migrationMovesTotal']} | {f['migrationMovesTotal']} |",
        f"| Mean people per M4 move | {(b['migrationPeopleMovedTotal'] / b['migrationMovesTotal']) if b['migrationMovesTotal'] else 0:.3f} | {(f['migrationPeopleMovedTotal'] / f['migrationMovesTotal']) if f['migrationMovesTotal'] else 0:.3f} |",
        f"| Total M9 departures | {b['temporaryDeparturesTotal']} | {f['temporaryDeparturesTotal']} |",
        f"| Total M9 visitor person-days | {b['temporaryVisitorPersonDaysTotal']} | {f['temporaryVisitorPersonDaysTotal']} |",
        f"| Total M9 visitor household-days | {b['temporaryVisitorHouseholdDaysTotal']} | {f['temporaryVisitorHouseholdDaysTotal']} |",
        f"| Maximum peak simultaneous visitors | {b['temporaryPeakVisitorsMax']} | {f['temporaryPeakVisitorsMax']} |",
        "",
        "## Pooled terminal household distributions",
        "",
        "Counts below pool active terminal households across all eight paired seeds.",
        "",
        f"- **Living members per household — fixed:** {distribution(b['terminalHouseholdSizeDistribution'])}",
        f"- **Living members per household — fission:** {distribution(f['terminalHouseholdSizeDistribution'])}",
        f"- **Household age (days) — fixed:** {distribution(b['terminalHouseholdAgeDaysDistribution'], 'd')}",
        f"- **Household age (days) — fission:** {distribution(f['terminalHouseholdAgeDaysDistribution'], 'd')}",
        f"- **Living genealogical generations — fixed:** {distribution(b['terminalHouseholdGenerationSpanDistribution'])}",
        f"- **Living genealogical generations — fission:** {distribution(f['terminalHouseholdGenerationSpanDistribution'])}",
        "",
        "## Interpretation",
        "",
    ]
    materially_different = (
        b["terminalActiveHouseholdsTotal"] != f["terminalActiveHouseholdsTotal"]
        or b["migrationMovesTotal"] != f["migrationMovesTotal"]
        or b["temporaryVisitorPersonDaysTotal"] != f["temporaryVisitorPersonDaysTotal"]
        or b["unmetNeedTotal"] != f["unmetNeedTotal"]
    )
    if materially_different:
        lines.append(
            "The declared lifecycle contrast is **material for at least one predeclared household/resource/mobility observable** in this synthetic ensemble. Household lifecycle must therefore remain an explicit structural uncertainty dimension for claims that depend on household sharing, M4 permanent migration, or M9 participation/aggregation. This does not establish which lifecycle is historically correct."
        )
    else:
        lines.append(
            "The declared lifecycle contrast did not alter the predeclared aggregate observables in this synthetic ensemble. That is robustness evidence only for this exact contrast and does not validate either lifecycle historically."
        )
    lines.extend([
        "",
        "The fixed-founder arm's active household ages are exactly the 40-year run duration by construction. The size-fission arm instead contains multiple household ages because annual creation boundaries are now preserved authoritatively and replayable. M9 visitor person-days, household-days and peak visitors are derived through the ordinary temporary-mobility observability replay rather than counted by a special analysis path.",
        "",
        "The machine-readable aggregate used for this page is `research/household-lifecycle-sensitivity-v1/reference-result.json`.",
    ])
    target.write_text("\n".join(lines) + "\n", encoding="utf-8")
    ''',
)

# Documentation promises the stronger age and M9 observability contract.
replace(
    "docs/research/household-lifecycle-structural-sensitivity-v1.md",
    """- total and active household records;\n- the living household-size distribution and maximum;\n- living genealogical-generation-span distribution and multi-generational household count;\n- exact uniform household age for the fixed-founder baseline.""",
    """- total and active household records;\n- the living household-size distribution and maximum;\n- active-household age distributions reconstructed from authoritative fission creation events;\n- living genealogical-generation-span distribution and multi-generational household count;\n- exact uniform household age for the fixed-founder baseline.""",
)
replace(
    "docs/research/household-lifecycle-structural-sensitivity-v1.md",
    """Existing authoritative/derived reports continue to provide the other #207 comparison targets:\nM3 unmet need and condition, M4 move frequency and people moved, M9 journey/aggregation events,\nterminal population and spatial occupancy. No explorer-only state becomes authoritative.""",
    """Existing authoritative/derived reports continue to provide the other #207 comparison targets:\nM3 unmet need and condition, M4 move frequency and people moved, M9 journey participation plus\nvisitor person-days/household-days/peak aggregation, terminal population and spatial occupancy.\nHousehold-fission events are authoritative solely so ordinary derived M9 replay can reconstruct\ndynamic household topology; no explorer-only state becomes authoritative.""",
)
replace(
    "docs/research/household-lifecycle-structural-sensitivity-v1.md",
    """with the same founder population, replacement-control demography, M3/M4 assumptions, annual M9\nschedule and synthetic world dimensions in both arms. The only structural treatment is the\nhousehold lifecycle.""",
    """with the same founder population, replacement-control demography, M3/M4 assumptions, annual M9\nschedule and synthetic world dimensions in both arms. The only structural treatment is the\nhousehold lifecycle. The preserved result pools terminal household size/age/generation-span\ndistributions and derives M9 aggregation through the standard observability replay.""",
)
replace(
    "docs/scientific-model.md",
    """7. after the year's subannual schedules complete, run the annual M2 fertility/parentage stage for survivors.\n\nUnder the v8 annual resource-accounting contract""",
    """7. after the year's subannual schedules complete, run the annual M2 fertility/parentage stage for survivors;\n8. if an explicit household-lifecycle treatment is enabled, apply its annual topology transition after fertility and reconcile dependent M4/M9 state before the annual metric snapshot.\n\nUnder the v8 annual resource-accounting contract""",
)

print("issue 207 observability integration patch applied")
