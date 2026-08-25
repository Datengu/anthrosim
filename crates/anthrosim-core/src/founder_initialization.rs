use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::ParameterProvenance,
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
    world::{PERMILLE_MAX, World},
};

/// Completeness statement for direct-parent state at the simulation boundary.
///
/// `Unspecified` means missing founder parent links are not scientifically interpretable as an
/// absence of living direct kin. `CompleteLivingDirectParents` means each omitted direct-parent
/// link explicitly states that no living direct parent represented in this founder population is
/// known/declared for that role. It does not claim complete genealogy beyond living direct parents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FounderGenealogyStatus {
    Unspecified,
    CompleteLivingDirectParents,
}

/// One persistent household declared at simulation day 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FounderHousehold {
    pub id: HouseholdId,
    pub location: CellId,
}

/// One living person declared at simulation day 0.
///
/// Birth and prior-birth timing use signed epoch-relative days. Negative values are before the
/// simulation epoch. `last_birth_day` is reproductive-history timing only: it does not imply that
/// the corresponding child is represented by a persistent person record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FounderPerson {
    pub id: PersonId,
    pub birth_day: i64,
    pub reproductive_sex: ReproductiveSex,
    pub household: HouseholdId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub female_parent: Option<PersonId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub male_parent: Option<PersonId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_birth_day: Option<i64>,
    pub condition_permille: u16,
}

/// Explicit, provenance-bearing founder state used instead of the synthetic validation preset.
///
/// The complete serialized definition is part of `ExperimentConfig` and therefore part of the
/// immutable experiment identity. `provenance` describes the declared epistemic source of the
/// founder state; evidence-closure enforcement remains the separate research-readiness gate in
/// issue #181.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FounderPopulationDefinition {
    pub schema_version: u32,
    pub initialization_id: String,
    pub provenance: ParameterProvenance,
    pub genealogy_status: FounderGenealogyStatus,
    pub households: Vec<FounderHousehold>,
    pub people: Vec<FounderPerson>,
}

impl FounderPopulationDefinition {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn new(
        initialization_id: impl Into<String>,
        provenance: ParameterProvenance,
        genealogy_status: FounderGenealogyStatus,
        households: Vec<FounderHousehold>,
        people: Vec<FounderPerson>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            initialization_id: initialization_id.into(),
            provenance,
            genealogy_status,
            households,
            people,
        }
    }

    /// Return one declared founder by stable one-based `PersonId` when IDs are canonical.
    #[must_use]
    pub fn person(&self, id: PersonId) -> Option<&FounderPerson> {
        let index = usize::try_from(id.0.checked_sub(1)?).ok()?;
        self.people.get(index).filter(|person| person.id == id)
    }

    /// Signed pre-run last-birth timing for the declared founder, if supplied.
    #[must_use]
    pub fn last_birth_day(&self, id: PersonId) -> Option<i64> {
        self.person(id)?.last_birth_day
    }

    pub fn validate(
        &self,
        expected_initial_population: u32,
        max_person_records: u64,
        world: &World,
    ) -> Result<(), FounderPopulationError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(FounderPopulationError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.initialization_id.trim().is_empty() {
            return Err(FounderPopulationError::EmptyInitializationId);
        }
        let expected_people = usize::try_from(expected_initial_population)
            .expect("u32 initial population must fit supported usize targets");
        if self.people.len() != expected_people {
            return Err(FounderPopulationError::PopulationCountMismatch {
                declared: self.people.len(),
                expected: expected_people,
            });
        }
        if self.people.len() as u64 > max_person_records {
            return Err(FounderPopulationError::PopulationExceedsRecordLimit {
                declared: self.people.len() as u64,
                limit: max_person_records,
            });
        }

        for (index, person) in self.people.iter().enumerate() {
            let expected = PersonId::new(index as u64 + 1);
            if person.id != expected {
                return Err(FounderPopulationError::NonCanonicalPersonId {
                    index,
                    expected,
                    found: person.id,
                });
            }
        }
        for (index, household) in self.households.iter().enumerate() {
            let expected = HouseholdId::new(index as u64 + 1);
            if household.id != expected {
                return Err(FounderPopulationError::NonCanonicalHouseholdId {
                    index,
                    expected,
                    found: household.id,
                });
            }
            if world.cell(household.location).is_none() {
                return Err(FounderPopulationError::HouseholdOutsideWorld {
                    household: household.id,
                    location: household.location,
                });
            }
        }

        let mut household_used = vec![false; self.households.len()];
        for person in &self.people {
            if person.birth_day > 0 {
                return Err(FounderPopulationError::FounderBornAfterEpoch {
                    person: person.id,
                    birth_day: person.birth_day,
                });
            }
            if person.condition_permille > PERMILLE_MAX {
                return Err(FounderPopulationError::InvalidCondition {
                    person: person.id,
                    condition: person.condition_permille,
                });
            }
            let household_index = usize::try_from(
                person
                    .household
                    .0
                    .checked_sub(1)
                    .ok_or(FounderPopulationError::InvalidHousehold {
                        person: person.id,
                        household: person.household,
                    })?,
            )
            .map_err(|_| FounderPopulationError::InvalidHousehold {
                person: person.id,
                household: person.household,
            })?;
            if household_index >= self.households.len() {
                return Err(FounderPopulationError::InvalidHousehold {
                    person: person.id,
                    household: person.household,
                });
            }
            household_used[household_index] = true;

            validate_parent(self, person, person.female_parent, ReproductiveSex::Female)?;
            validate_parent(self, person, person.male_parent, ReproductiveSex::Male)?;
            if let (Some(female), Some(male)) = (person.female_parent, person.male_parent)
                && female == male
            {
                return Err(FounderPopulationError::DuplicateParents { person: person.id });
            }

            if let Some(last_birth_day) = person.last_birth_day {
                if person.reproductive_sex != ReproductiveSex::Female {
                    return Err(FounderPopulationError::BirthHistoryOnNonFemale {
                        person: person.id,
                    });
                }
                if last_birth_day >= 0 {
                    return Err(FounderPopulationError::PriorBirthNotBeforeEpoch {
                        person: person.id,
                        last_birth_day,
                    });
                }
                if last_birth_day <= person.birth_day {
                    return Err(FounderPopulationError::PriorBirthNotAfterOwnBirth {
                        person: person.id,
                        birth_day: person.birth_day,
                        last_birth_day,
                    });
                }
            }
        }

        if let Some(index) = household_used.iter().position(|&used| !used) {
            return Err(FounderPopulationError::UnusedHousehold {
                household: HouseholdId::new(index as u64 + 1),
            });
        }
        Ok(())
    }
}

fn validate_parent(
    definition: &FounderPopulationDefinition,
    child: &FounderPerson,
    parent: Option<PersonId>,
    expected_sex: ReproductiveSex,
) -> Result<(), FounderPopulationError> {
    let Some(parent) = parent else {
        return Ok(());
    };
    if parent == child.id {
        return Err(FounderPopulationError::SelfParent { person: child.id });
    }
    let parent_person = definition
        .person(parent)
        .ok_or(FounderPopulationError::InvalidParent {
            person: child.id,
            parent,
        })?;
    if parent_person.reproductive_sex != expected_sex {
        return Err(FounderPopulationError::ParentSexMismatch {
            person: child.id,
            parent,
            expected: expected_sex,
            actual: parent_person.reproductive_sex,
        });
    }
    if parent_person.birth_day >= child.birth_day {
        return Err(FounderPopulationError::ParentNotOlder {
            person: child.id,
            parent,
        });
    }
    Ok(())
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FounderPopulationError {
    #[error("founder-population schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("founder initialization ID must not be empty")]
    EmptyInitializationId,
    #[error("declared founder count {declared} does not match configured initial population {expected}")]
    PopulationCountMismatch { declared: usize, expected: usize },
    #[error("declared founder count {declared} exceeds persistent record limit {limit}")]
    PopulationExceedsRecordLimit { declared: u64, limit: u64 },
    #[error("founder person index {index} must use ID {expected:?}, found {found:?}")]
    NonCanonicalPersonId {
        index: usize,
        expected: PersonId,
        found: PersonId,
    },
    #[error("founder household index {index} must use ID {expected:?}, found {found:?}")]
    NonCanonicalHouseholdId {
        index: usize,
        expected: HouseholdId,
        found: HouseholdId,
    },
    #[error("founder household {household:?} has location {location:?} outside the world")]
    HouseholdOutsideWorld {
        household: HouseholdId,
        location: CellId,
    },
    #[error("founder {person:?} is assigned to invalid household {household:?}")]
    InvalidHousehold {
        person: PersonId,
        household: HouseholdId,
    },
    #[error("founder household {household:?} is declared but has no founder members")]
    UnusedHousehold { household: HouseholdId },
    #[error("founder {person:?} has post-epoch birth day {birth_day}")]
    FounderBornAfterEpoch { person: PersonId, birth_day: i64 },
    #[error("founder {person:?} has invalid condition {condition} permille")]
    InvalidCondition { person: PersonId, condition: u16 },
    #[error("founder {person:?} is their own parent")]
    SelfParent { person: PersonId },
    #[error("founder {person:?} has the same non-null parent in both parent roles")]
    DuplicateParents { person: PersonId },
    #[error("founder {person:?} references invalid parent {parent:?}")]
    InvalidParent { person: PersonId, parent: PersonId },
    #[error(
        "founder {person:?} parent {parent:?} has incompatible reproductive sex: expected {expected:?}, found {actual:?}"
    )]
    ParentSexMismatch {
        person: PersonId,
        parent: PersonId,
        expected: ReproductiveSex,
        actual: ReproductiveSex,
    },
    #[error("founder {person:?} parent {parent:?} is not older than the child")]
    ParentNotOlder { person: PersonId, parent: PersonId },
    #[error("founder {person:?} has birth-history timing despite non-female reproductive sex")]
    BirthHistoryOnNonFemale { person: PersonId },
    #[error("founder {person:?} prior birth day {last_birth_day} is not before epoch day 0")]
    PriorBirthNotBeforeEpoch {
        person: PersonId,
        last_birth_day: i64,
    },
    #[error(
        "founder {person:?} prior birth day {last_birth_day} is not after own birth day {birth_day}"
    )]
    PriorBirthNotAfterOwnBirth {
        person: PersonId,
        birth_day: i64,
        last_birth_day: i64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::WorldConfig, rng::RngFactory, world::World};

    fn world() -> World {
        World::generate(WorldConfig::new(2, 1), RngFactory::new(1)).unwrap()
    }

    fn valid_definition() -> FounderPopulationDefinition {
        FounderPopulationDefinition::new(
            "declared-test-v1",
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
    fn valid_declared_founder_state_accepts_pre_run_history_and_living_parents() {
        let definition = valid_definition();
        definition.validate(3, 10, &world()).unwrap();
        assert_eq!(definition.last_birth_day(PersonId::new(3)), Some(-100));
    }

    #[test]
    fn declared_founder_ids_are_canonical_and_stable() {
        let mut definition = valid_definition();
        definition.people[1].id = PersonId::new(9);
        assert!(matches!(
            definition.validate(3, 10, &world()),
            Err(FounderPopulationError::NonCanonicalPersonId { index: 1, .. })
        ));
    }

    #[test]
    fn pre_run_birth_history_must_be_negative_and_after_the_founders_own_birth() {
        let mut definition = valid_definition();
        definition.people[2].last_birth_day = Some(0);
        assert!(matches!(
            definition.validate(3, 10, &world()),
            Err(FounderPopulationError::PriorBirthNotBeforeEpoch { .. })
        ));

        definition.people[2].last_birth_day = Some(-10_000);
        assert!(matches!(
            definition.validate(3, 10, &world()),
            Err(FounderPopulationError::PriorBirthNotAfterOwnBirth { .. })
        ));
    }

    #[test]
    fn parent_links_require_expected_sex_and_older_founders() {
        let mut definition = valid_definition();
        definition.people[2].female_parent = Some(PersonId::new(2));
        assert!(matches!(
            definition.validate(3, 10, &world()),
            Err(FounderPopulationError::ParentSexMismatch { .. })
        ));
    }
}
