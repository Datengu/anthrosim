use thiserror::Error;

use crate::{
    config::{
        DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID,
        FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID, HouseholdLifecycleConfig,
    },
    events::{EventKind, EventLog, HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION},
    ids::HouseholdId,
    population::{HouseholdFissionOutcome, Population, PopulationError},
    temporary_mobility::{TemporaryMobilityExecutionError, TemporaryMobilityState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HouseholdLifecycleOutcome {
    pub households_created: u64,
    pub people_reassigned: u64,
}

pub fn validate_household_lifecycle_config(
    config: &HouseholdLifecycleConfig,
) -> Result<(), HouseholdLifecycleError> {
    if config.schema_version != HouseholdLifecycleConfig::CURRENT_SCHEMA_VERSION {
        return Err(HouseholdLifecycleError::UnsupportedSchema {
            found: config.schema_version,
            supported: HouseholdLifecycleConfig::CURRENT_SCHEMA_VERSION,
        });
    }
    if config.model_id != DETERMINISTIC_DEPENDENCY_FISSION_HOUSEHOLD_LIFECYCLE_ID {
        return Err(HouseholdLifecycleError::UnsupportedModel {
            model_id: config.model_id.clone(),
        });
    }
    if config.max_living_members == 0 {
        return Err(HouseholdLifecycleError::ZeroMaximumLivingMembers);
    }
    if config.minimum_independent_age_years == 0 {
        return Err(HouseholdLifecycleError::ZeroMinimumIndependentAgeYears);
    }
    Ok(())
}

#[must_use]
pub fn household_lifecycle_model_id(config: Option<&HouseholdLifecycleConfig>) -> &str {
    config
        .map(|value| value.model_id.as_str())
        .unwrap_or(FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID)
}

pub(crate) fn apply_household_lifecycle_at_annual_boundary(
    population: &mut Population,
    temporary_mobility: &mut TemporaryMobilityState,
    events: &mut EventLog,
    config: &HouseholdLifecycleConfig,
    day: u64,
) -> Result<HouseholdLifecycleOutcome, HouseholdLifecycleError> {
    validate_household_lifecycle_config(config)?;
    let household_count = population.household_count();
    let mut eligible = Vec::with_capacity(household_count);
    for index in 0..household_count {
        let household = HouseholdId::new(
            u64::try_from(index)
                .map_err(|_| HouseholdLifecycleError::HouseholdIdOverflow)?
                .checked_add(1)
                .ok_or(HouseholdLifecycleError::HouseholdIdOverflow)?,
        );
        eligible.push(
            temporary_mobility
                .is_at_residence(household)
                .ok_or(HouseholdLifecycleError::MissingTemporaryPresence { household })?,
        );
    }
    let HouseholdFissionOutcome {
        households_created,
        people_reassigned,
        fissions,
    } = population.fission_oversized_households(
        config.max_living_members,
        config.minimum_independent_age_years,
        day,
        &eligible,
    )?;
    temporary_mobility.reconcile_household_topology_at_boundary(population, day)?;
    for fission in fissions {
        events.push_authoritative(
            day,
            EventKind::HouseholdFission {
                event_schema_version: HOUSEHOLD_FISSION_EVENT_SCHEMA_VERSION,
                source_household: fission.source_household,
                new_household: fission.new_household,
                residence: fission.residence,
                people_reassigned: fission.people_reassigned,
            },
        );
    }
    Ok(HouseholdLifecycleOutcome {
        households_created,
        people_reassigned,
    })
}

#[derive(Debug, Error)]
pub enum HouseholdLifecycleError {
    #[error("household lifecycle schema {found} is unsupported; supported schema is {supported}")]
    UnsupportedSchema { found: u32, supported: u32 },
    #[error("household lifecycle model {model_id:?} is unsupported")]
    UnsupportedModel { model_id: String },
    #[error("household lifecycle maximum living members must be greater than zero")]
    ZeroMaximumLivingMembers,
    #[error("household lifecycle minimum independent age must be greater than zero")]
    ZeroMinimumIndependentAgeYears,
    #[error("household identity does not fit supported u64 space")]
    HouseholdIdOverflow,
    #[error("temporary mobility has no presence state for household {household:?}")]
    MissingTemporaryPresence { household: HouseholdId },
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error(transparent)]
    TemporaryMobility(#[from] TemporaryMobilityExecutionError),
}
