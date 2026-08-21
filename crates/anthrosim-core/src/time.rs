use serde::{Deserialize, Serialize};

pub const DAYS_PER_YEAR: u64 = 365;

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
        Self {
            days: years.saturating_mul(DAYS_PER_YEAR),
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
}
