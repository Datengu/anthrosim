from pathlib import Path


def replace(path: str, old: str, new: str, count: int = 1) -> None:
    p = Path(path)
    text = p.read_text(encoding="utf-8")
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:160]!r}")
    p.write_text(text.replace(old, new, count), encoding="utf-8")


simulation = "crates/anthrosim-core/src/simulation.rs"

# Preserve the existing specific validation diagnostics, but still require the
# complete continuation identity before any RNG position is restored or any
# resumed execution can begin.
replace(
    simulation,
    '''        let actual_continuation_digest64 = compute_continuation_digest64(&checkpoint);
        if checkpoint.continuation_digest64 != actual_continuation_digest64 {
            return Err(SimulationError::CheckpointContinuationDigestMismatch {
                expected: checkpoint.continuation_digest64,
                actual: actual_continuation_digest64,
            });
        }
''',
    "",
)

replace(
    simulation,
    '''        checkpoint
            .resources
            .validate_checkpoint_state(&world, &checkpoint.experiment.resources)?;
        validate_terminal_checkpoint_state(&checkpoint)?;
        checkpoint
            .validate_invariants()
            .map_err(|error| SimulationError::CheckpointInvariantViolation {
                reason: error.to_string(),
            })?;

        let boundary_day = checkpoint.time.days();
''',
    '''        checkpoint
            .resources
            .validate_checkpoint_state(&world, &checkpoint.experiment.resources)?;
        let migration = MigrationSystem::from_checkpoint_state(
            &checkpoint.population,
            &world,
            &checkpoint.experiment.migration,
            checkpoint.migration.clone(),
        )?;
        validate_terminal_checkpoint_state(&checkpoint)?;

        let actual_state_digest64 = state_digest64_with_temporary_mobility(
            checkpoint.time.days(),
            world.digest64(),
            checkpoint.population.digest64(),
            checkpoint.resources.digest64(),
            migration.digest64(),
            &checkpoint.temporary_mobility,
        );
        if actual_state_digest64 != checkpoint.state_digest64 {
            return Err(SimulationError::CheckpointStateDigestMismatch {
                expected: checkpoint.state_digest64,
                actual: actual_state_digest64,
            });
        }

        let actual_continuation_digest64 = compute_continuation_digest64(&checkpoint);
        if checkpoint.continuation_digest64 != actual_continuation_digest64 {
            return Err(SimulationError::CheckpointContinuationDigestMismatch {
                expected: checkpoint.continuation_digest64,
                actual: actual_continuation_digest64,
            });
        }
        checkpoint
            .validate_invariants()
            .map_err(|error| SimulationError::CheckpointInvariantViolation {
                reason: error.to_string(),
            })?;

        let boundary_day = checkpoint.time.days();
''',
)

replace(
    simulation,
    '''        let migration = MigrationSystem::from_checkpoint_state(
            &checkpoint.population,
            &world,
            &checkpoint.experiment.migration,
            checkpoint.migration,
        )?;

        let mut demography_rngs = DemographyRngs::new(rng_factory);
''',
    '''        let mut demography_rngs = DemographyRngs::new(rng_factory);
''',
)

replace(
    simulation,
    '''        let actual_digest = simulation.state_digest64();
        if actual_digest != source_state_digest64 {
            return Err(SimulationError::CheckpointStateDigestMismatch {
                expected: source_state_digest64,
                actual: actual_digest,
            });
        }
        Ok(simulation)
''',
    '''        Ok(simulation)
''',
)

# A source-revision string remains compatibility-neutral, but it is now part of
# the provenance/output identity. Unauthorised mutation must fail integrity;
# an explicitly re-integritied artifact may still resume across source-neutral
# revisions.
replace(
    simulation,
    '''        checkpoint.git_commit = Some("source-neutral-revision-test".to_owned());

        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();
''',
    '''        checkpoint.git_commit = Some("source-neutral-revision-test".to_owned());
        assert!(matches!(
            Simulation::from_checkpoint(checkpoint.clone()),
            Err(SimulationError::CheckpointContinuationDigestMismatch { .. })
        ));
        checkpoint.refresh_continuation_digest64();

        let resumed = Simulation::from_checkpoint(checkpoint)
            .unwrap()
            .run_recorded()
            .unwrap();
''',
)

# The M9 resume-equivalence test intentionally normalises away provenance
# lineage. Recompute the new complete identity after that deliberate transform.
replace(
    "crates/anthrosim-core/src/m9_integration_tests.rs",
    '''    let mut resumed_checkpoint = resumed.checkpoint.clone();
    resumed_checkpoint.resume_lineage = ResumeLineage::new();
    assert_eq!(resumed_checkpoint, uninterrupted.checkpoint);
''',
    '''    let mut resumed_checkpoint = resumed.checkpoint.clone();
    resumed_checkpoint.resume_lineage = ResumeLineage::new();
    resumed_checkpoint.refresh_continuation_digest64();
    assert_eq!(resumed_checkpoint, uninterrupted.checkpoint);
''',
)

# Direct numeric boundary coverage for #176. The checked helper is deliberately
# private to ResourceSystem; child test modules can exercise the exact aggregate
# boundary without constructing an impossible normal runtime.
resources = Path("crates/anthrosim-core/src/resources.rs")
with resources.open("a", encoding="utf-8") as handle:
    handle.write(r'''

#[cfg(test)]
mod final_audit_overflow_tests {
    use super::*;

    fn system_with_stock(cell_food_stock: Vec<u64>) -> ResourceSystem {
        ResourceSystem {
            schema_version: ResourceSystem::CURRENT_SCHEMA_VERSION,
            model_id: "overflow-audit".to_owned(),
            initial_world_digest64: "0000000000000000".to_owned(),
            cell_food_stock,
            initial_food_stock: 0,
            regenerated_food: 0,
            harvested_food: 0,
            unmet_need: 0,
            periods_processed: 0,
            household_periods_with_unmet_need: 0,
            scarcity_deaths: 0,
        }
    }

    #[test]
    fn aggregate_stock_accepts_exact_u64_boundary() {
        let system = system_with_stock(vec![u64::MAX - 1, 1]);
        assert_eq!(system.checked_total_food_stock(), Ok(u64::MAX));
    }

    #[test]
    fn aggregate_stock_overflow_fails_closed() {
        let system = system_with_stock(vec![u64::MAX, 1]);
        assert_eq!(
            system.checked_total_food_stock(),
            Err(ResourceError::AccountingOverflow)
        );
    }
}
''')

# Add the missing death-history mutation counterpart for #178.
integrity_tests = Path("crates/anthrosim-core/tests/final_audit_integrity.rs")
with integrity_tests.open("a", encoding="utf-8") as handle:
    handle.write(r'''

#[test]
fn duplicate_death_history_is_rejected_after_rehash() {
    let mut demography = anthrosim_core::DemographyConfig::synthetic_validation_v1();
    for band in &mut demography.mortality_bands {
        band.annual_probability_per_million = 1_000_000;
    }
    for band in &mut demography.fertility_bands {
        band.annual_probability_per_million = 0;
    }

    let checkpoint = Simulation::new(
        ExperimentConfig::new(89, 2)
            .with_world(WorldConfig::new(1, 1))
            .with_population(PopulationConfig::new(100).with_max_person_records(10_000))
            .with_demography(demography),
    )
    .unwrap()
    .checkpoint_at_year(1)
    .unwrap();

    let mut json = serde_json::to_value(&checkpoint).unwrap();
    let events = json["events"]["events"].as_array_mut().unwrap();
    let death_indices: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| {
            (event["event"]["type"] == "death").then_some(index)
        })
        .collect();
    assert!(death_indices.len() >= 2);
    let first_event = events[death_indices[0]]["event"].clone();
    events[death_indices[1]]["event"] = first_event;
    let mut duplicate_death: SimulationCheckpoint = serde_json::from_value(json).unwrap();
    duplicate_death.refresh_continuation_digest64();
    assert!(duplicate_death.validate_invariants().is_err());
}
''')
