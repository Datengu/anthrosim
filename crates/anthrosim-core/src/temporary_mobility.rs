use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    events::{EventKind, EventLog, TemporaryJourneyIneligibility},
    evidence::EvidenceCatalog,
    focal_region::{FocalRegion, FocalRegionError, FocalRegionSource},
    ids::{CellId, HouseholdId, TemporaryJourneyId},
    population::Population,
    temporary_resource::{
        TemporaryResourceAccountingError, TemporaryResourceLedger, TemporaryResourcePeriod,
    },
    temporary_travel::{TemporaryTravelModel, TemporaryTravelModelError},
    world::World,
};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const TEMPORARY_EVENT_SCHEMA_VERSION: u32 = 2;
const M9_DESTINATION_TIE_POLICY_ID: &str = "m9/equal-cost-destination-keyed-v1";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryTriggerTiming {
    DepartureDay,
    TargetArrivalDay,
}

/// Exogenous M9 v1 trigger schedule. Trigger days are strictly increasing simulation days.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilitySchedule {
    pub schema_version: u32,
    pub schedule_id: String,
    pub trigger_timing: TemporaryTriggerTiming,
    pub trigger_days: Vec<u64>,
    pub stay_duration_days: u32,
}

impl TemporaryMobilitySchedule {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        schedule_id: impl Into<String>,
        trigger_timing: TemporaryTriggerTiming,
        trigger_days: Vec<u64>,
        stay_duration_days: u32,
    ) -> Result<Self, TemporaryMobilityProgramError> {
        let schedule = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            schedule_id: schedule_id.into(),
            trigger_timing,
            trigger_days,
            stay_duration_days,
        };
        schedule.validate()?;
        Ok(schedule)
    }

    pub fn validate(&self) -> Result<(), TemporaryMobilityProgramError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryMobilityProgramError::UnsupportedScheduleSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.schedule_id.trim().is_empty() {
            return Err(TemporaryMobilityProgramError::EmptyScheduleId);
        }
        if self.trigger_days.is_empty() {
            return Err(TemporaryMobilityProgramError::EmptyTriggerSchedule);
        }
        if self.trigger_days.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(TemporaryMobilityProgramError::NonCanonicalTriggerDays);
        }
        if self.stay_duration_days == 0 {
            return Err(TemporaryMobilityProgramError::ZeroStayDuration);
        }
        Ok(())
    }

    fn digest_into(&self, hash: &mut u64) {
        digest_u64(hash, u64::from(self.schema_version));
        digest_str(hash, &self.schedule_id);
        digest_u64(
            hash,
            match self.trigger_timing {
                TemporaryTriggerTiming::DepartureDay => 0,
                TemporaryTriggerTiming::TargetArrivalDay => 1,
            },
        );
        digest_u64(hash, self.trigger_days.len() as u64);
        for &day in &self.trigger_days {
            digest_u64(hash, day);
        }
        digest_u64(hash, u64::from(self.stay_duration_days));
    }
}

/// World-independent immutable M9 experiment definition.
///
/// The focal region and schedule are fixed experiment inputs, while M9.4 routing is deliberately
/// resolved from each run's authoritative world. This prevents a travel table derived from one
/// synthetic seed from being silently reused against another seed's movement-cost field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityConfig {
    pub schema_version: u32,
    pub region: FocalRegion,
    pub schedule: TemporaryMobilitySchedule,
    pub travel_model: TemporaryTravelModel,
}

impl TemporaryMobilityConfig {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn new(
        region: FocalRegion,
        schedule: TemporaryMobilitySchedule,
        travel_model: TemporaryTravelModel,
    ) -> Result<Self, TemporaryMobilityConfigError> {
        let config = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            region,
            schedule,
            travel_model,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), TemporaryMobilityConfigError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryMobilityConfigError::UnsupportedSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        self.region.validate_structure()?;
        self.schedule.validate()?;
        self.travel_model.validate()?;
        Ok(())
    }

    /// Validate evidence provenance claimed by an evidence-bound focal-region source.
    ///
    /// A serialized M9 definition must not be able to claim that its region came from a
    /// landscape mask while referring to an external evidence input that is absent from the
    /// experiment catalogue. Synthetic regions do not require an evidence catalogue.
    pub fn validate_evidence_context(
        &self,
        catalog: Option<&EvidenceCatalog>,
    ) -> Result<(), TemporaryMobilityConfigError> {
        self.validate()?;
        let FocalRegionSource::LandscapeMask {
            evidence_input_id, ..
        } = &self.region.source
        else {
            return Ok(());
        };
        let catalog =
            catalog.ok_or_else(|| TemporaryMobilityConfigError::MissingEvidenceCatalog {
                input_id: evidence_input_id.clone(),
            })?;
        if !catalog
            .external_inputs
            .iter()
            .any(|input| input.input_id == *evidence_input_id)
        {
            return Err(TemporaryMobilityConfigError::UnknownEvidenceInput {
                input_id: evidence_input_id.clone(),
            });
        }
        Ok(())
    }

    pub fn derive_program(
        &self,
        world: &World,
    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {
        self.derive_program_with_seed(world, 0)
    }

    pub fn derive_program_with_seed(
        &self,
        world: &World,
        destination_tie_seed: u64,
    ) -> Result<TemporaryMobilityProgram, TemporaryMobilityConfigError> {
        self.validate()?;
        self.region.validate(world)?;
        let travel = self.travel_model.derive_table_with_tie_seed(
            &self.region,
            world,
            destination_tie_seed,
        )?;
        Ok(TemporaryMobilityProgram::new(
            self.region.clone(),
            self.schedule.clone(),
            travel,
            world,
        )?)
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_u64(&mut hash, self.region.digest64());
        self.schedule.digest_into(&mut hash);
        digest_str(&mut hash, &self.travel_model.identity());
        hash
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "temporary-mobility-config-v{}-{:016x}",
            self.schema_version,
            self.digest64()
        )
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityConfigError {
    #[error(
        "temporary-mobility configuration schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error(
        "temporary-mobility focal region references evidence external input {input_id}, but no evidence catalogue was supplied"
    )]
    MissingEvidenceCatalog { input_id: String },
    #[error(
        "temporary-mobility focal region references unknown evidence external input {input_id}"
    )]
    UnknownEvidenceInput { input_id: String },
    #[error(transparent)]
    Region(#[from] FocalRegionError),
    #[error(transparent)]
    Program(#[from] TemporaryMobilityProgramError),
    #[error(transparent)]
    TravelModel(#[from] TemporaryTravelModelError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryTravelDestinationCandidate {
    pub destination: CellId,
    pub route_distance_edges: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TemporaryTravelResolution {
    Unreachable,
    Reachable {
        destination: CellId,
        outbound_travel_days: u32,
        return_travel_days: u32,
    },
}

/// Pre-resolved M9.3 travel input indexed by authoritative origin cell.
///
/// M9.3 consumes this table but does not define how it is produced. M9.4 is responsible for
/// deriving the authoritative table from model-facing movement cost and deterministic routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryTravelTable {
    pub schema_version: u32,
    resolutions: Vec<TemporaryTravelResolution>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    travel_model: Option<TemporaryTravelModel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accumulated_cost_units: Option<Vec<Option<u64>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    equal_cost_destinations: Option<Vec<Vec<TemporaryTravelDestinationCandidate>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    destination_tie_seed: Option<u64>,
}

impl TemporaryTravelTable {
    pub const CURRENT_SCHEMA_VERSION: u32 = 3;

    pub fn new(
        resolutions: Vec<TemporaryTravelResolution>,
        region: &FocalRegion,
        world: &World,
    ) -> Result<Self, TemporaryMobilityProgramError> {
        let table = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            resolutions,
            travel_model: None,
            accumulated_cost_units: None,
            equal_cost_destinations: None,
            destination_tie_seed: None,
        };
        table.validate(region, world)?;
        Ok(table)
    }

    pub(crate) fn new_m9_4(
        resolutions: Vec<TemporaryTravelResolution>,
        accumulated_cost_units: Vec<Option<u64>>,
        equal_cost_destinations: Vec<Vec<TemporaryTravelDestinationCandidate>>,
        destination_tie_seed: u64,
        travel_model: TemporaryTravelModel,
        region: &FocalRegion,
        world: &World,
    ) -> Result<Self, TemporaryMobilityProgramError> {
        let table = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            resolutions,
            travel_model: Some(travel_model),
            accumulated_cost_units: Some(accumulated_cost_units),
            equal_cost_destinations: Some(equal_cost_destinations),
            destination_tie_seed: Some(destination_tie_seed),
        };
        table.validate(region, world)?;
        Ok(table)
    }

    #[must_use]
    pub fn resolution(&self, origin: CellId) -> Option<TemporaryTravelResolution> {
        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;
        self.resolutions.get(index).copied()
    }

    #[must_use]
    pub fn travel_model(&self) -> Option<&TemporaryTravelModel> {
        self.travel_model.as_ref()
    }

    #[must_use]
    pub fn accumulated_cost_units(&self, origin: CellId) -> Option<u64> {
        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;
        self.accumulated_cost_units
            .as_ref()?
            .get(index)
            .copied()
            .flatten()
    }

    #[must_use]
    pub fn equal_cost_destinations(
        &self,
        origin: CellId,
    ) -> Option<&[TemporaryTravelDestinationCandidate]> {
        let index = usize::try_from(origin.0.checked_sub(1)?).ok()?;
        self.equal_cost_destinations
            .as_ref()?
            .get(index)
            .map(Vec::as_slice)
    }

    #[must_use]
    pub fn equal_cost_destination_count(&self, origin: CellId) -> Option<u32> {
        self.equal_cost_destinations(origin)?.len().try_into().ok()
    }

    #[must_use]
    pub fn route_distance_edges(&self, origin: CellId, destination: CellId) -> Option<u32> {
        self.equal_cost_destinations(origin)?
            .iter()
            .find(|candidate| candidate.destination == destination)
            .map(|candidate| candidate.route_distance_edges)
    }

    /// Resolve one household/trigger destination without consuming a mutable RNG stream.
    #[must_use]
    pub fn resolution_for(
        &self,
        origin: CellId,
        household: HouseholdId,
        trigger_index: u32,
    ) -> Option<TemporaryTravelResolution> {
        let base = self.resolution(origin)?;
        let TemporaryTravelResolution::Reachable {
            outbound_travel_days,
            return_travel_days,
            ..
        } = base
        else {
            return Some(base);
        };
        let Some(candidates) = self.equal_cost_destinations(origin) else {
            return Some(base);
        };
        if candidates.len() <= 1 {
            return Some(base);
        }
        let mut hash = FNV_OFFSET_BASIS;
        digest_str(&mut hash, M9_DESTINATION_TIE_POLICY_ID);
        digest_u64(&mut hash, self.destination_tie_seed.unwrap_or(0));
        digest_u64(&mut hash, origin.0);
        digest_u64(&mut hash, household.0);
        digest_u64(&mut hash, u64::from(trigger_index));
        hash = avalanche64(hash);
        let index = usize::try_from(hash % candidates.len() as u64).ok()?;
        Some(TemporaryTravelResolution::Reachable {
            destination: candidates[index].destination,
            outbound_travel_days,
            return_travel_days,
        })
    }

    pub fn validate(
        &self,
        region: &FocalRegion,
        world: &World,
    ) -> Result<(), TemporaryMobilityProgramError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(
                TemporaryMobilityProgramError::UnsupportedTravelTableSchema {
                    found: self.schema_version,
                    supported: Self::CURRENT_SCHEMA_VERSION,
                },
            );
        }
        if self.resolutions.len() != world.cell_count() {
            return Err(TemporaryMobilityProgramError::TravelTableShapeMismatch {
                table: self.resolutions.len(),
                world: world.cell_count(),
            });
        }

        let m9_4 = match (
            &self.travel_model,
            &self.accumulated_cost_units,
            &self.equal_cost_destinations,
            self.destination_tie_seed,
        ) {
            (None, None, None, None) => None,
            (Some(model), Some(costs), Some(candidates), Some(_tie_seed)) => {
                model.validate().map_err(|error| {
                    TemporaryMobilityProgramError::InvalidTravelModel {
                        reason: error.to_string(),
                    }
                })?;
                if costs.len() != world.cell_count() {
                    return Err(
                        TemporaryMobilityProgramError::TravelCostTableShapeMismatch {
                            table: costs.len(),
                            world: world.cell_count(),
                        },
                    );
                }
                if candidates.len() != world.cell_count() {
                    return Err(
                        TemporaryMobilityProgramError::TravelDestinationCandidateShapeMismatch {
                            table: candidates.len(),
                            world: world.cell_count(),
                        },
                    );
                }
                for &cell in region.member_cells() {
                    if !model.is_traversable(world, cell) {
                        return Err(TemporaryMobilityProgramError::TravelRegionCellImpassable {
                            cell,
                        });
                    }
                }
                Some((model, costs, candidates))
            }
            _ => {
                return Err(TemporaryMobilityProgramError::IncompleteTravelCostMetadata);
            }
        };

        for (index, resolution) in self.resolutions.iter().enumerate() {
            let origin = CellId::new(index as u64 + 1);
            if let TemporaryTravelResolution::Reachable { destination, .. } = resolution {
                if world.cell(*destination).is_none() {
                    return Err(
                        TemporaryMobilityProgramError::TravelDestinationOutsideWorld {
                            origin,
                            destination: *destination,
                        },
                    );
                }
                if !region.contains(*destination) {
                    return Err(
                        TemporaryMobilityProgramError::TravelDestinationOutsideRegion {
                            origin,
                            destination: *destination,
                        },
                    );
                }
                if !region.contains(origin) && *destination == origin {
                    return Err(TemporaryMobilityProgramError::TravelDestinationIsOrigin {
                        origin,
                    });
                }
            }

            if let Some((model, costs, candidates)) = m9_4 {
                match (*resolution, costs[index], candidates[index].as_slice()) {
                    (TemporaryTravelResolution::Unreachable, None, []) => {}
                    (
                        TemporaryTravelResolution::Reachable {
                            destination,
                            outbound_travel_days,
                            return_travel_days,
                        },
                        Some(cost),
                        candidates,
                    ) if !candidates.is_empty() => {
                        if candidates[0].destination != destination
                            || candidates
                                .windows(2)
                                .any(|pair| pair[0].destination >= pair[1].destination)
                            || candidates.iter().any(|candidate| {
                                world.cell(candidate.destination).is_none()
                                    || !region.contains(candidate.destination)
                            })
                        {
                            return Err(
                                TemporaryMobilityProgramError::InvalidTravelDestinationCandidates {
                                    origin,
                                },
                            );
                        }
                        let expected = model.travel_days(cost).map_err(|error| {
                            TemporaryMobilityProgramError::InvalidTravelModel {
                                reason: error.to_string(),
                            }
                        })?;
                        if outbound_travel_days != expected || return_travel_days != expected {
                            return Err(
                                TemporaryMobilityProgramError::TravelDurationCostMismatch {
                                    origin,
                                    expected,
                                    outbound: outbound_travel_days,
                                    returning: return_travel_days,
                                },
                            );
                        }
                    }
                    _ => {
                        return Err(
                            TemporaryMobilityProgramError::TravelCostResolutionMismatch { origin },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn digest_into(&self, hash: &mut u64) {
        digest_u64(hash, u64::from(self.schema_version));
        digest_u64(hash, self.resolutions.len() as u64);
        for resolution in &self.resolutions {
            match *resolution {
                TemporaryTravelResolution::Unreachable => digest_u64(hash, 0),
                TemporaryTravelResolution::Reachable {
                    destination,
                    outbound_travel_days,
                    return_travel_days,
                } => {
                    digest_u64(hash, 1);
                    digest_u64(hash, destination.0);
                    digest_u64(hash, u64::from(outbound_travel_days));
                    digest_u64(hash, u64::from(return_travel_days));
                }
            }
        }
        match &self.travel_model {
            None => digest_u64(hash, 0),
            Some(model) => {
                digest_u64(hash, 1);
                digest_str(hash, &model.identity());
            }
        }
        match &self.accumulated_cost_units {
            None => digest_u64(hash, 0),
            Some(costs) => {
                digest_u64(hash, 1);
                digest_u64(hash, costs.len() as u64);
                for cost in costs {
                    match cost {
                        None => digest_u64(hash, 0),
                        Some(cost) => {
                            digest_u64(hash, 1);
                            digest_u64(hash, *cost);
                        }
                    }
                }
            }
        }
        match &self.equal_cost_destinations {
            None => digest_u64(hash, 0),
            Some(rows) => {
                digest_u64(hash, 1);
                digest_u64(hash, rows.len() as u64);
                for row in rows {
                    digest_u64(hash, row.len() as u64);
                    for candidate in row {
                        digest_u64(hash, candidate.destination.0);
                        digest_u64(hash, u64::from(candidate.route_distance_edges));
                    }
                }
            }
        }
        match self.destination_tie_seed {
            None => digest_u64(hash, 0),
            Some(seed) => {
                digest_u64(hash, 1);
                digest_u64(hash, seed);
            }
        }
    }
}

/// Immutable M9.3 execution input: one focal region, one exogenous schedule and a resolved travel
/// table. Social motivation and routing semantics are deliberately outside this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityProgram {
    pub schema_version: u32,
    pub region: FocalRegion,
    pub schedule: TemporaryMobilitySchedule,
    pub travel: TemporaryTravelTable,
}

impl TemporaryMobilityProgram {
    pub const CURRENT_SCHEMA_VERSION: u32 = 2;

    pub fn new(
        region: FocalRegion,
        schedule: TemporaryMobilitySchedule,
        travel: TemporaryTravelTable,
        world: &World,
    ) -> Result<Self, TemporaryMobilityProgramError> {
        let program = Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            region,
            schedule,
            travel,
        };
        program.validate(world)?;
        Ok(program)
    }

    pub fn validate(&self, world: &World) -> Result<(), TemporaryMobilityProgramError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryMobilityProgramError::UnsupportedProgramSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        self.region.validate(world)?;
        self.schedule.validate()?;
        self.travel.validate(&self.region, world)?;
        Ok(())
    }

    #[must_use]
    pub fn digest64(&self) -> u64 {
        let mut hash = FNV_OFFSET_BASIS;
        digest_u64(&mut hash, u64::from(self.schema_version));
        digest_u64(&mut hash, self.region.digest64());
        self.schedule.digest_into(&mut hash);
        self.travel.digest_into(&mut hash);
        hash
    }

    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "temporary-mobility-program-v{}-{:016x}",
            self.schema_version,
            self.digest64()
        )
    }
}

/// Timing and causal identity required to continue one active temporary journey exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTemporaryJourney {
    pub journey: TemporaryJourneyId,
    pub household: HouseholdId,
    pub region_id: String,
    pub region_identity: String,
    pub trigger_index: Option<u32>,
    pub trigger_day: u64,
    pub residence: CellId,
    pub destination: CellId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub travel_model_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accumulated_travel_cost_units: Option<u64>,
    pub departure_day: u64,
    pub arrival_day: u64,
    pub return_departure_day: u64,
    pub completion_day: u64,
    pub outbound_travel_days: u32,
    pub return_travel_days: u32,
}

impl ActiveTemporaryJourney {
    fn validate(
        &self,
        population: &Population,
        world: &World,
    ) -> Result<(), TemporaryMobilityError> {
        if self.journey == TemporaryJourneyId::INVALID {
            return Err(TemporaryMobilityError::InvalidJourney {
                household: self.household,
            });
        }
        let residence = population.household_location(self.household).ok_or(
            TemporaryMobilityError::InvalidHousehold {
                household: self.household,
            },
        )?;
        if residence != self.residence {
            return Err(TemporaryMobilityError::ResidenceChangedDuringJourney {
                household: self.household,
                expected: self.residence,
                actual: residence,
            });
        }
        if world.cell(self.destination).is_none() {
            return Err(TemporaryMobilityError::InvalidDestination {
                household: self.household,
                destination: self.destination,
            });
        }
        if self.destination == residence {
            return Err(TemporaryMobilityError::DestinationIsResidence {
                household: self.household,
                residence,
            });
        }
        if self.arrival_day
            != self
                .departure_day
                .saturating_add(u64::from(self.outbound_travel_days))
            || self.return_departure_day <= self.arrival_day
            || self.completion_day
                != self
                    .return_departure_day
                    .saturating_add(u64::from(self.return_travel_days))
        {
            return Err(TemporaryMobilityError::InvalidJourneyTiming {
                household: self.household,
                journey: self.journey,
            });
        }
        if self.region_id.trim().is_empty() || self.region_identity.trim().is_empty() {
            return Err(TemporaryMobilityError::InvalidRegionIdentity {
                household: self.household,
                journey: self.journey,
            });
        }
        match (
            self.travel_model_identity.as_deref(),
            self.accumulated_travel_cost_units,
        ) {
            (None, None) => {}
            (Some(identity), Some(_)) if !identity.trim().is_empty() => {}
            _ => {
                return Err(TemporaryMobilityError::InvalidTravelMetadata {
                    household: self.household,
                    journey: self.journey,
                });
            }
        }
        Ok(())
    }

    fn digest_into(&self, hash: &mut u64) {
        digest_u64(hash, self.journey.0);
        digest_u64(hash, self.household.0);
        digest_str(hash, &self.region_id);
        digest_str(hash, &self.region_identity);
        digest_u64(hash, self.trigger_index.map_or(u64::MAX, u64::from));
        digest_u64(hash, self.trigger_day);
        digest_u64(hash, self.residence.0);
        digest_u64(hash, self.destination.0);
        match &self.travel_model_identity {
            None => digest_u64(hash, 0),
            Some(identity) => {
                digest_u64(hash, 1);
                digest_str(hash, identity);
            }
        }
        match self.accumulated_travel_cost_units {
            None => digest_u64(hash, 0),
            Some(cost) => {
                digest_u64(hash, 1);
                digest_u64(hash, cost);
            }
        }
        digest_u64(hash, self.departure_day);
        digest_u64(hash, self.arrival_day);
        digest_u64(hash, self.return_departure_day);
        digest_u64(hash, self.completion_day);
        digest_u64(hash, u64::from(self.outbound_travel_days));
        digest_u64(hash, u64::from(self.return_travel_days));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessedTemporaryTrigger {
    trigger_index: u32,
    household: HouseholdId,
}

/// Compact authoritative presence and scheduler state parallel to persistent household residence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryMobilityState {
    pub schema_version: u32,
    household_presence: Vec<HouseholdPresence>,
    active_journeys: Vec<Option<ActiveTemporaryJourney>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    program: Option<TemporaryMobilityProgram>,
    processed_triggers: Vec<ProcessedTemporaryTrigger>,
    next_journey_id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resource_ledger: Option<TemporaryResourceLedger>,
}

impl TemporaryMobilityState {
    pub const CURRENT_SCHEMA_VERSION: u32 = 4;

    #[must_use]
    pub fn at_residence(population: &Population) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            household_presence: vec![HouseholdPresence::AtResidence; population.household_count()],
            active_journeys: vec![None; population.household_count()],
            program: None,
            processed_triggers: Vec::new(),
            next_journey_id: 1,
            resource_ledger: None,
        }
    }

    pub fn with_program(
        population: &Population,
        program: TemporaryMobilityProgram,
        world: &World,
    ) -> Result<Self, TemporaryMobilityProgramError> {
        program.validate(world)?;
        let mut state = Self::at_residence(population);
        state.program = Some(program);
        state.resource_ledger = Some(TemporaryResourceLedger::new(
            population.household_count(),
            0,
        ));
        Ok(state)
    }

    pub fn enable_program(
        &mut self,
        program: TemporaryMobilityProgram,
        population: &Population,
        world: &World,
    ) -> Result<(), TemporaryMobilityProgramError> {
        if self.program.is_some()
            || !self.all_at_residence()
            || self.active_journeys.iter().any(Option::is_some)
            || !self.processed_triggers.is_empty()
            || self.next_journey_id != 1
        {
            return Err(TemporaryMobilityProgramError::ProgramAlreadyInitialized);
        }
        if self.household_count() != population.household_count() {
            return Err(TemporaryMobilityProgramError::StateHouseholdCountMismatch);
        }
        program.validate(world)?;
        self.program = Some(program);
        self.resource_ledger = Some(TemporaryResourceLedger::new(
            population.household_count(),
            0,
        ));
        Ok(())
    }

    #[must_use]
    pub fn program(&self) -> Option<&TemporaryMobilityProgram> {
        self.program.as_ref()
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
    pub fn active_journey(&self, household: HouseholdId) -> Option<&ActiveTemporaryJourney> {
        let index = household_index(household, self.household_count())?;
        self.active_journeys.get(index)?.as_ref()
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

    /// True only for the exact disabled-M9 state. This is the compatibility boundary used by the
    /// legacy state-digest path.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.program.is_none()
            && self.all_at_residence()
            && self.active_journeys.iter().all(Option::is_none)
            && self.processed_triggers.is_empty()
            && self.next_journey_id == 1
            && self.resource_ledger.is_none()
    }

    pub(crate) fn resource_period_snapshot(
        &mut self,
        day: u64,
        world: &World,
    ) -> Result<Option<TemporaryResourcePeriod>, TemporaryMobilityExecutionError> {
        let Some(ledger) = self.resource_ledger.as_mut() else {
            return Ok(None);
        };
        Ok(Some(ledger.snapshot_period(
            day,
            &self.household_presence,
            world,
        )?))
    }

    pub(crate) fn complete_resource_period(
        &mut self,
        day: u64,
    ) -> Result<(), TemporaryMobilityExecutionError> {
        if let Some(ledger) = self.resource_ledger.as_mut() {
            ledger.reset_after_settlement(day)?;
        }
        Ok(())
    }

    /// Remove active temporary state for households with no living members.
    pub(crate) fn reconcile_after_population_change(&mut self, population: &Population) {
        for index in 0..self.household_count() {
            let household = HouseholdId::new(index as u64 + 1);
            if self.household_presence[index].is_at_residence() {
                continue;
            }
            if household_living_count(population, household) == 0 {
                self.household_presence[index] = HouseholdPresence::AtResidence;
                self.active_journeys[index] = None;
            }
        }
    }

    /// Return the next temporary-mobility boundary on or after `current_day` and no later than
    /// `end_day`. The scan is household-based and event-driven; it does not iterate through days.
    pub(crate) fn next_boundary_day(
        &self,
        current_day: u64,
        end_day: u64,
        population: &Population,
    ) -> Result<Option<u64>, TemporaryMobilityExecutionError> {
        let mut next = None;

        for (index, presence) in self.household_presence.iter().copied().enumerate() {
            let Some(active) = self.active_journeys[index].as_ref() else {
                continue;
            };
            let due = match presence {
                HouseholdPresence::AtResidence => continue,
                HouseholdPresence::OutboundTransit { .. } => active.arrival_day,
                HouseholdPresence::Visiting { .. } => active.return_departure_day,
                HouseholdPresence::ReturnTransit { .. } => active.completion_day,
            };
            if due < current_day {
                return Err(TemporaryMobilityExecutionError::OverdueActiveTransition {
                    household: active.household,
                    journey: active.journey,
                    due,
                    current_day,
                });
            }
            if due <= end_day {
                next = Some(next.map_or(due, |prior: u64| prior.min(due)));
            }
        }

        let Some(program) = self.program.as_ref() else {
            return Ok(next);
        };
        for (trigger_index, &trigger_day) in program.schedule.trigger_days.iter().enumerate() {
            let trigger_index = u32::try_from(trigger_index)
                .map_err(|_| TemporaryMobilityExecutionError::TooManyTriggers)?;
            for raw in 1..=self.household_count() as u64 {
                let household = HouseholdId::new(raw);
                if self.trigger_processed(trigger_index, household) {
                    continue;
                }
                let evaluation_day = trigger_evaluation_day(
                    program,
                    trigger_day,
                    household,
                    population,
                    current_day,
                )?;
                if evaluation_day < current_day {
                    return Err(TemporaryMobilityExecutionError::OverdueTrigger {
                        household,
                        trigger_index,
                        evaluation_day,
                        current_day,
                    });
                }
                if evaluation_day <= end_day {
                    next = Some(next.map_or(evaluation_day, |prior| prior.min(evaluation_day)));
                }
            }
        }
        Ok(next)
    }

    /// Apply all M9 lifecycle transitions and trigger evaluations due on one authoritative day.
    ///
    /// Ordering is frozen as: return completions, arrivals, return departures, new outward
    /// departures. Zero-day transit closes immediately after the corresponding start event.
    pub(crate) fn process_day(
        &mut self,
        day: u64,
        population: &Population,
        world: &World,
        events: &mut EventLog,
    ) -> Result<TemporaryMobilityDayOutcome, TemporaryMobilityExecutionError> {
        if let Some(ledger) = self.resource_ledger.as_mut() {
            ledger.accrue_until(day, &self.household_presence)?;
        }
        let mut outcome = TemporaryMobilityDayOutcome::default();

        // Complete return transit first, in stable household order.
        for index in 0..self.household_count() {
            let should_complete = matches!(
                self.household_presence[index],
                HouseholdPresence::ReturnTransit { .. }
            ) && self.active_journeys[index]
                .as_ref()
                .is_some_and(|active| active.completion_day == day);
            if should_complete {
                self.complete_journey(index, day, population, events)?;
                outcome.completed = outcome.completed.saturating_add(1);
            }
        }

        // Then arrivals.
        for index in 0..self.household_count() {
            let should_arrive = matches!(
                self.household_presence[index],
                HouseholdPresence::OutboundTransit { .. }
            ) && self.active_journeys[index]
                .as_ref()
                .is_some_and(|active| active.arrival_day == day);
            if should_arrive {
                self.arrive(index, day, population, events)?;
                outcome.arrived = outcome.arrived.saturating_add(1);
            }
        }

        // Return departures are starts and therefore occur after completions/arrivals.
        for index in 0..self.household_count() {
            let should_return = matches!(
                self.household_presence[index],
                HouseholdPresence::Visiting { .. }
            ) && self.active_journeys[index]
                .as_ref()
                .is_some_and(|active| active.return_departure_day == day);
            if should_return {
                let completed_immediately = self.start_return(index, day, population, events)?;
                outcome.return_departed = outcome.return_departed.saturating_add(1);
                if completed_immediately {
                    outcome.completed = outcome.completed.saturating_add(1);
                }
            }
        }

        let Some(program) = self.program.clone() else {
            self.validate(population, world)
                .map_err(TemporaryMobilityExecutionError::InvalidState)?;
            return Ok(outcome);
        };

        // New departures are last. Trigger index then household ID defines stable ordering.
        for (trigger_index_usize, &trigger_day) in program.schedule.trigger_days.iter().enumerate()
        {
            let trigger_index = u32::try_from(trigger_index_usize)
                .map_err(|_| TemporaryMobilityExecutionError::TooManyTriggers)?;
            for raw in 1..=self.household_count() as u64 {
                let household = HouseholdId::new(raw);
                if self.trigger_processed(trigger_index, household) {
                    continue;
                }
                let evaluation_day =
                    trigger_evaluation_day(&program, trigger_day, household, population, day)?;
                if evaluation_day != day {
                    continue;
                }

                match self.evaluate_trigger(
                    &program,
                    trigger_index,
                    trigger_day,
                    household,
                    day,
                    population,
                    world,
                    events,
                )? {
                    TriggerEvaluation::Departed {
                        arrived_immediately,
                    } => {
                        outcome.departed = outcome.departed.saturating_add(1);
                        if arrived_immediately {
                            outcome.arrived = outcome.arrived.saturating_add(1);
                        }
                    }
                    TriggerEvaluation::Skipped(reason) => {
                        outcome.skipped.push(TemporaryJourneySkip {
                            household,
                            trigger_index,
                            reason,
                        });
                    }
                }
            }
        }

        self.validate(population, world)
            .map_err(TemporaryMobilityExecutionError::InvalidState)?;
        Ok(outcome)
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
        if self.household_count() != population.household_count()
            || self.active_journeys.len() != population.household_count()
        {
            return Err(TemporaryMobilityValidationError::HouseholdCountMismatch {
                presence: self.household_count(),
                journeys: self.active_journeys.len(),
                population: population.household_count(),
            });
        }
        if self.next_journey_id == 0 {
            return Err(TemporaryMobilityValidationError::InvalidNextJourneyId);
        }
        if self.program.is_some() != self.resource_ledger.is_some() {
            return Err(TemporaryMobilityValidationError::ResourceLedgerProgramMismatch);
        }
        if let Some(program) = &self.program {
            program.validate(world).map_err(|error| {
                TemporaryMobilityValidationError::InvalidProgram {
                    reason: error.to_string(),
                }
            })?;
        } else if !self.processed_triggers.is_empty() {
            return Err(TemporaryMobilityValidationError::ProcessedTriggersWithoutProgram);
        }
        if self
            .processed_triggers
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(TemporaryMobilityValidationError::NonCanonicalProcessedTriggers);
        }

        let mut active_journeys = BTreeSet::new();
        let mut max_journey = 0_u64;
        for index in 0..self.household_count() {
            let household = HouseholdId::new(index as u64 + 1);
            let presence = self.household_presence[index];
            validate_presence(household, presence, population, world).map_err(|error| {
                TemporaryMobilityValidationError::InvalidPresence {
                    household,
                    reason: error.to_string(),
                }
            })?;

            match (
                presence.active_journey(),
                self.active_journeys[index].as_ref(),
            ) {
                (None, None) => {}
                (Some(journey), Some(active)) => {
                    active.validate(population, world).map_err(|error| {
                        TemporaryMobilityValidationError::InvalidActiveJourney {
                            household,
                            reason: error.to_string(),
                        }
                    })?;
                    if active.household != household
                        || active.journey != journey
                        || presence.destination() != Some(active.destination)
                    {
                        return Err(TemporaryMobilityValidationError::PresenceJourneyMismatch {
                            household,
                        });
                    }
                    if let Some(program) = &self.program {
                        let expected_model_identity =
                            program.travel.travel_model().map(|model| model.identity());
                        let expected_cost = program.travel.accumulated_cost_units(active.residence);
                        let resolution_matches = matches!(
                            program.travel.resolution(active.residence),
                            Some(TemporaryTravelResolution::Reachable {
                                destination,
                                outbound_travel_days,
                                return_travel_days,
                            }) if destination == active.destination
                                && outbound_travel_days == active.outbound_travel_days
                                && return_travel_days == active.return_travel_days
                        );
                        if active.region_id != program.region.region_id
                            || active.region_identity != program.region.identity()
                            || !resolution_matches
                            || active.travel_model_identity != expected_model_identity
                            || active.accumulated_travel_cost_units != expected_cost
                        {
                            return Err(
                                TemporaryMobilityValidationError::ActiveJourneyProgramMismatch {
                                    household,
                                },
                            );
                        }
                    }
                    if !active_journeys.insert(journey) {
                        return Err(TemporaryMobilityValidationError::DuplicateActiveJourney {
                            journey,
                        });
                    }
                    max_journey = max_journey.max(journey.0);
                }
                _ => {
                    return Err(TemporaryMobilityValidationError::PresenceJourneyMismatch {
                        household,
                    });
                }
            }
        }
        if self.next_journey_id <= max_journey {
            return Err(TemporaryMobilityValidationError::NextJourneyIdNotAhead {
                next: self.next_journey_id,
                active_max: max_journey,
            });
        }

        if let Some(program) = &self.program {
            for processed in &self.processed_triggers {
                if usize::try_from(processed.trigger_index)
                    .ok()
                    .is_none_or(|index| index >= program.schedule.trigger_days.len())
                    || household_index(processed.household, self.household_count()).is_none()
                {
                    return Err(TemporaryMobilityValidationError::InvalidProcessedTrigger);
                }
            }
        }
        Ok(())
    }

    pub fn validate_at_day(
        &self,
        day: u64,
        population: &Population,
        world: &World,
    ) -> Result<(), TemporaryMobilityValidationError> {
        self.validate(population, world)?;
        if let Some(ledger) = &self.resource_ledger {
            ledger
                .validate(self.household_count(), world, day)
                .map_err(
                    |error| TemporaryMobilityValidationError::InvalidResourceLedger {
                        reason: error.to_string(),
                    },
                )?;
        }
        for index in 0..self.household_count() {
            let Some(active) = self.active_journeys[index].as_ref() else {
                continue;
            };
            let valid = match self.household_presence[index] {
                HouseholdPresence::AtResidence => false,
                HouseholdPresence::OutboundTransit { .. } => {
                    active.departure_day <= day && day < active.arrival_day
                }
                HouseholdPresence::Visiting { .. } => {
                    active.arrival_day <= day && day < active.return_departure_day
                }
                HouseholdPresence::ReturnTransit { .. } => {
                    active.return_departure_day <= day && day < active.completion_day
                }
            };
            if !valid {
                return Err(TemporaryMobilityValidationError::PresenceTimingMismatch {
                    household: active.household,
                    journey: active.journey,
                    day,
                });
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
        digest_u64(&mut hash, self.active_journeys.len() as u64);
        for active in &self.active_journeys {
            match active {
                None => digest_u64(&mut hash, 0),
                Some(active) => {
                    digest_u64(&mut hash, 1);
                    active.digest_into(&mut hash);
                }
            }
        }
        match &self.program {
            None => digest_u64(&mut hash, 0),
            Some(program) => {
                digest_u64(&mut hash, 1);
                digest_u64(&mut hash, program.digest64());
            }
        }
        digest_u64(&mut hash, self.processed_triggers.len() as u64);
        for processed in &self.processed_triggers {
            digest_u64(&mut hash, u64::from(processed.trigger_index));
            digest_u64(&mut hash, processed.household.0);
        }
        digest_u64(&mut hash, self.next_journey_id);
        match &self.resource_ledger {
            None => digest_u64(&mut hash, 0),
            Some(ledger) => {
                digest_u64(&mut hash, 1);
                ledger.digest_into(&mut hash);
            }
        }
        hash
    }

    fn trigger_processed(&self, trigger_index: u32, household: HouseholdId) -> bool {
        self.processed_triggers
            .binary_search(&ProcessedTemporaryTrigger {
                trigger_index,
                household,
            })
            .is_ok()
    }

    fn mark_trigger_processed(&mut self, trigger_index: u32, household: HouseholdId) {
        let key = ProcessedTemporaryTrigger {
            trigger_index,
            household,
        };
        match self.processed_triggers.binary_search(&key) {
            Ok(_) => {}
            Err(index) => self.processed_triggers.insert(index, key),
        }
    }

    fn evaluate_trigger(
        &mut self,
        program: &TemporaryMobilityProgram,
        trigger_index: u32,
        trigger_day: u64,
        household: HouseholdId,
        day: u64,
        population: &Population,
        world: &World,
        events: &mut EventLog,
    ) -> Result<TriggerEvaluation, TemporaryMobilityExecutionError> {
        let reason = if household_living_count(population, household) == 0 {
            Some(TemporaryJourneyIneligibility::NoLivingMembers)
        } else if self
            .presence(household)
            .is_some_and(|presence| !presence.is_at_residence())
        {
            Some(TemporaryJourneyIneligibility::ActiveJourney)
        } else if program
            .region
            .contains_residence(household, population)
            .unwrap_or(false)
        {
            Some(TemporaryJourneyIneligibility::ResidenceInRegion)
        } else {
            None
        };
        if let Some(reason) = reason {
            self.record_skip(
                program,
                trigger_index,
                trigger_day,
                household,
                day,
                reason,
                events,
            );
            return Ok(TriggerEvaluation::Skipped(reason));
        }

        let residence = population
            .household_location(household)
            .ok_or(TemporaryMobilityExecutionError::InvalidHousehold { household })?;
        let resolution = program
            .travel
            .resolution_for(residence, household, trigger_index)
            .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;
        let TemporaryTravelResolution::Reachable {
            destination,
            outbound_travel_days,
            return_travel_days,
        } = resolution
        else {
            let reason = TemporaryJourneyIneligibility::Unreachable;
            self.record_skip(
                program,
                trigger_index,
                trigger_day,
                household,
                day,
                reason,
                events,
            );
            return Ok(TriggerEvaluation::Skipped(reason));
        };

        let travel_model_identity = program.travel.travel_model().map(|model| model.identity());
        let accumulated_travel_cost_units = if travel_model_identity.is_some() {
            Some(
                program
                    .travel
                    .accumulated_cost_units(residence)
                    .ok_or(TemporaryMobilityExecutionError::MissingTravelCost { residence })?,
            )
        } else {
            None
        };

        let departure_day = match program.schedule.trigger_timing {
            TemporaryTriggerTiming::DepartureDay => trigger_day,
            TemporaryTriggerTiming::TargetArrivalDay => {
                let Some(departure) = trigger_day.checked_sub(u64::from(outbound_travel_days))
                else {
                    let reason = TemporaryJourneyIneligibility::DepartureBeforeSimulationStart;
                    self.record_skip(
                        program,
                        trigger_index,
                        trigger_day,
                        household,
                        day,
                        reason,
                        events,
                    );
                    return Ok(TriggerEvaluation::Skipped(reason));
                };
                departure
            }
        };
        if departure_day < day {
            let reason = TemporaryJourneyIneligibility::DepartureWindowMissed;
            self.record_skip(
                program,
                trigger_index,
                trigger_day,
                household,
                day,
                reason,
                events,
            );
            return Ok(TriggerEvaluation::Skipped(reason));
        }
        if departure_day != day {
            return Err(
                TemporaryMobilityExecutionError::TriggerEvaluatedOnWrongDay {
                    household,
                    trigger_index,
                    expected: departure_day,
                    actual: day,
                },
            );
        }

        let journey = TemporaryJourneyId::new(self.next_journey_id);
        self.next_journey_id = self
            .next_journey_id
            .checked_add(1)
            .ok_or(TemporaryMobilityExecutionError::JourneyIdExhausted)?;
        let arrival_day = departure_day
            .checked_add(u64::from(outbound_travel_days))
            .ok_or(TemporaryMobilityExecutionError::JourneyTimeOverflow)?;
        let return_departure_day = arrival_day
            .checked_add(u64::from(program.schedule.stay_duration_days))
            .ok_or(TemporaryMobilityExecutionError::JourneyTimeOverflow)?;
        let completion_day = return_departure_day
            .checked_add(u64::from(return_travel_days))
            .ok_or(TemporaryMobilityExecutionError::JourneyTimeOverflow)?;
        let active = ActiveTemporaryJourney {
            journey,
            household,
            region_id: program.region.region_id.clone(),
            region_identity: program.region.identity(),
            trigger_index: Some(trigger_index),
            trigger_day,
            residence,
            destination,
            travel_model_identity: travel_model_identity.clone(),
            accumulated_travel_cost_units,
            departure_day,
            arrival_day,
            return_departure_day,
            completion_day,
            outbound_travel_days,
            return_travel_days,
        };
        active
            .validate(population, world)
            .map_err(TemporaryMobilityExecutionError::InvalidJourney)?;
        let index = household_index(household, self.household_count())
            .ok_or(TemporaryMobilityExecutionError::InvalidHousehold { household })?;
        self.active_journeys[index] = Some(active.clone());
        self.household_presence[index] = HouseholdPresence::OutboundTransit {
            journey,
            destination,
        };
        self.mark_trigger_processed(trigger_index, household);
        let people_affected = household_living_count(population, household);
        events.push_authoritative(
            day,
            EventKind::TemporaryJourneyDeparted {
                event_schema_version: TEMPORARY_EVENT_SCHEMA_VERSION,
                household,
                journey,
                region_id: active.region_id.clone(),
                region_identity: active.region_identity.clone(),
                residence,
                destination,
                travel_model_identity,
                accumulated_travel_cost_units,
                people_affected,
                trigger_index,
                trigger_day,
                departure_day,
                arrival_day,
                return_departure_day,
                completion_day,
                outbound_travel_days,
                return_travel_days,
            },
        );

        let mut arrived_immediately = false;
        if outbound_travel_days == 0 {
            self.arrive(index, day, population, events)?;
            arrived_immediately = true;
        }
        Ok(TriggerEvaluation::Departed {
            arrived_immediately,
        })
    }

    fn record_skip(
        &mut self,
        program: &TemporaryMobilityProgram,
        trigger_index: u32,
        trigger_day: u64,
        household: HouseholdId,
        day: u64,
        reason: TemporaryJourneyIneligibility,
        events: &mut EventLog,
    ) {
        self.mark_trigger_processed(trigger_index, household);
        events.push_authoritative(
            day,
            EventKind::TemporaryJourneyNotStarted {
                event_schema_version: TEMPORARY_EVENT_SCHEMA_VERSION,
                household,
                region_id: program.region.region_id.clone(),
                region_identity: program.region.identity(),
                trigger_index,
                trigger_day,
                reason,
            },
        );
    }

    fn arrive(
        &mut self,
        index: usize,
        day: u64,
        population: &Population,
        events: &mut EventLog,
    ) -> Result<(), TemporaryMobilityExecutionError> {
        let active = self.active_journeys[index]
            .as_ref()
            .ok_or(TemporaryMobilityExecutionError::MissingActiveJourney)?
            .clone();
        if active.arrival_day != day {
            return Err(TemporaryMobilityExecutionError::TransitionDayMismatch {
                journey: active.journey,
                expected: active.arrival_day,
                actual: day,
            });
        }
        self.household_presence[index] = HouseholdPresence::Visiting {
            journey: active.journey,
            destination: active.destination,
        };
        events.push_authoritative(
            day,
            EventKind::TemporaryJourneyArrived {
                event_schema_version: TEMPORARY_EVENT_SCHEMA_VERSION,
                household: active.household,
                journey: active.journey,
                region_id: active.region_id,
                region_identity: active.region_identity,
                destination: active.destination,
                people_affected: household_living_count(population, active.household),
            },
        );
        Ok(())
    }

    fn start_return(
        &mut self,
        index: usize,
        day: u64,
        population: &Population,
        events: &mut EventLog,
    ) -> Result<bool, TemporaryMobilityExecutionError> {
        let active = self.active_journeys[index]
            .as_ref()
            .ok_or(TemporaryMobilityExecutionError::MissingActiveJourney)?
            .clone();
        if active.return_departure_day != day {
            return Err(TemporaryMobilityExecutionError::TransitionDayMismatch {
                journey: active.journey,
                expected: active.return_departure_day,
                actual: day,
            });
        }
        self.household_presence[index] = HouseholdPresence::ReturnTransit {
            journey: active.journey,
            destination: active.destination,
        };
        events.push_authoritative(
            day,
            EventKind::TemporaryReturnDeparted {
                event_schema_version: TEMPORARY_EVENT_SCHEMA_VERSION,
                household: active.household,
                journey: active.journey,
                region_id: active.region_id.clone(),
                region_identity: active.region_identity.clone(),
                destination: active.destination,
                residence: active.residence,
                people_affected: household_living_count(population, active.household),
            },
        );
        if active.return_travel_days == 0 {
            self.complete_journey(index, day, population, events)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn complete_journey(
        &mut self,
        index: usize,
        day: u64,
        population: &Population,
        events: &mut EventLog,
    ) -> Result<(), TemporaryMobilityExecutionError> {
        let active = self.active_journeys[index]
            .as_ref()
            .ok_or(TemporaryMobilityExecutionError::MissingActiveJourney)?
            .clone();
        if active.completion_day != day {
            return Err(TemporaryMobilityExecutionError::TransitionDayMismatch {
                journey: active.journey,
                expected: active.completion_day,
                actual: day,
            });
        }
        self.household_presence[index] = HouseholdPresence::AtResidence;
        self.active_journeys[index] = None;
        events.push_authoritative(
            day,
            EventKind::TemporaryJourneyCompleted {
                event_schema_version: TEMPORARY_EVENT_SCHEMA_VERSION,
                household: active.household,
                journey: active.journey,
                region_id: active.region_id,
                region_identity: active.region_identity,
                residence: active.residence,
                people_affected: household_living_count(population, active.household),
            },
        );
        Ok(())
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
        self.active_journeys[index] = match presence {
            HouseholdPresence::AtResidence => None,
            HouseholdPresence::OutboundTransit {
                journey,
                destination,
            } => Some(test_active_journey(
                household,
                journey,
                destination,
                population,
                0,
                999_998,
                999_999,
                1_000_000,
            )),
            HouseholdPresence::Visiting {
                journey,
                destination,
            } => Some(test_active_journey(
                household,
                journey,
                destination,
                population,
                0,
                0,
                999_999,
                1_000_000,
            )),
            HouseholdPresence::ReturnTransit {
                journey,
                destination,
            } => Some(test_active_journey(
                household,
                journey,
                destination,
                population,
                0,
                0,
                1,
                1_000_000,
            )),
        };
        if let Some(active) = &self.active_journeys[index] {
            self.next_journey_id = self.next_journey_id.max(active.journey.0.saturating_add(1));
        }
        Ok(())
    }
}

#[cfg(test)]
fn test_active_journey(
    household: HouseholdId,
    journey: TemporaryJourneyId,
    destination: CellId,
    population: &Population,
    departure_day: u64,
    arrival_day: u64,
    return_departure_day: u64,
    completion_day: u64,
) -> ActiveTemporaryJourney {
    ActiveTemporaryJourney {
        journey,
        household,
        region_id: "test-region".to_owned(),
        region_identity: "test-region-identity".to_owned(),
        trigger_index: None,
        trigger_day: departure_day,
        residence: population.household_location(household).unwrap(),
        destination,
        travel_model_identity: None,
        accumulated_travel_cost_units: None,
        departure_day,
        arrival_day,
        return_departure_day,
        completion_day,
        outbound_travel_days: u32::try_from(arrival_day.saturating_sub(departure_day))
            .unwrap_or(u32::MAX),
        return_travel_days: u32::try_from(completion_day.saturating_sub(return_departure_day))
            .unwrap_or(u32::MAX),
    }
}

fn trigger_evaluation_day(
    program: &TemporaryMobilityProgram,
    trigger_day: u64,
    household: HouseholdId,
    population: &Population,
    current_day: u64,
) -> Result<u64, TemporaryMobilityExecutionError> {
    if household_living_count(population, household) == 0
        || program
            .region
            .contains_residence(household, population)
            .unwrap_or(false)
    {
        return Ok(trigger_day);
    }
    let residence = population
        .household_location(household)
        .ok_or(TemporaryMobilityExecutionError::InvalidHousehold { household })?;
    let resolution = program
        .travel
        .resolution(residence)
        .ok_or(TemporaryMobilityExecutionError::MissingTravelResolution { residence })?;
    match program.schedule.trigger_timing {
        TemporaryTriggerTiming::DepartureDay => Ok(trigger_day),
        TemporaryTriggerTiming::TargetArrivalDay => {
            let TemporaryTravelResolution::Reachable {
                outbound_travel_days,
                ..
            } = resolution
            else {
                return Ok(trigger_day);
            };
            let Some(departure_day) = trigger_day.checked_sub(u64::from(outbound_travel_days))
            else {
                // A pre-simulation departure is residence-dependent. Keep the trigger pending until
                // the target boundary so an intervening M4 relocation can make a future departure
                // feasible; if it never does, evaluate_trigger records the explicit pre-start skip.
                return Ok(trigger_day);
            };
            if departure_day < current_day && trigger_day >= current_day {
                Ok(trigger_day)
            } else {
                Ok(departure_day)
            }
        }
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
    if household_living_count(population, household) == 0 {
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

fn household_living_count(population: &Population, household: HouseholdId) -> u32 {
    u32::try_from(
        (0..population.person_count())
            .filter(|&index| {
                population.is_alive_index(index)
                    && population.household_at_index(index) == Some(household)
            })
            .count(),
    )
    .unwrap_or(u32::MAX)
}

fn household_index(household: HouseholdId, household_count: usize) -> Option<usize> {
    let index = usize::try_from(household.0.checked_sub(1)?).ok()?;
    (index < household_count).then_some(index)
}

fn digest_str(hash: &mut u64, value: &str) {
    digest_u64(hash, value.len() as u64);
    for byte in value.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

fn avalanche64(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemporaryMobilityDayOutcome {
    pub departed: u64,
    pub arrived: u64,
    pub return_departed: u64,
    pub completed: u64,
    pub skipped: Vec<TemporaryJourneySkip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporaryJourneySkip {
    pub household: HouseholdId,
    pub trigger_index: u32,
    pub reason: TemporaryJourneyIneligibility,
}

enum TriggerEvaluation {
    Departed { arrived_immediately: bool },
    Skipped(TemporaryJourneyIneligibility),
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityProgramError {
    #[error(
        "temporary mobility program schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedProgramSchema { found: u32, supported: u32 },
    #[error(
        "temporary mobility schedule schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedScheduleSchema { found: u32, supported: u32 },
    #[error(
        "temporary travel-table schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedTravelTableSchema { found: u32, supported: u32 },
    #[error("temporary mobility schedule identifier is empty")]
    EmptyScheduleId,
    #[error("temporary mobility schedule has no trigger days")]
    EmptyTriggerSchedule,
    #[error("temporary mobility trigger days are not strictly increasing")]
    NonCanonicalTriggerDays,
    #[error("temporary mobility stay duration must be at least one day")]
    ZeroStayDuration,
    #[error("temporary travel table has {table} entries but world has {world} cells")]
    TravelTableShapeMismatch { table: usize, world: usize },
    #[error("temporary travel-cost table has {table} entries but world has {world} cells")]
    TravelCostTableShapeMismatch { table: usize, world: usize },
    #[error(
        "temporary equal-cost destination table has {table} entries but world has {world} cells"
    )]
    TravelDestinationCandidateShapeMismatch { table: usize, world: usize },
    #[error("temporary travel table has incomplete M9.4 model/cost metadata")]
    IncompleteTravelCostMetadata,
    #[error("temporary travel model is invalid: {reason}")]
    InvalidTravelModel { reason: String },
    #[error("temporary travel M9.4 cost presence does not match resolution for {origin:?}")]
    TravelCostResolutionMismatch { origin: CellId },
    #[error("temporary travel equal-cost destination metadata is invalid for {origin:?}")]
    InvalidTravelDestinationCandidates { origin: CellId },
    #[error(
        "temporary travel duration for {origin:?} does not match stored cost: expected {expected}, outbound {outbound}, return {returning}"
    )]
    TravelDurationCostMismatch {
        origin: CellId,
        expected: u32,
        outbound: u32,
        returning: u32,
    },
    #[error("focal-region cell {cell:?} is impassable under the stored temporary travel model")]
    TravelRegionCellImpassable { cell: CellId },
    #[error(
        "temporary travel from {origin:?} resolves destination {destination:?} outside the world"
    )]
    TravelDestinationOutsideWorld { origin: CellId, destination: CellId },
    #[error(
        "temporary travel from {origin:?} resolves destination {destination:?} outside the focal region"
    )]
    TravelDestinationOutsideRegion { origin: CellId, destination: CellId },
    #[error("temporary travel from {origin:?} resolves to the same non-region origin")]
    TravelDestinationIsOrigin { origin: CellId },
    #[error("temporary mobility program is already initialized or state is already active")]
    ProgramAlreadyInitialized,
    #[error("temporary mobility state household count does not match the population")]
    StateHouseholdCountMismatch,
    #[error(transparent)]
    Region(#[from] FocalRegionError),
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
    #[error(
        "household {household:?} residence changed during active journey: expected {expected:?}, found {actual:?}"
    )]
    ResidenceChangedDuringJourney {
        household: HouseholdId,
        expected: CellId,
        actual: CellId,
    },
    #[error("household {household:?} journey {journey:?} has invalid timing")]
    InvalidJourneyTiming {
        household: HouseholdId,
        journey: TemporaryJourneyId,
    },
    #[error("household {household:?} journey {journey:?} has invalid region identity")]
    InvalidRegionIdentity {
        household: HouseholdId,
        journey: TemporaryJourneyId,
    },
    #[error("household {household:?} journey {journey:?} has invalid travel metadata")]
    InvalidTravelMetadata {
        household: HouseholdId,
        journey: TemporaryJourneyId,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryMobilityValidationError {
    #[error("temporary mobility schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error(
        "temporary mobility has {presence} presence states and {journeys} journey slots but population has {population} households"
    )]
    HouseholdCountMismatch {
        presence: usize,
        journeys: usize,
        population: usize,
    },
    #[error("temporary mobility program is invalid: {reason}")]
    InvalidProgram { reason: String },
    #[error("temporary mobility has processed triggers but no configured program")]
    ProcessedTriggersWithoutProgram,
    #[error("temporary mobility processed-trigger set is not strictly ordered and duplicate-free")]
    NonCanonicalProcessedTriggers,
    #[error("temporary mobility processed-trigger entry is invalid")]
    InvalidProcessedTrigger,
    #[error("temporary mobility next journey ID is invalid")]
    InvalidNextJourneyId,
    #[error("temporary mobility program and M9.5 resource ledger are not enabled together")]
    ResourceLedgerProgramMismatch,
    #[error("temporary mobility resource ledger is invalid: {reason}")]
    InvalidResourceLedger { reason: String },
    #[error(
        "temporary mobility next journey ID {next} is not greater than active maximum {active_max}"
    )]
    NextJourneyIdNotAhead { next: u64, active_max: u64 },
    #[error("household {household:?} has invalid temporary presence: {reason}")]
    InvalidPresence {
        household: HouseholdId,
        reason: String,
    },
    #[error("household {household:?} has invalid active journey: {reason}")]
    InvalidActiveJourney {
        household: HouseholdId,
        reason: String,
    },
    #[error("household {household:?} presence does not match its active journey record")]
    PresenceJourneyMismatch { household: HouseholdId },
    #[error(
        "household {household:?} active journey does not match the configured temporary mobility program"
    )]
    ActiveJourneyProgramMismatch { household: HouseholdId },
    #[error("temporary journey {journey:?} is active for more than one household")]
    DuplicateActiveJourney { journey: TemporaryJourneyId },
    #[error(
        "household {household:?} journey {journey:?} presence is inconsistent with checkpoint day {day}"
    )]
    PresenceTimingMismatch {
        household: HouseholdId,
        journey: TemporaryJourneyId,
        day: u64,
    },
}

#[derive(Debug, Error)]
pub enum TemporaryMobilityExecutionError {
    #[error(transparent)]
    InvalidState(TemporaryMobilityValidationError),
    #[error(transparent)]
    ResourceAccounting(#[from] TemporaryResourceAccountingError),
    #[error(transparent)]
    InvalidJourney(TemporaryMobilityError),
    #[error("temporary mobility references invalid household {household:?}")]
    InvalidHousehold { household: HouseholdId },
    #[error("temporary mobility travel table has no entry for residence {residence:?}")]
    MissingTravelResolution { residence: CellId },
    #[error(
        "temporary mobility M9.4 table has no travel cost for reachable residence {residence:?}"
    )]
    MissingTravelCost { residence: CellId },
    #[error("temporary mobility trigger count exceeds supported u32 identity")]
    TooManyTriggers,
    #[error("temporary journey ID space exhausted")]
    JourneyIdExhausted,
    #[error("temporary journey timing overflowed simulation day range")]
    JourneyTimeOverflow,
    #[error("temporary journey active record is missing")]
    MissingActiveJourney,
    #[error("temporary journey {journey:?} transition expected day {expected} but ran on {actual}")]
    TransitionDayMismatch {
        journey: TemporaryJourneyId,
        expected: u64,
        actual: u64,
    },
    #[error(
        "household {household:?} journey {journey:?} has overdue transition day {due} before current day {current_day}"
    )]
    OverdueActiveTransition {
        household: HouseholdId,
        journey: TemporaryJourneyId,
        due: u64,
        current_day: u64,
    },
    #[error(
        "household {household:?} trigger {trigger_index} has overdue evaluation day {evaluation_day} before current day {current_day}"
    )]
    OverdueTrigger {
        household: HouseholdId,
        trigger_index: u32,
        evaluation_day: u64,
        current_day: u64,
    },
    #[error(
        "household {household:?} trigger {trigger_index} evaluated on day {actual}, expected {expected}"
    )]
    TriggerEvaluatedOnWrongDay {
        household: HouseholdId,
        trigger_index: u32,
        expected: u64,
        actual: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{PopulationConfig, WorldConfig},
        focal_region::FocalRegionSource,
        rng::RngFactory,
    };

    fn fixture(seed: u64) -> (World, Population) {
        let world = World::generate(WorldConfig::new(4, 4), RngFactory::new(seed)).unwrap();
        let population = Population::initialize(
            PopulationConfig::new(20).with_target_household_size(5),
            &world,
            RngFactory::new(seed),
        )
        .unwrap();
        (world, population)
    }

    fn program(
        world: &World,
        population: &Population,
        trigger_timing: TemporaryTriggerTiming,
        trigger_days: Vec<u64>,
        travel_days: u32,
    ) -> TemporaryMobilityProgram {
        let residence = population.household_location(HouseholdId::new(1)).unwrap();
        let destination = (1..=world.cell_count() as u64)
            .map(CellId::new)
            .find(|&cell| cell != residence)
            .unwrap();
        let region = FocalRegion::new(
            "test-region",
            FocalRegionSource::Synthetic,
            vec![destination],
        )
        .unwrap();
        let resolutions = (1..=world.cell_count() as u64)
            .map(|raw| {
                let origin = CellId::new(raw);
                if region.contains(origin) {
                    TemporaryTravelResolution::Unreachable
                } else {
                    TemporaryTravelResolution::Reachable {
                        destination,
                        outbound_travel_days: travel_days,
                        return_travel_days: travel_days,
                    }
                }
            })
            .collect();
        let travel = TemporaryTravelTable::new(resolutions, &region, world).unwrap();
        TemporaryMobilityProgram::new(
            region,
            TemporaryMobilitySchedule::new("test-schedule", trigger_timing, trigger_days, 3)
                .unwrap(),
            travel,
            world,
        )
        .unwrap()
    }

    fn run_boundaries(
        state: &mut TemporaryMobilityState,
        population: &Population,
        world: &World,
        end_day: u64,
        events: &mut EventLog,
    ) {
        let mut current = 0;
        while let Some(day) = state
            .next_boundary_day(current, end_day, population)
            .unwrap()
        {
            state.process_day(day, population, world, events).unwrap();
            current = day;
            if day == end_day {
                break;
            }
        }
    }

    #[test]
    fn target_arrival_lifecycle_uses_half_open_visit_interval() {
        let (world, population) = fixture(7);
        let program = program(
            &world,
            &population,
            TemporaryTriggerTiming::TargetArrivalDay,
            vec![10],
            2,
        );
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();

        assert_eq!(
            state.next_boundary_day(0, 20, &population).unwrap(),
            Some(8)
        );
        state
            .process_day(8, &population, &world, &mut events)
            .unwrap();
        assert!(matches!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::OutboundTransit { .. })
        ));
        state
            .process_day(10, &population, &world, &mut events)
            .unwrap();
        assert!(matches!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::Visiting { .. })
        ));
        state
            .process_day(13, &population, &world, &mut events)
            .unwrap();
        assert!(matches!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::ReturnTransit { .. })
        ));
        state
            .process_day(15, &population, &world, &mut events)
            .unwrap();
        assert_eq!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::AtResidence)
        );
    }

    #[test]
    fn repeated_departure_schedule_allocates_stable_journey_ids() {
        let (world, population) = fixture(11);
        let program = program(
            &world,
            &population,
            TemporaryTriggerTiming::DepartureDay,
            vec![2, 20],
            1,
        );
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();
        run_boundaries(&mut state, &population, &world, 30, &mut events);

        let departed: Vec<_> = events
            .events
            .iter()
            .filter_map(|record| match record.event {
                EventKind::TemporaryJourneyDeparted {
                    household, journey, ..
                } if household == HouseholdId::new(1) => Some(journey),
                _ => None,
            })
            .collect();
        assert_eq!(
            departed,
            vec![TemporaryJourneyId::new(1), TemporaryJourneyId::new(5)]
        );
        assert!(state.all_at_residence());
    }

    #[test]
    fn resident_household_is_explicitly_skipped() {
        let (world, population) = fixture(13);
        let residence = population.household_location(HouseholdId::new(1)).unwrap();
        let region = FocalRegion::new(
            "resident-region",
            FocalRegionSource::Synthetic,
            vec![residence],
        )
        .unwrap();
        let travel = TemporaryTravelTable::new(
            vec![TemporaryTravelResolution::Unreachable; world.cell_count()],
            &region,
            &world,
        )
        .unwrap();
        let program = TemporaryMobilityProgram::new(
            region,
            TemporaryMobilitySchedule::new(
                "resident-skip",
                TemporaryTriggerTiming::DepartureDay,
                vec![5],
                2,
            )
            .unwrap(),
            travel,
            &world,
        )
        .unwrap();
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();
        let outcome = state
            .process_day(5, &population, &world, &mut events)
            .unwrap();
        assert!(outcome.skipped.iter().any(|skip| {
            skip.household == HouseholdId::new(1)
                && skip.reason == TemporaryJourneyIneligibility::ResidenceInRegion
        }));
    }

    #[test]
    fn zero_day_transit_closes_on_start_day_without_daily_loop() {
        let (world, population) = fixture(17);
        let program = program(
            &world,
            &population,
            TemporaryTriggerTiming::DepartureDay,
            vec![5],
            0,
        );
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();
        state
            .process_day(5, &population, &world, &mut events)
            .unwrap();
        assert!(matches!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::Visiting { .. })
        ));
        state
            .process_day(8, &population, &world, &mut events)
            .unwrap();
        assert_eq!(
            state.presence(HouseholdId::new(1)),
            Some(HouseholdPresence::AtResidence)
        );
    }

    #[test]
    fn active_journey_must_match_program_travel_resolution_exactly() {
        let (world, population) = fixture(23);
        let program = program(
            &world,
            &population,
            TemporaryTriggerTiming::DepartureDay,
            vec![5],
            2,
        );
        let mut state = TemporaryMobilityState::with_program(&population, program, &world).unwrap();
        let mut events = EventLog::new();
        state
            .process_day(5, &population, &world, &mut events)
            .unwrap();

        let active = state.active_journeys[0]
            .as_mut()
            .expect("household 1 should have an active journey");
        active.outbound_travel_days = active.outbound_travel_days.saturating_add(1);
        active.arrival_day = active
            .departure_day
            .saturating_add(u64::from(active.outbound_travel_days));
        active.return_departure_day = active.arrival_day.saturating_add(3);
        active.completion_day = active
            .return_departure_day
            .saturating_add(u64::from(active.return_travel_days));

        assert!(matches!(
            state.validate(&population, &world),
            Err(TemporaryMobilityValidationError::ActiveJourneyProgramMismatch { household })
                if household == HouseholdId::new(1)
        ));
    }

    #[test]
    fn disabled_state_keeps_legacy_compatibility_boundary() {
        let (world, population) = fixture(19);
        let state = TemporaryMobilityState::at_residence(&population);
        state.validate(&population, &world).unwrap();
        assert!(state.is_disabled());
        assert!(state.all_at_residence());
    }
}
