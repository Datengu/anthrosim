use anthrosim_core::{
    SpatialExtentAdequacyCriterion, SpatialExtentMetricObservation, SpatialExtentMetricTolerance,
    SpatialExtentObservation, assess_spatial_extent_convergence,
};

fn criterion() -> SpatialExtentAdequacyCriterion {
    SpatialExtentAdequacyCriterion {
        schema_version: SpatialExtentAdequacyCriterion::CURRENT_SCHEMA_VERSION,
        criterion_id: "audit-v6-area-i-tail-divergence".to_owned(),
        required_consecutive_stable_extensions: 2,
        minimum_buffer_cells: Some(2),
        metric_tolerances: vec![SpatialExtentMetricTolerance {
            metric_id: "claim_relevant_inner_metric".to_owned(),
            max_absolute_difference: Some(0),
            max_relative_difference_permille: Some(0),
        }],
    }
}

fn observation(buffer_cells: u32, value: u64) -> SpatialExtentObservation {
    SpatialExtentObservation {
        buffer_cells,
        metrics: vec![SpatialExtentMetricObservation {
            metric_id: "claim_relevant_inner_metric".to_owned(),
            value,
        }],
    }
}

#[test]
fn late_extent_divergence_invalidates_an_apparently_converged_prefix() {
    let converged_prefix = assess_spatial_extent_convergence(
        criterion(),
        vec![
            observation(0, 100),
            observation(1, 100),
            observation(2, 100),
            observation(3, 100),
            observation(4, 100),
        ],
    )
    .expect("controlled converged prefix should be valid");

    assert_eq!(converged_prefix.trailing_stable_extensions, 2);
    assert!(converged_prefix.latest_extension_within_tolerance);
    assert!(converged_prefix.latest_extension_eligible_for_stability_sequence);
    assert!(!converged_prefix.material_boundary_dependence_at_latest_extension);
    assert!(converged_prefix.adequate);

    let late_divergence = assess_spatial_extent_convergence(
        criterion(),
        vec![
            observation(0, 100),
            observation(1, 100),
            observation(2, 100),
            observation(3, 100),
            observation(4, 100),
            observation(5, 150),
        ],
    )
    .expect("controlled late-divergence sequence should be valid");

    let latest = late_divergence
        .comparisons
        .last()
        .expect("six observations must produce a latest comparison");
    let metric = latest
        .metrics
        .first()
        .expect("declared metric must be compared");

    println!(
        "prefix_adequate={} prefix_trailing_stable={} late_adequate={} late_trailing_stable={} late_absolute_difference={} late_relative_difference_permille={} material_boundary_dependence={}",
        converged_prefix.adequate,
        converged_prefix.trailing_stable_extensions,
        late_divergence.adequate,
        late_divergence.trailing_stable_extensions,
        metric.absolute_difference,
        metric.relative_difference_permille,
        late_divergence.material_boundary_dependence_at_latest_extension,
    );

    assert_eq!(metric.absolute_difference, 50);
    assert_eq!(metric.relative_difference_permille, 334);
    assert!(!metric.within_tolerance);
    assert!(!late_divergence.latest_extension_within_tolerance);
    assert!(late_divergence.latest_extension_eligible_for_stability_sequence);
    assert!(late_divergence.material_boundary_dependence_at_latest_extension);
    assert_eq!(late_divergence.trailing_stable_extensions, 0);
    assert!(
        !late_divergence.adequate,
        "a late material divergence must invalidate the previously adequate trailing sequence"
    );
}
