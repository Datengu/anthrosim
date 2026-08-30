use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

use crate::{
    config::{DemographyConfig, ParameterProvenance},
    ids::{CellId, HouseholdId, PersonId},
    population::ReproductiveSex,
    time::DAYS_PER_YEAR,
    world::{PERMILLE_MAX, World},
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const CONTENT_IDENTITY_DOMAIN: &[u8] = b"anthrosim-founder-population-definition-v1";

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
///
/// Programmatically constructed definitions are deliberately editable before persistence. JSON
/// deserialization seals the scientific contents to the serialized `contentDigest64` when one is
/// present, or to the content observed at load time when it is omitted by a standalone input file.
/// Serialization always writes the current deterministic digest. Consequently, persisted run and
/// checkpoint artifacts reject later content mutation unless their integrity metadata is also
/// deliberately rewritten; this is an integrity/reproducibility check, not a cryptographic seal.
#[derive(Debug, Clone)]
pub struct FounderPopulationDefinition {
    pub schema_version: u32,
    pub initialization_id: String,
    pub provenance: ParameterProvenance,
    pub genealogy_status: FounderGenealogyStatus,
    pub households: Vec<FounderHousehold>,
    pub people: Vec<FounderPerson>,
    expected_content_digest64: Option<u64>,
}

impl PartialEq for FounderPopulationDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.initialization_id == other.initialization_id
            && self.provenance == other.provenance
            && self.genealogy_status == other.genealogy_status
            && self.households == other.households
            && self.people == other.people
    }
}

impl Eq for FounderPopulationDefinition {}

impl Serialize for FounderPopulationDefinition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FounderPopulationDefinition", 7)?;
        state.serialize_field("schemaVersion", &self.schema_version)?;
        state.serialize_field("initializationId", &self.initialization_id)?;
        state.serialize_field("provenance", &self.provenance)?;
        state.serialize_field("genealogyStatus", &self.genealogy_status)?;
        state.serialize_field("contentDigest64", &self.content_digest64())?;
        state.serialize_field("households", &self.households)?;
        state.serialize_field("people", &self.people)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for FounderPopulationDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct WireDefinition {
            schema_version: u32,
            initialization_id: String,
            provenance: ParameterProvenance,
            genealogy_status: FounderGenealogyStatus,
            #[serde(default)]
            content_digest64: Option<u64>,
            households: Vec<FounderHousehold>,
            people: Vec<FounderPerson>,
        }

        let wire = WireDefinition::deserialize(deserializer)?;
        let mut definition = Self {
            schema_version: wire.schema_version,
            initialization_id: wire.initialization_id,
            provenance: wire.provenance,
            genealogy_status: wire.genealogy_status,
            households: wire.households,
            people: wire.people,
            expected_content_digest64: wire.content_digest64,
        };
        if definition.expected_content_digest64.is_none() {
            definition.expected_content_digest64 = Some(definition.content_digest64());
        }
        Ok(definition)
    }
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
            expected_content_digest64: None,
        }
    }

    /// Deterministic identity of every scientifically consequential founder-definition field.
    ///
    /// The digest deliberately excludes only the serialized digest itself. It is a compact
    /// reproducibility/integrity identity, not a cryptographic authenticity mechanism.
    #[must_use]
    pub fn content_digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_bytes(&mut hash, CONTENT_IDENTITY_DOMAIN);
        digest_u32(&mut hash, self.schema_version);
        digest_bytes(&mut hash, self.initialization_id.as_bytes());
        digest_u8(&mut hash, provenance_code(self.provenance));
        digest_u8(&mut hash, genealogy_code(self.genealogy_status));
        digest_u64(
            &mut hash,
            u64::try_from(self.households.len()).expect("founder household count must fit u64"),
        );
        for household in &self.households {
            digest_u64(&mut hash, household.id.0);
            digest_u64(&mut hash, household.location.0);
        }
        digest_u64(
            &mut hash,
            u64::try_from(self.people.len()).expect("founder person count must fit u64"),
        );
        for person in &self.people {
            digest_u64(&mut hash, person.id.0);
            digest_i64(&mut hash, person.birth_day);
            digest_u8(&mut hash, reproductive_sex_code(person.reproductive_sex));
            digest_u64(&mut hash, person.household.0);
            digest_optional_person_id(&mut hash, person.female_parent);
            digest_optional_person_id(&mut hash, person.male_parent);
            digest_optional_i64(&mut hash, person.last_birth_day);
            digest_u16(&mut hash, person.condition_permille);
        }
        hash
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
        demography: &DemographyConfig,
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
            let household_index = usize::try_from(person.household.0.checked_sub(1).ok_or(
                FounderPopulationError::InvalidHousehold {
                    person: person.id,
                    household: person.household,
                },
            )?)
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

            validate_parent(
                self,
                person,
                person.female_parent,
                ReproductiveSex::Female,
                demography,
            )?;
            validate_parent(
                self,
                person,
                person.male_parent,
                ReproductiveSex::Male,
                demography,
            )?;
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
                let age_days = reproductive_age_days(person.birth_day, last_birth_day)
                    .expect("validated prior birth must be after founder birth");
                if !female_reproductive_age_supported(demography, age_days) {
                    return Err(
                        FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge {
                            person: person.id,
                            last_birth_day,
                            age_days,
                        },
                    );
                }
            }
        }

        if let Some(index) = household_used.iter().position(|&used| !used) {
            return Err(FounderPopulationError::UnusedHousehold {
                household: HouseholdId::new(index as u64 + 1),
            });
        }

        if let Some(expected) = self.expected_content_digest64 {
            let actual = self.content_digest64();
            if actual != expected {
                return Err(FounderPopulationError::ContentIdentityMismatch { expected, actual });
            }
        }
        Ok(())
    }
}

fn validate_parent(
    definition: &FounderPopulationDefinition,
    child: &FounderPerson,
    parent: Option<PersonId>,
    expected_sex: ReproductiveSex,
    demography: &DemographyConfig,
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
    let age_days = reproductive_age_days(parent_person.birth_day, child.birth_day)
        .expect("validated parent must be older than child");
    let supported = match expected_sex {
        ReproductiveSex::Female => female_reproductive_age_supported(demography, age_days),
        ReproductiveSex::Male => male_reproductive_age_supported(demography, age_days),
    };
    if !supported {
        return Err(
            FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                person: child.id,
                parent,
                parent_sex: expected_sex,
                age_days,
            },
        );
    }
    Ok(())
}

fn reproductive_age_days(birth_day: i64, event_day: i64) -> Option<u64> {
    u64::try_from(event_day.checked_sub(birth_day)?).ok()
}

fn female_reproductive_age_supported(demography: &DemographyConfig, age_days: u64) -> bool {
    let age_years = age_days / DAYS_PER_YEAR;
    demography.fertility_bands.iter().any(|band| {
        age_years >= u64::from(band.start_age_years)
            && age_years < u64::from(band.end_age_years_exclusive)
            && band.annual_probability_per_million > 0
    })
}

fn male_reproductive_age_supported(demography: &DemographyConfig, age_days: u64) -> bool {
    let age_years = age_days / DAYS_PER_YEAR;
    age_years >= u64::from(demography.male_parent_min_age_years)
        && age_years < u64::from(demography.male_parent_max_age_years_exclusive)
}

fn provenance_code(provenance: ParameterProvenance) -> u8 {
    match provenance {
        ParameterProvenance::EmpiricalDirect => 1,
        ParameterProvenance::EmpiricalDerived => 2,
        ParameterProvenance::EvidenceInformed => 3,
        ParameterProvenance::SyntheticValidation => 4,
        ParameterProvenance::Unresolved => 5,
    }
}

fn genealogy_code(status: FounderGenealogyStatus) -> u8 {
    match status {
        FounderGenealogyStatus::Unspecified => 1,
        FounderGenealogyStatus::CompleteLivingDirectParents => 2,
    }
}

fn reproductive_sex_code(sex: ReproductiveSex) -> u8 {
    match sex {
        ReproductiveSex::Female => 1,
        ReproductiveSex::Male => 2,
    }
}

fn digest_optional_person_id(hash: &mut u64, value: Option<PersonId>) {
    match value {
        None => digest_u8(hash, 0),
        Some(id) => {
            digest_u8(hash, 1);
            digest_u64(hash, id.0);
        }
    }
}

fn digest_optional_i64(hash: &mut u64, value: Option<i64>) {
    match value {
        None => digest_u8(hash, 0),
        Some(value) => {
            digest_u8(hash, 1);
            digest_i64(hash, value);
        }
    }
}

fn digest_bytes(hash: &mut u64, bytes: &[u8]) {
    digest_u64(
        hash,
        u64::try_from(bytes.len()).expect("content identity byte length must fit u64"),
    );
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

fn digest_u8(hash: &mut u64, value: u8) {
    digest_bytes_raw(hash, &[value]);
}

fn digest_u16(hash: &mut u64, value: u16) {
    digest_bytes_raw(hash, &value.to_le_bytes());
}

fn digest_u32(hash: &mut u64, value: u32) {
    digest_bytes_raw(hash, &value.to_le_bytes());
}

fn digest_u64(hash: &mut u64, value: u64) {
    digest_bytes_raw(hash, &value.to_le_bytes());
}

fn digest_i64(hash: &mut u64, value: i64) {
    digest_bytes_raw(hash, &value.to_le_bytes());
}

fn digest_bytes_raw(hash: &mut u64, bytes: &[u8]) {
    for &byte in bytes {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(FNV_PRIME);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FounderPopulationError {
    #[error("founder-population schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("founder initialization ID must not be empty")]
    EmptyInitializationId,
    #[error(
        "declared founder count {declared} does not match configured initial population {expected}"
    )]
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
    #[error(
        "founder {person:?} parent {parent:?} ({parent_sex:?}) was age {age_days} days at the child's birth, outside the configured reproductive-age support"
    )]
    ParentOutsideConfiguredReproductiveAge {
        person: PersonId,
        parent: PersonId,
        parent_sex: ReproductiveSex,
        age_days: u64,
    },
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
    #[error(
        "founder {person:?} prior birth day {last_birth_day} occurred at age {age_days} days, outside the configured female fertility-age support"
    )]
    PriorBirthOutsideConfiguredFertilityAge {
        person: PersonId,
        last_birth_day: i64,
        age_days: u64,
    },
    #[error(
        "founder population content identity mismatch: stored {expected}, reconstructed {actual}"
    )]
    ContentIdentityMismatch { expected: u64, actual: u64 },
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
        definition
            .validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1(),
            )
            .unwrap();
        assert_eq!(definition.last_birth_day(PersonId::new(3)), Some(-100));
    }

    #[test]
    fn declared_founder_ids_are_canonical_and_stable() {
        let mut definition = valid_definition();
        definition.people[1].id = PersonId::new(9);
        assert!(matches!(
            definition.validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1()
            ),
            Err(FounderPopulationError::NonCanonicalPersonId { index: 1, .. })
        ));
    }

    #[test]
    fn pre_run_birth_history_must_be_negative_and_after_the_founders_own_birth() {
        let mut definition = valid_definition();
        definition.people[2].last_birth_day = Some(0);
        assert!(matches!(
            definition.validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1()
            ),
            Err(FounderPopulationError::PriorBirthNotBeforeEpoch { .. })
        ));

        definition.people[2].last_birth_day = Some(-10_000);
        assert!(matches!(
            definition.validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1()
            ),
            Err(FounderPopulationError::PriorBirthNotAfterOwnBirth { .. })
        ));
    }

    #[test]
    fn parent_links_require_expected_sex_and_older_founders() {
        let mut definition = valid_definition();
        definition.people[2].female_parent = Some(PersonId::new(2));
        assert!(matches!(
            definition.validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1()
            ),
            Err(FounderPopulationError::ParentSexMismatch { .. })
        ));
    }

    #[test]
    fn parent_reproductive_age_support_follows_declared_demography_boundaries() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;

        definition.people[0].birth_day = child_birth - (18 * year - 1);
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Female,
                    ..
                }
            )
        ));
        definition.people[0].birth_day = child_birth - 18 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].birth_day = child_birth - (45 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].birth_day = child_birth - 45 * year;
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Female,
                    ..
                }
            )
        ));

        definition = valid_definition();
        let child_birth = definition.people[2].birth_day;
        definition.people[1].birth_day = child_birth - (18 * year - 1);
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Male,
                    ..
                }
            )
        ));
        definition.people[1].birth_day = child_birth - 18 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[1].birth_day = child_birth - (70 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[1].birth_day = child_birth - 70 * year;
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Male,
                    ..
                }
            )
        ));
    }

    #[test]
    fn one_day_old_parent_is_rejected() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        definition.people[0].birth_day = definition.people[2].birth_day - 1;
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Female,
                    age_days: 1,
                    ..
                }
            )
        ));
    }

    #[test]
    fn prior_birth_reproductive_age_support_uses_fertility_schedule_boundaries() {
        let demography = DemographyConfig::synthetic_validation_v1();
        let mut definition = valid_definition();
        // Isolate prior-birth history from the separate parent-age rule.
        definition.people[2].female_parent = None;
        definition.people[2].male_parent = None;
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = -80 * year;
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year - 1);
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge { .. })
        ));
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 18 * year);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 45 * year - 1);
        definition.validate(3, 10, &world(), &demography).unwrap();
        definition.people[0].last_birth_day = Some(definition.people[0].birth_day + 45 * year);
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(FounderPopulationError::PriorBirthOutsideConfiguredFertilityAge { .. })
        ));
    }

    #[test]
    fn founder_reproductive_history_tracks_custom_fertility_support() {
        let mut demography = DemographyConfig::synthetic_validation_v1();
        demography.fertility_bands = vec![
            crate::config::AgeProbabilityBand::new(0, 21, 0),
            crate::config::AgeProbabilityBand::new(21, 22, 1),
            crate::config::AgeProbabilityBand::new(22, u32::MAX, 0),
        ];
        let mut definition = valid_definition();
        // Isolate the mother-age rule from the child's unrelated prior-birth history.
        definition.people[2].last_birth_day = None;
        let child_birth = definition.people[2].birth_day;
        let year = DAYS_PER_YEAR as i64;
        definition.people[0].birth_day = child_birth - 20 * year;
        assert!(matches!(
            definition.validate(3, 10, &world(), &demography),
            Err(
                FounderPopulationError::ParentOutsideConfiguredReproductiveAge {
                    parent_sex: ReproductiveSex::Female,
                    ..
                }
            )
        ));
        definition.people[0].birth_day = child_birth - 21 * year;
        definition.validate(3, 10, &world(), &demography).unwrap();
    }

    #[test]
    fn serialized_content_identity_detects_valid_post_load_mutation() {
        let definition = valid_definition();
        let json = serde_json::to_string(&definition).unwrap();
        assert!(json.contains("contentDigest64"));

        let mut loaded: FounderPopulationDefinition = serde_json::from_str(&json).unwrap();
        loaded.people[2].last_birth_day = Some(-200);
        assert!(matches!(
            loaded.validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1()
            ),
            Err(FounderPopulationError::ContentIdentityMismatch { .. })
        ));
    }

    #[test]
    fn standalone_json_without_digest_is_sealed_at_load_time() {
        let definition = valid_definition();
        let mut json_value = serde_json::to_value(&definition).unwrap();
        json_value
            .as_object_mut()
            .unwrap()
            .remove("contentDigest64");
        let loaded: FounderPopulationDefinition = serde_json::from_value(json_value).unwrap();
        loaded
            .validate(
                3,
                10,
                &world(),
                &DemographyConfig::synthetic_validation_v1(),
            )
            .unwrap();

        let round_trip = serde_json::to_value(&loaded).unwrap();
        assert!(round_trip.get("contentDigest64").is_some());
    }

    #[test]
    fn content_identity_covers_genealogy_residence_condition_and_birth_history() {
        let definition = valid_definition();
        let baseline = definition.content_digest64();

        let mut changed = definition.clone();
        changed.people[2].last_birth_day = Some(-200);
        assert_ne!(changed.content_digest64(), baseline);

        let mut changed = definition.clone();
        changed.people[2].male_parent = None;
        assert_ne!(changed.content_digest64(), baseline);

        let mut changed = definition.clone();
        changed.households[0].location = CellId::new(2);
        assert_ne!(changed.content_digest64(), baseline);

        let mut changed = definition;
        changed.people[2].condition_permille = 701;
        assert_ne!(changed.content_digest64(), baseline);
    }
}
