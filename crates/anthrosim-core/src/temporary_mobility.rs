use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    population::Population,
    world::World,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Authoritative M9 physical-presence state for one household.
///
/// Persistent residence remains in `Population::household_location`. Transit deliberately has no
/// occupied world cell in M9 v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum HouseholdPresence {
    AtResidence,
    OutboundTransit {
        journey: TemporaryJourneyId,
        destination: CellId,
    },
    Visiting {
        journey: TemporaryJourneyId,
        destination: CellId,
    },
    ReturnTransit {
        journey: TemporaryJourneyId,
        destination: CellId,
    },
}

impl HouseholdPresence {
    #[must_use]
    pub const fn is_at_residence(self) -> bool {
        matches!(self, Self::AtResidence)
    }

    #[must_use]
    pub const fn active_journey(self) -> Option<TemporaryJourneyId> {
        match self {
            Self::AtResidence => None,
            Self::OutboundTransit { journey, .. }
            | Self::Visiting { journey, .. }
            | Self::ReturnTransit { journey, .. } => Some(journey),
        }
    }

    #[must_use]
    pub const fn destination(self) -> Option<CellId> {
        match self {
            Self::AtResidence => None,
            Self::OutboundTransit { destination, .. }
            | Self::Visiting { destination, .. }
            | Self::ReturnTransit { destination, .. } => Some(destination),
        }
    }
}

/// Compact authoritative presence layer parallel to persistent household residence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityState {
    pub schema_version: u32,
    household_presence: Vec<HouseholdPresence>,
}

impl TemporaryMobilityState {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn at_residence(population: &Population) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            household_presence: vec![HouseholdPresence::AtResidence; population.household_count()],
        }
    }

    #[must_use]
    pub fn household_count(&self) -> usize {
        self.household_presence.len()
    }

    #[must_use]
    pub fn presence(&self, household: HouseholdId) -> Option<HouseholdPresence> {
        self.household_presence
            .get(household_index(household, self.household_count())?)
            .copied()
    }

    #[must_use]
    pub fn is_at_residence(&self, household: HouseholdId) -> Option<bool> {
        self.presence(household)
            .map(HouseholdPresence::is_at_residence)
    }

    #[must_use]
    pub fn current_cell(&self, household: HouseholdId, population: &Population) -> Option<CellId> {
        match self.presence(household)? {
            HouseholdPresence::AtResidence => population.household_location(household),
            HouseholdPresence::Visiting { destination, .. } => Some(destination),
            HouseholdPresence::OutboundTransit { .. } | HouseholdPresence::ReturnTransit { .. } => {
                None
            }
        }
    }

    #[must_use]
    pub fn all_at_residence(&self) -> bool {
        self.household_presence
            .iter()
            .all(|presence| presence.is_at_residence())
    }

    /// Remove active temporary state for households with no living members.
    ///
    /// M9 presence is household-coordinated. If the final living member dies, there is no longer a
    /// physical traveller to represent, so the active journey state is retired deterministically.
    pub(crate) fn reconcile_after_population_change(&mut self, population: &Population) {
        for (index, presence) in self.household_presence.iter_mut().enumerate() {
            if presence.is_at_residence() {
                continue;
            }
            let household = HouseholdId::new(index as u64 + 1);
            if !household_has_living_member(population, household) {
                *presence = HouseholdPresence::AtResidence;
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn set_presence(
        &mut self,
        household: HouseholdId,
        presence: HouseholdPresence,
        population: &Population,
        world: &World,
    ) -> Result<(), TemporaryMobilityError> {
        let index = household_index(household, self.household_count())
            .ok_or(TemporaryMobilityError::InvalidHousehold { household })?;
        validate_presence(household, presence, population, world)?;
        self.household_presence[index] = presence;
        Ok(())
    }

    pub fn validate(
        &self,
        population: &Population,
        world: &World,
    ) -> Result<(), TemporaryMobilityValidationError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryMobilityValidationError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.household_count() != population.household_count() {
            return Err(TemporaryMobilityValidationError::HouseholdCountMismatch {
                state: self.household_count(),
                population: population.household_count(),
            });
        }

        let mut active_journeys = BTreeSet::new();
        for (index, &presence) in self.household_presence.iter().enumerate() {
            let household = HouseholdId::new(index as u64 + 1);
            validate_presence(household, presence, population, world).map_err(|error| {
                TemporaryMobilityValidationError::InvalidPresence {
                    household,
                    reason: error.to_string(),
                }
            })?;
            if let Some(journey) = presence.active_journey()
                && !active_journeys.insert(journey)
            {
                return Err(TemporaryMobilityValidationError::DuplicateActiveJourney { journey });
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_u64(&mut hash, self.household_count() as u64);
        for presence in &self.household_presence {
            match *presence {
                HouseholdPresence::AtResidence => digest_u64(&mut hash, 0),
                HouseholdPresence::OutboundTransit {
                    journey,
                    destination,
                } => {
                    digest_u64(&mut hash, 1);
                    digest_u64(&mut hash, journey.0);
                    digest_u64(&mut hash, destination.0);
                }
                HouseholdPresence::Visiting {
                    journey,
                    destination,
                } => {
                    digest_u64(&mut hash, 2);
                    digest_u64(&mut hash, journey.0);
                    digest_u64(&mut hash, destination.0);
                }
                HouseholdPresence::ReturnTransit {
                    journey,
                    destination,
                } => {
                    digest_u64(&mut hash, 3);
                    digest_u64(&mut hash, journey.0);
                    digest_u64(&mut hash, destination.0);
                }
            }
        }
        hash
    }
}

fn validate_presence(
    household: HouseholdId,
    presence: HouseholdPresence,
    population: &Population,
    world: &World,
) -> Result<(), TemporaryMobilityError> {
    let residence = population
        .household_location(household)
        .ok_or(TemporaryMobilityError::InvalidHousehold { household })?;
    if world.cell(residence).is_none() {
        return Err(TemporaryMobilityError::InvalidResidence {
            household,
            residence,
        });
    }

    let Some(journey) = presence.active_journey() else {
        return Ok(());
    };
    if !household_has_living_member(population, household) {
        return Err(TemporaryMobilityError::NoLivingMembers { household });
    }
    if journey == TemporaryJourneyId::INVALID {
        return Err(TemporaryMobilityError::InvalidJourney { household });
    }
    let destination = presence
        .destination()
        .ok_or(TemporaryMobilityError::MissingDestination { household })?;
    if world.cell(destination).is_none() {
        return Err(TemporaryMobilityError::InvalidDestination {
            household,
            destination,
        });
    }
    if destination == residence {
        return Err(TemporaryMobilityError::DestinationIsResidence {
            household,
            residence,
        });
    }
    Ok(())
}

fn household_has_living_member(population: &Population, household: HouseholdId) -> bool {
    (0..population.person_count()).any(|index| {
        population.is_alive_index(index)
            && population.household_at_index(index) == Some(household)
    })
}

fn household_index(household: HouseholdId, household_count: usize) -> Option<usize> {
    let index = usize::try_from(household.0.checked_sub(1)?).ok()?;
    (index < household_count).then_some(index)
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityError {
    #[error("temporary mobility references invalid household {household:?}")]
    InvalidHousehold { household: HouseholdId },
    #[error("household {household:?} has invalid residence {residence:?}")]
    InvalidResidence {
        household: HouseholdId,
        residence: CellId,
    },
    #[error("household {household:?} has no living members for an active temporary journey")]
    NoLivingMembers { household: HouseholdId },
    #[error("household {household:?} has an active temporary state with invalid journey ID")]
    InvalidJourney { household: HouseholdId },
    #[error("household {household:?} has an active temporary state without a destination")]
    MissingDestination { household: HouseholdId },
    #[error("household {household:?} temporary destination {destination:?} is outside the world")]
    InvalidDestination {
        household: HouseholdId,
        destination: CellId,
    },
    #[error("household {household:?} temporary destination equals residence {residence:?}")]
    DestinationIsResidence {
        household: HouseholdId,
        residence: CellId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityValidationError {
    #[error("temporary mobility schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error(
        "temporary mobility has {state} household states but population has {population} households"
    )]
    HouseholdCountMismatch { state: usize, population: usize },
    #[error("household {household:?} has invalid temporary presence: {reason}")]
    InvalidPresence {
        household: HouseholdId,
        reason: String,
    },
    #[error("temporary journey {journey:?} is active for more than one household")]
    DuplicateActiveJourney { journey: TemporaryJourneyId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{PopulationConfig, WorldConfig},
        rng::RngFactory,
    };

    fn fixture(seed: u64) -> (World, Population) {
        let world = World::generate(WorldConfig::new(8, 8), RngFactory::new(seed)).unwrap();
        let population = Population::initialize(
            PopulationConfig::new(20).with_target_household_size(5),
            &world,
            RngFactory::new(seed),
        )
        .unwrap();
        (world, population)
    }

    fn different_cell(world: &World, residence: CellId) -> CellId {
        (1..=world.cell_count() as u64)
            .map(CellId::new)
            .find(|&cell| cell != residence)
            .unwrap()
    }

    #[test]
    fn founders_begin_at_residence() {
        let (world, population) = fixture(7);
        let state = TemporaryMobilityState::at_residence(&population);
        assert!(state.all_at_residence());
        for raw in 1..=population.household_count() as u64 {
            let household = HouseholdId::new(raw);
            assert_eq!(state.is_at_residence(household), Some(true));
            assert_eq!(
                state.current_cell(household, &population),
                population.household_location(household)
            );
        }
        state.validate(&population, &world).unwrap();
    }

    #[test]
    fn transit_has_no_arbitrary_cell_and_visit_preserves_residence() {
        let (world, population) = fixture(11);
        let mut state = TemporaryMobilityState::at_residence(&population);
        let household = HouseholdId::new(1);
        let residence = population.household_location(household).unwrap();
        let destination = different_cell(&world, residence);
        state
            .set_presence(
                household,
                HouseholdPresence::OutboundTransit {
                    journey: TemporaryJourneyId::new(1),
                    destination,
                },
                &population,
                &world,
            )
            .unwrap();
        assert_eq!(state.current_cell(household, &population), None);
        assert_eq!(population.household_location(household), Some(residence));

        state
            .set_presence(
                household,
                HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination,
                },
                &population,
                &world,
            )
            .unwrap();
        assert_eq!(state.current_cell(household, &population), Some(destination));
        assert_eq!(population.household_location(household), Some(residence));
    }

    #[test]
    fn duplicate_active_journey_ids_are_rejected() {
        let (world, population) = fixture(17);
        let mut state = TemporaryMobilityState::at_residence(&population);
        let journey = TemporaryJourneyId::new(3);
        for raw in 1..=2 {
            let household = HouseholdId::new(raw);
            let destination = different_cell(
                &world,
                population.household_location(household).unwrap(),
            );
            state
                .set_presence(
                    household,
                    HouseholdPresence::Visiting {
                        journey,
                        destination,
                    },
                    &population,
                    &world,
                )
                .unwrap();
        }
        assert_eq!(
            state.validate(&population, &world),
            Err(TemporaryMobilityValidationError::DuplicateActiveJourney { journey })
        );
    }

    #[test]
    fn destination_cannot_equal_residence() {
        let (world, population) = fixture(19);
        let mut state = TemporaryMobilityState::at_residence(&population);
        let household = HouseholdId::new(1);
        let residence = population.household_location(household).unwrap();
        assert_eq!(
            state.set_presence(
                household,
                HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination: residence,
                },
                &population,
                &world,
            ),
            Err(TemporaryMobilityError::DestinationIsResidence {
                household,
                residence,
            })
        );
    }

    #[test]
    fn digest_changes_with_active_presence() {
        let (world, population) = fixture(23);
        let mut state = TemporaryMobilityState::at_residence(&population);
        let baseline = state.digest64();
        let household = HouseholdId::new(1);
        let destination = different_cell(
            &world,
            population.household_location(household).unwrap(),
        );
        state
            .set_presence(
                household,
                HouseholdPresence::ReturnTransit {
                    journey: TemporaryJourneyId::new(1),
                    destination,
                },
                &population,
                &world,
            )
            .unwrap();
        assert_ne!(state.digest64(), baseline);
    }

    #[test]
    fn last_member_death_retires_active_presence() {
        let (world, mut population) = fixture(29);
        let mut state = TemporaryMobilityState::at_residence(&population);
        let household = HouseholdId::new(1);
        let destination = different_cell(
            &world,
            population.household_location(household).unwrap(),
        );
        state
            .set_presence(
                household,
                HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination,
                },
                &population,
                &world,
            )
            .unwrap();

        for index in 0..population.person_count() {
            if population.household_at_index(index) == Some(household) {
                assert!(population.mark_death(index, 10));
            }
        }
        assert!(state.validate(&population, &world).is_err());
        state.reconcile_after_population_change(&population);
        assert_eq!(state.presence(household), Some(HouseholdPresence::AtResidence));
        state.validate(&population, &world).unwrap();
    }
}
