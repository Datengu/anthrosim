use anthrosim_core::{ExperimentConfig, ResumeLineage, Simulation};

#[test]
fn year_zero_checkpoint_resume_preserves_exact_authoritative_output() {
    let config = ExperimentConfig::new(91_337, 2);
    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();

    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(0)
        .unwrap();
    let resumed = Simulation::from_checkpoint(checkpoint)
        .unwrap()
        .run_recorded()
        .unwrap();

    assert_eq!(
        resumed.manifest.state_digest64,
        uninterrupted.manifest.state_digest64
    );

    let uninterrupted_days: Vec<_> = uninterrupted
        .metrics()
        .snapshots
        .iter()
        .map(|snapshot| snapshot.day)
        .collect();
    let resumed_days: Vec<_> = resumed
        .metrics()
        .snapshots
        .iter()
        .map(|snapshot| snapshot.day)
        .collect();
    assert_eq!(resumed_days, uninterrupted_days);

    let mut resumed_checkpoint = resumed.checkpoint.clone();
    resumed_checkpoint.resume_lineage = ResumeLineage::new();
    resumed_checkpoint = resumed_checkpoint.seal_continuation_identity();
    assert_eq!(resumed_checkpoint, uninterrupted.checkpoint);
}
