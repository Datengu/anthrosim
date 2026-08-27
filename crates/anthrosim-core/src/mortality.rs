use rand::Rng;
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::{config::PROBABILITY_PER_MILLION, time::DAYS_PER_YEAR};

/// Exact conditional probability represented as an unreduced non-negative rational.
///
/// Keeping authoritative mortality arithmetic rational avoids cross-platform floating-point
/// drift while preserving exact survival-equivalent interval conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProbabilityFraction {
    pub numerator: u128,
    pub denominator: u128,
}

impl ProbabilityFraction {
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    #[cfg(test)]
    pub(crate) fn from_per_million(probability: u32) -> Result<Self, MortalityMathError> {
        if probability > PROBABILITY_PER_MILLION {
            return Err(MortalityMathError::ProbabilityOutOfRange { probability });
        }
        Ok(Self {
            numerator: u128::from(probability),
            denominator: u128::from(PROBABILITY_PER_MILLION),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompetingMortalityCause {
    ConditionMediated,
    Background,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MortalityMathError {
    #[error("mortality probability {probability} exceeds one million")]
    ProbabilityOutOfRange { probability: u32 },
    #[error("mortality interval {start}..{end} lies outside one 365-day model year")]
    InvalidAnnualInterval { start: u64, end: u64 },
    #[error("mortality probability fraction has a zero denominator")]
    ZeroDenominator,
    #[error("mortality probability arithmetic overflowed")]
    ArithmeticOverflow,
}

/// Convert one annual background-mortality probability into the exact conditional risk over an
/// elapsed sub-interval of the same model year.
///
/// The annual probability is interpreted as cumulative incidence over `[0, 365)`. Cumulative
/// survival is linear in elapsed model days, `S(t) = 1 - p*t/365`, so the conditional interval
/// risk is `1 - S(end)/S(start)`. Multiplying survival across any complete partition of the year
/// therefore recovers exactly `1 - p`; adding more M3 boundaries does not multiply the annual M2
/// risk. This is a discrete elapsed-time contract, not a continuous-time exponential hazard.
pub(crate) fn annual_probability_for_interval(
    annual_probability: u32,
    start: u64,
    end: u64,
) -> Result<ProbabilityFraction, MortalityMathError> {
    if annual_probability > PROBABILITY_PER_MILLION {
        return Err(MortalityMathError::ProbabilityOutOfRange {
            probability: annual_probability,
        });
    }
    if start > end || end > DAYS_PER_YEAR {
        return Err(MortalityMathError::InvalidAnnualInterval { start, end });
    }
    if start == end || annual_probability == 0 {
        return Ok(ProbabilityFraction::ZERO);
    }

    let probability = u128::from(annual_probability);
    let scale = u128::from(PROBABILITY_PER_MILLION);
    let year = u128::from(DAYS_PER_YEAR);
    let numerator = probability
        .checked_mul(u128::from(end - start))
        .ok_or(MortalityMathError::ArithmeticOverflow)?;
    let denominator = scale
        .checked_mul(year)
        .and_then(|value| value.checked_sub(probability.checked_mul(u128::from(start))?))
        .ok_or(MortalityMathError::ArithmeticOverflow)?;
    if denominator == 0 {
        return Err(MortalityMathError::ZeroDenominator);
    }
    Ok(ProbabilityFraction {
        numerator,
        denominator,
    })
}

pub(crate) fn probability_fraction_per_million_ceil(
    probability: ProbabilityFraction,
) -> Result<u32, MortalityMathError> {
    if probability.denominator == 0 {
        return Err(MortalityMathError::ZeroDenominator);
    }
    if probability.numerator == 0 {
        return Ok(0);
    }
    let scaled = probability
        .numerator
        .checked_mul(u128::from(PROBABILITY_PER_MILLION))
        .ok_or(MortalityMathError::ArithmeticOverflow)?
        .div_ceil(probability.denominator)
        .min(u128::from(PROBABILITY_PER_MILLION));
    u32::try_from(scaled).map_err(|_| MortalityMathError::ArithmeticOverflow)
}

pub(crate) fn draw_probability_fraction<R: Rng + ?Sized>(
    rng: &mut R,
    probability: ProbabilityFraction,
) -> Result<bool, MortalityMathError> {
    if probability.denominator == 0 {
        return Err(MortalityMathError::ZeroDenominator);
    }
    if probability.numerator == 0 {
        return Ok(false);
    }
    if probability.numerator >= probability.denominator {
        return Ok(true);
    }
    Ok(draw_bounded_u128(rng, probability.denominator) < probability.numerator)
}

/// Resolve two explicit cause-specific interval risks without scheduler priority.
///
/// Each cause receives its own independent latent trigger on its pre-existing deterministic RNG
/// stream. Neither trigger is allowed to suppress the other. Survival is therefore exactly the
/// product `(1 - q_condition) * (1 - q_background)`, regardless of which cause is evaluated first.
/// If exactly one trigger fires, that cause is authoritative. If both fire, attribution is resolved
/// symmetrically in proportion to the two cause-specific interval risks. The tie draw combines one
/// draw from each stream with XOR, so exchanging the two cause labels/streams exchanges the
/// attribution but cannot create a first-called advantage.
pub(crate) fn resolve_two_cause_competing_mortality(
    condition_probability: ProbabilityFraction,
    background_probability: ProbabilityFraction,
    condition_rng: &mut ChaCha8Rng,
    background_rng: &mut ChaCha8Rng,
) -> Result<Option<CompetingMortalityCause>, MortalityMathError> {
    let condition_trigger = draw_probability_fraction(condition_rng, condition_probability)?;
    let background_trigger = draw_probability_fraction(background_rng, background_probability)?;

    match (condition_trigger, background_trigger) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(CompetingMortalityCause::ConditionMediated)),
        (false, true) => Ok(Some(CompetingMortalityCause::Background)),
        (true, true) => {
            let condition_weight = u64::from(probability_fraction_per_million_ceil(
                condition_probability,
            )?);
            let background_weight = u64::from(probability_fraction_per_million_ceil(
                background_probability,
            )?);
            let total_weight = condition_weight
                .checked_add(background_weight)
                .ok_or(MortalityMathError::ArithmeticOverflow)?;
            if total_weight == 0 {
                return Err(MortalityMathError::ZeroDenominator);
            }
            let draw = draw_symmetric_bounded(condition_rng, background_rng, total_weight);
            if draw < condition_weight {
                Ok(Some(CompetingMortalityCause::ConditionMediated))
            } else {
                Ok(Some(CompetingMortalityCause::Background))
            }
        }
    }
}

fn draw_bounded_u128<R: Rng + ?Sized>(rng: &mut R, upper_exclusive: u128) -> u128 {
    debug_assert!(upper_exclusive > 0);
    let acceptance_limit = u128::MAX - (u128::MAX % upper_exclusive);
    loop {
        let draw = (u128::from(rng.next_u64()) << 64) | u128::from(rng.next_u64());
        if draw < acceptance_limit {
            return draw % upper_exclusive;
        }
    }
}

fn draw_symmetric_bounded(
    left: &mut ChaCha8Rng,
    right: &mut ChaCha8Rng,
    upper_exclusive: u64,
) -> u64 {
    debug_assert!(upper_exclusive > 0);
    let acceptance_limit = u64::MAX - (u64::MAX % upper_exclusive);
    loop {
        // XOR is commutative and preserves a uniform word when the two named streams are
        // independent. The tie allocator therefore has no left/right or call-order preference.
        let draw = left.next_u64() ^ right.next_u64();
        if draw < acceptance_limit {
            return draw % upper_exclusive;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn partition_bounds(periods: u64, index: u64) -> (u64, u64) {
        (
            index * DAYS_PER_YEAR / periods,
            (index + 1) * DAYS_PER_YEAR / periods,
        )
    }

    #[test]
    fn annual_risk_partition_preserves_complete_year_survival() {
        for annual_probability in [0, 1, 50_000, 200_000, 999_999, 1_000_000] {
            for periods in [1_u64, 4, 12, 365] {
                let mut survival_numerator = 1_u128;
                let mut survival_denominator = 1_u128;
                for index in 0..periods {
                    let (start, end) = partition_bounds(periods, index);
                    let interval =
                        annual_probability_for_interval(annual_probability, start, end).unwrap();
                    survival_numerator = survival_numerator
                        .checked_mul(interval.denominator - interval.numerator)
                        .unwrap();
                    survival_denominator = survival_denominator
                        .checked_mul(interval.denominator)
                        .unwrap();
                    let gcd = gcd_u128(survival_numerator, survival_denominator);
                    survival_numerator /= gcd;
                    survival_denominator /= gcd;
                }
                assert_eq!(
                    survival_numerator * u128::from(PROBABILITY_PER_MILLION),
                    survival_denominator * u128::from(PROBABILITY_PER_MILLION - annual_probability),
                    "annual={annual_probability}, periods={periods}"
                );
            }
        }
    }

    #[test]
    fn single_cause_edges_are_exact() {
        let zero = ProbabilityFraction::from_per_million(0).unwrap();
        let certain = ProbabilityFraction::from_per_million(1_000_000).unwrap();
        let mut condition_rng = ChaCha8Rng::seed_from_u64(1);
        let mut background_rng = ChaCha8Rng::seed_from_u64(2);
        assert_eq!(
            resolve_two_cause_competing_mortality(
                certain,
                zero,
                &mut condition_rng,
                &mut background_rng
            )
            .unwrap(),
            Some(CompetingMortalityCause::ConditionMediated)
        );

        let mut condition_rng = ChaCha8Rng::seed_from_u64(1);
        let mut background_rng = ChaCha8Rng::seed_from_u64(2);
        assert_eq!(
            resolve_two_cause_competing_mortality(
                zero,
                certain,
                &mut condition_rng,
                &mut background_rng
            )
            .unwrap(),
            Some(CompetingMortalityCause::Background)
        );
    }

    #[test]
    fn exchanging_causes_and_streams_exchanges_attribution_exactly() {
        let condition = ProbabilityFraction::from_per_million(350_000).unwrap();
        let background = ProbabilityFraction::from_per_million(600_000).unwrap();
        let mut left_condition = ChaCha8Rng::seed_from_u64(91);
        let mut left_background = ChaCha8Rng::seed_from_u64(177);
        let mut right_condition = ChaCha8Rng::seed_from_u64(177);
        let mut right_background = ChaCha8Rng::seed_from_u64(91);

        for _ in 0..10_000 {
            let left = resolve_two_cause_competing_mortality(
                condition,
                background,
                &mut left_condition,
                &mut left_background,
            )
            .unwrap();
            let right = resolve_two_cause_competing_mortality(
                background,
                condition,
                &mut right_condition,
                &mut right_background,
            )
            .unwrap();
            assert_eq!(left.map(swap), right);
        }
    }

    #[test]
    fn controlled_frequencies_match_independent_union_and_risk_weighted_dual_trigger() {
        let condition = ProbabilityFraction::from_per_million(200_000).unwrap();
        let background = ProbabilityFraction::from_per_million(300_000).unwrap();
        let mut condition_rng = ChaCha8Rng::seed_from_u64(20801);
        let mut background_rng = ChaCha8Rng::seed_from_u64(20802);
        let trials = 100_000_u64;
        let mut survived = 0_u64;
        let mut condition_deaths = 0_u64;
        let mut background_deaths = 0_u64;

        for _ in 0..trials {
            match resolve_two_cause_competing_mortality(
                condition,
                background,
                &mut condition_rng,
                &mut background_rng,
            )
            .unwrap()
            {
                None => survived += 1,
                Some(CompetingMortalityCause::ConditionMediated) => condition_deaths += 1,
                Some(CompetingMortalityCause::Background) => background_deaths += 1,
            }
        }

        // q_total = 1 - (1 - .2)(1 - .3) = .44.
        // Exactly-one trigger contributions are .14 condition and .24 background. The .06 dual
        // trigger mass is split 2:3, yielding expectations .164 and .276 respectively.
        assert_within(condition_deaths, 16_400, 900);
        assert_within(background_deaths, 27_600, 900);
        assert_within(trials - survived, 44_000, 1_000);
        assert_eq!(survived + condition_deaths + background_deaths, trials);
    }

    fn swap(cause: CompetingMortalityCause) -> CompetingMortalityCause {
        match cause {
            CompetingMortalityCause::ConditionMediated => CompetingMortalityCause::Background,
            CompetingMortalityCause::Background => CompetingMortalityCause::ConditionMediated,
        }
    }

    fn assert_within(actual: u64, expected: u64, tolerance: u64) {
        assert!(
            actual.abs_diff(expected) <= tolerance,
            "actual={actual}, expected={expected}, tolerance={tolerance}"
        );
    }

    fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
        while right != 0 {
            let remainder = left % right;
            left = right;
            right = remainder;
        }
        left.max(1)
    }
}
