use anthrosim_core::{ExperimentConfig, ResourcePeriodObservation, Simulation};

#[test]
fn resource_period_history_is_preserved_and_reconciles() {
    let mut config = ExperimentConfig::new(21501, 2);
    config.world.width = 3;
    config.world.height = 2;
    config.population.initial_population = 24;
    config.resources.annual_need_units_per_person = 220;
    let checkpoint = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(2)
        .unwrap();
    let periods = checkpoint.resources.period_observations();
    assert_eq!(
        periods.len() as u64,
        checkpoint
            .resources
            .summary(&checkpoint.population)
            .periods_processed
    );
    assert!(
        checkpoint
            .resources
            .period_observation_history_complete_from_start()
    );
    for period in periods {
        assert_eq!(
            period.schema_version,
            ResourcePeriodObservation::CURRENT_SCHEMA_VERSION
        );
        assert_eq!(period.total_need, period.supplied + period.unmet);
        assert_eq!(
            period.stock_before_regeneration + period.regenerated - period.supplied,
            period.stock_after_harvest
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.total_need).sum::<u64>(),
            period.total_need
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.supplied).sum::<u64>(),
            period.supplied
        );
        assert_eq!(
            period.cells.iter().map(|cell| cell.unmet).sum::<u64>(),
            period.unmet
        );
    }
}

#[test]
fn checkpoint_resume_preserves_exact_period_history() {
    let mut config = ExperimentConfig::new(21502, 3);
    config.world.width = 2;
    config.world.height = 2;
    config.population.initial_population = 20;
    let uninterrupted = Simulation::new(config.clone())
        .unwrap()
        .run_recorded()
        .unwrap();
    let boundary = Simulation::new(config)
        .unwrap()
        .checkpoint_at_year(1)
        .unwrap();
    let resumed = Simulation::from_checkpoint(boundary)
        .unwrap()
        .run_recorded()
        .unwrap();
    assert_eq!(
        uninterrupted.checkpoint.resources.period_observations(),
        resumed.checkpoint.resources.period_observations()
    );
}

#[test]
fn period_history_distinguishes_temporal_shapes_with_same_terminal_totals() {
    let mut config = ExperimentConfig::new(21503, 1);
    config.world.width = 1;
    config.world.height = 1;
    config.population.initial_population = 5;
    let checkpoint = Simulation::new(config)
        .unwrap()
        .run_recorded()
        .unwrap()
        .checkpoint;
    let template = checkpoint.resources.period_observations()[0].clone();
    let mut chronic = vec![
        template.clone(),
        template.clone(),
        template.clone(),
        template.clone(),
    ];
    let mut acute = chronic.clone();
    for (index, period) in chronic.iter_mut().enumerate() {
        period.sequence = index as u64 + 1;
        period.unmet = 25;
    }
    for (index, period) in acute.iter_mut().enumerate() {
        period.sequence = index as u64 + 1;
        period.unmet = if index == 0 { 100 } else { 0 };
    }
    assert_eq!(
        chronic.iter().map(|period| period.unmet).sum::<u64>(),
        acute.iter().map(|period| period.unmet).sum::<u64>()
    );
    assert_ne!(
        chronic
            .iter()
            .map(|period| period.unmet)
            .collect::<Vec<_>>(),
        acute.iter().map(|period| period.unmet).collect::<Vec<_>>()
    );
}
