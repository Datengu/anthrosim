use std::{path::PathBuf, process::ExitCode};

use clap::Parser;

#[path = "../bundle.rs"]
mod bundle;
#[path = "../pack.rs"]
mod pack;

#[derive(Debug, Parser)]
#[command(
    name = "anthrosim-pack",
    version,
    about = "Package a completed AnthroSim run bundle into one deterministic ZIP archive"
)]
struct Cli {
    /// Completed AnthroSim run directory to package.
    run_dir: PathBuf,

    /// Optional archive output path. Defaults to RUN_DIR.zip beside the run directory.
    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match pack::pack_completed_run(&cli.run_dir, cli.output.as_deref()) {
        Ok(path) => {
            println!("wrote run archive {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("anthrosim-pack: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicU64, Ordering},
    };

    use anthrosim_core::{
        ExperimentConfig, GridGeometry, LandscapeBundle, LandscapeLayer, LandscapeLayerRole,
        LandscapeSimulation, LandscapeValueDomain, NoDataPolicy, PopulationConfig, Simulation,
        SpatialFieldTransform, SpatialLandscapeSimulation, SpatialMechanismConfig,
        SpatialTargetField, TransformDirection, WorldConfig,
    };

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn resumed_bundle_packages_successfully() {
        let root = test_dir("resumed");
        let config = base_config(81, 1);
        let checkpoint = Simulation::new(config).unwrap().checkpoint_at_year(0).unwrap();
        let simulation = Simulation::from_checkpoint(checkpoint).unwrap();
        let world = simulation.world().clone();
        let resume_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();

        fs::create_dir_all(&root).unwrap();
        write_json(&root.join("world.json"), &world);
        write_json(
            &root.join("resume-start-population.json"),
            &resume_population,
        );
        write_core_outputs(&root, &recorded.manifest, recorded.events(), recorded.metrics(), &recorded.checkpoint);

        let archive = pack::pack_completed_run(&root, None).unwrap();
        assert!(archive.is_file());
        cleanup(&root);
    }

    #[test]
    fn landscape_bound_bundle_packages_successfully() {
        let root = test_dir("landscape");
        let landscape = empty_landscape();
        let simulation = LandscapeSimulation::new(base_config(82, 0), landscape.clone()).unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();

        fs::create_dir_all(&root).unwrap();
        write_json(&root.join("landscape.json"), &landscape);
        write_json(&root.join("world.json"), &world);
        write_json(
            &root.join("initial-population.json"),
            &initial_population,
        );
        write_core_outputs(
            &root,
            recorded.core_manifest(),
            recorded.events(),
            recorded.metrics(),
            recorded.core_checkpoint(),
        );
        write_json(
            &root.join("landscape-manifest.json"),
            &recorded.manifest,
        );
        write_json(
            &root.join("landscape-checkpoint.json"),
            &recorded.checkpoint,
        );

        let archive = pack::pack_completed_run(&root, None).unwrap();
        assert!(archive.is_file());
        cleanup(&root);
    }

    #[test]
    fn transformed_spatial_bundle_packages_successfully() {
        let root = test_dir("spatial");
        let (landscape, mechanisms) = transformed_landscape();
        let simulation = SpatialLandscapeSimulation::new(
            base_config(83, 0),
            landscape.clone(),
            mechanisms.clone(),
        )
        .unwrap();
        let world = simulation.world().clone();
        let initial_population = simulation.population().clone();
        let recorded = simulation.run_recorded().unwrap();

        fs::create_dir_all(&root).unwrap();
        write_json(&root.join("landscape.json"), &landscape);
        write_json(&root.join("spatial-mechanisms.json"), &mechanisms);
        write_json(&root.join("world.json"), &world);
        write_json(
            &root.join("initial-population.json"),
            &initial_population,
        );
        write_core_outputs(
            &root,
            recorded.core_manifest(),
            recorded.events(),
            recorded.metrics(),
            recorded.core_checkpoint(),
        );
        write_json(
            &root.join("landscape-manifest.json"),
            &recorded.manifest,
        );
        write_json(
            &root.join("landscape-checkpoint.json"),
            &recorded.checkpoint,
        );

        let archive = pack::pack_completed_run(&root, None).unwrap();
        assert!(archive.is_file());
        cleanup(&root);
    }

    fn base_config(seed: u64, years: u64) -> ExperimentConfig {
        ExperimentConfig::new(seed, years)
            .with_world(WorldConfig::new(4, 4))
            .with_population(
                PopulationConfig::new(8)
                    .with_target_household_size(2)
                    .with_max_person_records(64),
            )
    }

    fn empty_landscape() -> LandscapeBundle {
        LandscapeBundle::new(4, 4, geometry(), Vec::new())
    }

    fn transformed_landscape() -> (LandscapeBundle, SpatialMechanismConfig) {
        let domain = LandscapeValueDomain { min: 0, max: 1_000 };
        let layer = LandscapeLayer {
            layer_id: "terrain".to_owned(),
            role: LandscapeLayerRole::TerrainTraversal,
            unit: "permille".to_owned(),
            value_domain: Some(domain),
            evidence_input_id: None,
            values: vec![Some(500); 16],
        };
        let transform = SpatialFieldTransform::new(
            SpatialTargetField::MovementCost,
            "terrain",
            "permille",
            domain,
            1_000,
            2_000,
            TransformDirection::Direct,
            NoDataPolicy::Reject,
        );
        (
            LandscapeBundle::new(4, 4, geometry(), vec![layer]),
            SpatialMechanismConfig::new("packer-test", vec![transform]),
        )
    }

    fn geometry() -> GridGeometry {
        GridGeometry {
            origin_x: 0,
            origin_y: 0,
            cell_size_x: 100,
            cell_size_y: 100,
            coordinate_unit: "m".to_owned(),
            spatial_reference: "EPSG:27700".to_owned(),
        }
    }

    fn write_core_outputs(
        root: &Path,
        manifest: &anthrosim_core::RunManifest,
        events: &anthrosim_core::EventLog,
        metrics: &anthrosim_core::MetricSeries,
        checkpoint: &anthrosim_core::SimulationCheckpoint,
    ) {
        write_json(&root.join("manifest.json"), manifest);
        write_json(&root.join("events.json"), events);
        write_json(&root.join("metrics.json"), metrics);
        write_json(&root.join("checkpoint.json"), checkpoint);
    }

    fn write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) {
        let json = serde_json::to_string_pretty(value).unwrap();
        fs::write(path, format!("{json}\n")).unwrap();
    }

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "anthrosim-pack-integration-{label}-{}-{id}",
            std::process::id()
        ))
    }

    fn cleanup(root: &Path) {
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(root.with_extension("zip"));
    }
}
