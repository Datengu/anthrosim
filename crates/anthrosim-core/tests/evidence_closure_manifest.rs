use anthrosim_core::{EvidenceClosureStatus, ExperimentConfig, ParameterProvenance, Simulation};

#[test]
fn completed_run_preserves_synthetic_evidence_closure_status() {
    let run = Simulation::new(ExperimentConfig::default())
        .expect("synthetic simulation should initialize")
        .run_recorded()
        .expect("synthetic simulation should complete");

    assert_eq!(
        run.manifest.evidence_closure.status,
        EvidenceClosureStatus::NotApplicableSynthetic
    );
    run.validate_invariants()
        .expect("untampered recorded run should validate");
}

#[test]
fn completed_run_preserves_not_closed_status_for_unresolved_assumption() {
    let mut experiment = ExperimentConfig::default();
    experiment.resources.provenance = ParameterProvenance::Unresolved;

    let run = Simulation::new(experiment)
        .expect("unresolved exploratory simulation should remain executable")
        .run_recorded()
        .expect("unresolved exploratory simulation should complete");

    assert_eq!(
        run.manifest.evidence_closure.status,
        EvidenceClosureStatus::NotClosed
    );
    assert!(!run.manifest.evidence_closure.failures.is_empty());
    run.validate_invariants()
        .expect("preserved unresolved readiness should still reconcile");
}

#[test]
fn recorded_run_rejects_tampered_evidence_closure_label() {
    let mut run = Simulation::new(ExperimentConfig::default())
        .expect("synthetic simulation should initialize")
        .run_recorded()
        .expect("synthetic simulation should complete");

    run.manifest.evidence_closure.status = EvidenceClosureStatus::Closed;

    assert!(
        run.validate_invariants().is_err(),
        "a saved readiness label must not be independently forgeable"
    );
}
