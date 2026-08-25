use serde::{Deserialize, Serialize};

pub const DAYS_PER_YEAR: u64 = 365;
/// Largest whole-year duration for which epoch-relative signed birth-day arithmetic remains
/// representable even for the maximum configured synthetic founder age (`u16::MAX` years).
pub const MAX_SUPPORTED_DURATION_YEARS: u64 =
    (i64::MAX as u64 - u16::MAX as u64 * DAYS_PER_YEAR) / DAYS_PER_YEAR;

/// Integer simulation time. v0.1 intentionally avoids floating-point clocks.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct SimTime {
    days: u64,
}

impl SimTime {
    pub const ZERO: Self = Self { days: 0 };

    #[must_use]
    pub const fn from_days(days: u64) -> Self {
        Self { days }
    }

    #[must_use]
    pub const fn from_years(years: u64) -> Self {
        match years.checked_mul(DAYS_PER_YEAR) {
            Some(days) => Self { days },
            None => panic!("simulation year count exceeds the u64 day representation"),
        }
    }

    #[must_use]
    pub const fn days(self) -> u64 {
        self.days
    }

    #[must_use]
    pub const fn completed_years(self) -> u64 {
        self.days / DAYS_PER_YEAR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn years_round_trip_at_year_boundaries() {
        let time = SimTime::from_years(12_345);
        assert_eq!(time.days(), 12_345 * DAYS_PER_YEAR);
        assert_eq!(time.completed_years(), 12_345);
    }

    #[test]
    fn supported_duration_bound_preserves_signed_founder_age_arithmetic() {
        let terminal_day = SimTime::from_years(MAX_SUPPORTED_DURATION_YEARS).days();
        let oldest_founder_age_days = u64::from(u16::MAX) * DAYS_PER_YEAR;
        assert!(terminal_day <= i64::MAX as u64 - oldest_founder_age_days);
        assert!(
            SimTime::from_years(MAX_SUPPORTED_DURATION_YEARS + 1).days()
                > i64::MAX as u64 - oldest_founder_age_days
        );
    }

    #[test]
    #[should_panic(expected = "simulation year count exceeds the u64 day representation")]
    fn from_years_does_not_silently_saturate_on_overflow() {
        let _ = SimTime::from_years(u64::MAX);
    }
}
