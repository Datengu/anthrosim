use thiserror::Error;

use crate::{
    config::{
        DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID, FIXED_FOUNDER_HOUSEHOLD_LIFECYCLE_ID,
        HouseholdLifecycleConfig,
    },
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
    if config.model_id != DETERMINISTIC_SIZE_FISSION_HOUSEHOLD_LIFECYCLE_ID {
        return Err(HouseholdLifecycleError::UnsupportedModel {
            model_id: config.model_id.clone(),
        });
    }
    if config.max_living_members == 0 {
        return Err(HouseholdLifecycleError::ZeroMaximumLivingMembers);
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
    } = population.fission_oversized_households(config.max_living_members, &eligible)?;
    temporary_mobility.reconcile_household_topology_at_boundary(population, day)?;
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
    #[error("household identity does not fit supported u64 space")]
    HouseholdIdOverflow,
    #[error("temporary mobility has no presence state for household {household:?}")]
    MissingTemporaryPresence { household: HouseholdId },
    #[error(transparent)]
    Population(#[from] PopulationError),
    #[error(transparent)]
    TemporaryMobility(#[from] TemporaryMobilityExecutionError),
}
