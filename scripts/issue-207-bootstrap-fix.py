from pathlib import Path

root = Path(__file__).resolve().parents[1]

example_path = root / "crates/anthrosim-core/examples/household_lifecycle_sensitivity.rs"
text = example_path.read_text(encoding="utf-8")
old = '''        let resources = run.checkpoint.resources.summary(&run.checkpoint.population);
        aggregate.unmet_need_total += resources.unmet_need;
        let migration = anthrosim_core::MigrationSystem::from_checkpoint_state(
            &run.checkpoint.population,
            &anthrosim_core::World::generate(
                run.checkpoint.experiment.world,
                anthrosim_core::rng::RngFactory::new(run.checkpoint.experiment.seed),
            )
            .unwrap(),
            &run.checkpoint.experiment.migration,
            run.checkpoint.migration.clone(),
        )
        .unwrap()
        .summary();
        aggregate.migration_moves_total += migration.moves_completed;
        aggregate.migration_people_moved_total += migration.people_moved;
'''
new = '''        aggregate.unmet_need_total += run.manifest.resources.unmet_need;
        aggregate.migration_moves_total += run.manifest.migration.moves_completed;
        aggregate.migration_people_moved_total += run.manifest.migration.people_moved;
'''
if old not in text:
    raise SystemExit("example migration-summary anchor not found")
text = text.replace(old, new, 1)
old = "resources.productivity_scale_permille = 1_200;"
new = "resources.productivity_scale_permille = 1_000;"
if old not in text:
    raise SystemExit("resource productivity scale anchor not found")
example_path.write_text(text.replace(old, new, 1), encoding="utf-8")

observability_path = root / "crates/anthrosim-core/src/household_observability.rs"
text = observability_path.read_text(encoding="utf-8")
old = 'person {person:?} references invalid household {household:?}'
new = 'invalid household identity {household:?}'
if old not in text:
    raise SystemExit("household observability diagnostic anchor not found")
observability_path.write_text(text.replace(old, new, 1), encoding="utf-8")

population_path = root / "crates/anthrosim-core/src/population.rs"
text = population_path.read_text(encoding="utf-8")
old = '''        for household_index in 0..original_household_count {
            if !eligible_households[household_index] {
                continue;
            }
'''
new = '''        for (household_index, &is_eligible) in eligible_households.iter().enumerate() {
            if !is_eligible {
                continue;
            }
'''
if old not in text:
    raise SystemExit("household fission loop anchor not found")
population_path.write_text(text.replace(old, new, 1), encoding="utf-8")
