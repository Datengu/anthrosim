use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ids::CellId, temporary_mobility::HouseholdPresence, world::World};

const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Duration counts for one household within the current/settled resource period.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryResourcePresenceDays {
    pub at_residence_days: u64,
    pub outbound_transit_days: u64,
    pub visiting_days: u64,
    pub return_transit_days: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visitor_destination: Option<CellId>,
}

impl TemporaryResourcePresenceDays {
    pub fn total_days(&self) -> Result<u64, TemporaryResourceAccountingError> {
        self.at_residence_days
            .checked_add(self.outbound_transit_days)
            .and_then(|value| value.checked_add(self.visiting_days))
            .and_then(|value| value.checked_add(self.return_transit_days))
            .ok_or(TemporaryResourceAccountingError::DurationOverflow)
    }

    pub fn home_provisioning_days(&self) -> Result<u64, TemporaryResourceAccountingError> {
        self.at_residence_days
            .checked_add(self.outbound_transit_days)
            .and_then(|value| value.checked_add(self.return_transit_days))
            .ok_or(TemporaryResourceAccountingError::DurationOverflow)
    }

    fn accrue(
        &mut self,
        presence: HouseholdPresence,
        duration: u64,
    ) -> Result<(), TemporaryResourceAccountingError> {
        match presence {
            HouseholdPresence::AtResidence => {
                self.at_residence_days = self
                    .at_residence_days
                    .checked_add(duration)
                    .ok_or(TemporaryResourceAccountingError::DurationOverflow)?;
            }
            HouseholdPresence::OutboundTransit { .. } => {
                self.outbound_transit_days = self
                    .outbound_transit_days
                    .checked_add(duration)
                    .ok_or(TemporaryResourceAccountingError::DurationOverflow)?;
            }
            HouseholdPresence::Visiting { destination, .. } => {
                if self
                    .visitor_destination
                    .is_some_and(|existing| existing != destination)
                {
                    return Err(
                        TemporaryResourceAccountingError::VisitorDestinationChanged {
                            previous: self.visitor_destination.expect("checked Some"),
                            next: destination,
                        },
                    );
                }
                self.visitor_destination = Some(destination);
                self.visiting_days = self
                    .visiting_days
                    .checked_add(duration)
                    .ok_or(TemporaryResourceAccountingError::DurationOverflow)?;
            }
            HouseholdPresence::ReturnTransit { .. } => {
                self.return_transit_days = self
                    .return_transit_days
                    .checked_add(duration)
                    .ok_or(TemporaryResourceAccountingError::DurationOverflow)?;
            }
        }
        Ok(())
    }
}

/// Completed, immutable M9.5 presence-duration input for one M3 resource period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryResourcePeriod {
    pub schema_version: u32,
    pub start_day: u64,
    pub end_day: u64,
    pub households: Vec<TemporaryResourcePresenceDays>,
}

impl TemporaryResourcePeriod {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn duration_days(&self) -> Result<u64, TemporaryResourceAccountingError> {
        self.end_day
            .checked_sub(self.start_day)
            .filter(|duration| *duration > 0)
            .ok_or(TemporaryResourceAccountingError::InvalidPeriodBounds {
                start_day: self.start_day,
                end_day: self.end_day,
            })
    }

    pub fn validate(
        &self,
        household_count: usize,
        world: &World,
    ) -> Result<(), TemporaryResourceAccountingError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryResourceAccountingError::UnsupportedPeriodSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.households.len() != household_count {
            return Err(TemporaryResourceAccountingError::HouseholdCountMismatch {
                ledger: self.households.len(),
                expected: household_count,
            });
        }
        let duration = self.duration_days()?;
        for (index, entry) in self.households.iter().enumerate() {
            let actual = entry.total_days()?;
            if actual != duration {
                return Err(
                    TemporaryResourceAccountingError::HouseholdDurationMismatch {
                        household_index: index,
                        expected: duration,
                        actual,
                    },
                );
            }
            validate_destination(index, entry, world)?;
        }
        Ok(())
    }
}

/// In-progress authoritative M9.5 duration ledger embedded in temporary-mobility state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TemporaryResourceLedger {
    schema_version: u32,
    period_start_day: u64,
    accounted_until_day: u64,
    households: Vec<TemporaryResourcePresenceDays>,
}

impl TemporaryResourceLedger {
    pub(crate) const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub(crate) fn new(household_count: usize, start_day: u64) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            period_start_day: start_day,
            accounted_until_day: start_day,
            households: vec![TemporaryResourcePresenceDays::default(); household_count],
        }
    }

    pub(crate) fn accrue_until(
        &mut self,
        day: u64,
        presence: &[HouseholdPresence],
    ) -> Result<(), TemporaryResourceAccountingError> {
        if presence.len() != self.households.len() {
            return Err(TemporaryResourceAccountingError::HouseholdCountMismatch {
                ledger: self.households.len(),
                expected: presence.len(),
            });
        }
        let duration = day.checked_sub(self.accounted_until_day).ok_or(
            TemporaryResourceAccountingError::TimeReversed {
                accounted_until_day: self.accounted_until_day,
                requested_day: day,
            },
        )?;
        if duration == 0 {
            return Ok(());
        }
        for (entry, state) in self.households.iter_mut().zip(presence.iter().copied()) {
            entry.accrue(state, duration)?;
        }
        self.accounted_until_day = day;
        Ok(())
    }

    pub(crate) fn snapshot_period(
        &mut self,
        day: u64,
        presence: &[HouseholdPresence],
        world: &World,
    ) -> Result<TemporaryResourcePeriod, TemporaryResourceAccountingError> {
        self.accrue_until(day, presence)?;
        let period = TemporaryResourcePeriod {
            schema_version: TemporaryResourcePeriod::CURRENT_SCHEMA_VERSION,
            start_day: self.period_start_day,
            end_day: day,
            households: self.households.clone(),
        };
        period.validate(self.households.len(), world)?;
        Ok(period)
    }

    pub(crate) fn reset_after_settlement(
        &mut self,
        day: u64,
    ) -> Result<(), TemporaryResourceAccountingError> {
        if self.accounted_until_day != day {
            return Err(
                TemporaryResourceAccountingError::ResetBeforeSettlementBoundary {
                    accounted_until_day: self.accounted_until_day,
                    settlement_day: day,
                },
            );
        }
        self.period_start_day = day;
        self.households
            .fill(TemporaryResourcePresenceDays::default());
        Ok(())
    }

    pub(crate) fn validate(
        &self,
        household_count: usize,
        world: &World,
        current_day: u64,
    ) -> Result<(), TemporaryResourceAccountingError> {
        if self.schema_version != Self::CURRENT_SCHEMA_VERSION {
            return Err(TemporaryResourceAccountingError::UnsupportedLedgerSchema {
                found: self.schema_version,
                supported: Self::CURRENT_SCHEMA_VERSION,
            });
        }
        if self.households.len() != household_count {
            return Err(TemporaryResourceAccountingError::HouseholdCountMismatch {
                ledger: self.households.len(),
                expected: household_count,
            });
        }
        if self.period_start_day > self.accounted_until_day
            || self.accounted_until_day != current_day
        {
            return Err(TemporaryResourceAccountingError::LedgerTimeMismatch {
                period_start_day: self.period_start_day,
                accounted_until_day: self.accounted_until_day,
                current_day,
            });
        }
        let elapsed = self.accounted_until_day - self.period_start_day;
        for (index, entry) in self.households.iter().enumerate() {
            let actual = entry.total_days()?;
            if actual != elapsed {
                return Err(
                    TemporaryResourceAccountingError::HouseholdDurationMismatch {
                        household_index: index,
                        expected: elapsed,
                        actual,
                    },
                );
            }
            validate_destination(index, entry, world)?;
        }
        Ok(())
    }

    pub(crate) fn digest_into(&self, hash: &mut u64) {
        digest_u64(hash, u64::from(self.schema_version));
        digest_u64(hash, self.period_start_day);
        digest_u64(hash, self.accounted_until_day);
        digest_u64(hash, self.households.len() as u64);
        for entry in &self.households {
            digest_u64(hash, entry.at_residence_days);
            digest_u64(hash, entry.outbound_transit_days);
            digest_u64(hash, entry.visiting_days);
            digest_u64(hash, entry.return_transit_days);
            match entry.visitor_destination {
                None => digest_u64(hash, 0),
                Some(destination) => {
                    digest_u64(hash, 1);
                    digest_u64(hash, destination.0);
                }
            }
        }
    }
}

fn validate_destination(
    household_index: usize,
    entry: &TemporaryResourcePresenceDays,
    world: &World,
) -> Result<(), TemporaryResourceAccountingError> {
    match (entry.visiting_days, entry.visitor_destination) {
        (0, None) => Ok(()),
        (0, Some(destination)) => Err(
            TemporaryResourceAccountingError::UnexpectedVisitorDestination {
                household_index,
                destination,
            },
        ),
        (_, None) => {
            Err(TemporaryResourceAccountingError::MissingVisitorDestination { household_index })
        }
        (_, Some(destination)) if world.cell(destination).is_none() => Err(
            TemporaryResourceAccountingError::VisitorDestinationOutsideWorld {
                household_index,
                destination,
            },
        ),
        (_, Some(_)) => Ok(()),
    }
}

fn digest_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = (*hash).wrapping_mul(FNV_PRIME);
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TemporaryResourceAccountingError {
    #[error(
        "temporary resource ledger schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedLedgerSchema { found: u32, supported: u32 },
    #[error(
        "temporary resource period schema {found} is unsupported; supported schema is {supported}"
    )]
    UnsupportedPeriodSchema { found: u32, supported: u32 },
    #[error("temporary resource ledger has {ledger} households but expected {expected}")]
    HouseholdCountMismatch { ledger: usize, expected: usize },
    #[error("temporary resource accounting duration overflowed")]
    DurationOverflow,
    #[error(
        "temporary resource ledger cannot move backward from day {accounted_until_day} to {requested_day}"
    )]
    TimeReversed {
        accounted_until_day: u64,
        requested_day: u64,
    },
    #[error(
        "temporary resource ledger time is inconsistent: start {period_start_day}, accounted {accounted_until_day}, current {current_day}"
    )]
    LedgerTimeMismatch {
        period_start_day: u64,
        accounted_until_day: u64,
        current_day: u64,
    },
    #[error("temporary resource period bounds are invalid: {start_day}..{end_day}")]
    InvalidPeriodBounds { start_day: u64, end_day: u64 },
    #[error(
        "temporary resource household {household_index} has {actual} accounted days; expected {expected}"
    )]
    HouseholdDurationMismatch {
        household_index: usize,
        expected: u64,
        actual: u64,
    },
    #[error(
        "temporary resource visitor destination changed within one period: {previous:?} -> {next:?}"
    )]
    VisitorDestinationChanged { previous: CellId, next: CellId },
    #[error(
        "temporary resource household {household_index} has visiting days without a destination"
    )]
    MissingVisitorDestination { household_index: usize },
    #[error(
        "temporary resource household {household_index} has destination {destination:?} but no visiting days"
    )]
    UnexpectedVisitorDestination {
        household_index: usize,
        destination: CellId,
    },
    #[error(
        "temporary resource household {household_index} visitor destination {destination:?} is outside the world"
    )]
    VisitorDestinationOutsideWorld {
        household_index: usize,
        destination: CellId,
    },
    #[error(
        "temporary resource ledger cannot reset at day {settlement_day}; it is accounted only through {accounted_until_day}"
    )]
    ResetBeforeSettlementBoundary {
        accounted_until_day: u64,
        settlement_day: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::TemporaryJourneyId;

    #[test]
    fn ledger_counts_half_open_presence_intervals_without_daily_iteration() {
        let mut ledger = TemporaryResourceLedger::new(1, 0);
        ledger
            .accrue_until(10, &[HouseholdPresence::AtResidence])
            .unwrap();
        ledger
            .accrue_until(
                12,
                &[HouseholdPresence::OutboundTransit {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();
        ledger
            .accrue_until(
                17,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();
        ledger
            .accrue_until(
                20,
                &[HouseholdPresence::ReturnTransit {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();

        let entry = ledger.households[0];
        assert_eq!(entry.at_residence_days, 10);
        assert_eq!(entry.outbound_transit_days, 2);
        assert_eq!(entry.visiting_days, 5);
        assert_eq!(entry.return_transit_days, 3);
        assert_eq!(entry.home_provisioning_days().unwrap(), 15);
        assert_eq!(entry.total_days().unwrap(), 20);
        assert_eq!(entry.visitor_destination, Some(CellId::new(2)));
    }

    #[test]
    fn visitor_destination_cannot_change_inside_one_resource_period() {
        let mut ledger = TemporaryResourceLedger::new(1, 0);
        ledger
            .accrue_until(
                1,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(1),
                    destination: CellId::new(2),
                }],
            )
            .unwrap();
        let error = ledger
            .accrue_until(
                2,
                &[HouseholdPresence::Visiting {
                    journey: TemporaryJourneyId::new(2),
                    destination: CellId::new(3),
                }],
            )
            .unwrap_err();
        assert!(matches!(
            error,
            TemporaryResourceAccountingError::VisitorDestinationChanged { .. }
        ));
    }
}
