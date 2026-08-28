from pathlib import Path

root = Path(__file__).resolve().parents[1]
path = root / "crates/anthrosim-core/examples/household_lifecycle_sensitivity.rs"
text = path.read_text(encoding="utf-8")
old = '''            let resources = run.checkpoint.resources.summary(&run.checkpoint.population);\n            aggregate.unmet_need_total += resources.unmet_need;\n            let migration = anthrosim_core::MigrationSystem::from_checkpoint_state(\n                &run.checkpoint.population,\n                &anthrosim_core::World::generate(\n                    run.checkpoint.experiment.world,\n                    anthrosim_core::rng::RngFactory::new(run.checkpoint.experiment.seed),\n                )\n                .unwrap(),\n                &run.checkpoint.experiment.migration,\n                run.checkpoint.migration.clone(),\n            )\n            .unwrap()\n            .summary();\n            aggregate.migration_moves_total += migration.moves_completed;\n            aggregate.migration_people_moved_total += migration.people_moved;\n'''
new = '''            aggregate.unmet_need_total += run.manifest.resources.unmet_need;\n            aggregate.migration_moves_total += run.manifest.migration.moves_completed;\n            aggregate.migration_people_moved_total += run.manifest.migration.people_moved;\n'''
if old not in text:
    raise SystemExit("example migration-summary anchor not found")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
