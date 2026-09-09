use crate::{
    ids::CellId,
    resources::duration_weighted_needs,
    temporary_resource::TemporaryResourcePresenceDays,
};

fn split_for_household_index(household_index: usize, period_sequence: u64) -> (u64, u64) {
    let presence = TemporaryResourcePresenceDays {
        at_residence_days: 45,
        outbound_transit_days: 0,
        visiting_days: 45,
        return_transit_days: 0,
        visitor_destination: Some(CellId::new(2)),
    };
    duration_weighted_needs(1, &presence, household_index, period_sequence).unwrap()
}

#[test]
fn m9_duration_rounding_tie_is_invariant_to_canonical_household_relabelling() {
    for period_sequence in 0..16_u64 {
        let canonical = split_for_household_index(0, period_sequence);
        let relabelled = split_for_household_index(1, period_sequence);

        println!(
            "duration relabel control: period={period_sequence} canonical={canonical:?} relabelled={relabelled:?}"
        );

        assert_eq!(
            canonical, relabelled,
            "pure canonical HouseholdId/index relabelling must not move an indivisible 50/50 resource-need remainder between home and visitor attribution at period {period_sequence}"
        );
        assert_eq!(canonical.0 + canonical.1, 1);
    }
}
