from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load(path: str) -> str:
    return (ROOT / path).read_text()


def save(path: str, text: str) -> None:
    (ROOT / path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one occurrence, found {count}: {old[:120]!r}")
    return text.replace(old, new, 1)


# ensemble.rs: immutable settings, loader/preflight, config propagation, execution acceptance.
path = "crates/anthrosim-cli/src/ensemble.rs"
text = load(path)
text = replace_once(
    text,
    "    Simulation, SimulationCheckpoint, SpatialLandscapeCheckpoint, SpatialLandscapeRecordedRun,\n    SpatialLandscapeRunManifest, SpatialLandscapeSimulation, SpatialMechanismConfig, World,\n    WorldConfig, validate_spatial_landscape_recorded_run,\n",
    "    Simulation, SimulationCheckpoint, SpatialLandscapeCheckpoint, SpatialLandscapeRecordedRun,\n    SpatialLandscapeRunManifest, SpatialLandscapeSimulation, SpatialMechanismConfig,\n    TemporaryMobilityConfig, World, WorldConfig, validate_spatial_landscape_recorded_run,\n",
    "ensemble imports",
)
text = replace_once(
    text,
    "    pub(crate) disable_migration: bool,\n    pub(crate) migration_radius: u16,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub(crate) spatial: Option<SpatialRunSettings>,\n",
    "    pub(crate) disable_migration: bool,\n    pub(crate) migration_radius: u16,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub(crate) temporary_mobility: Option<TemporaryMobilityConfig>,\n    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n    pub(crate) spatial: Option<SpatialRunSettings>,\n",
    "ensemble settings field",
)
needle = "pub(crate) fn experiment_config(seed: u64, settings: &EnsembleRunSettings) -> ExperimentConfig {\n"
insert = '''pub(crate) fn load_temporary_mobility_config(
    path: &Path,
) -> Result<TemporaryMobilityConfig, Box<dyn std::error::Error>> {
    let definition: TemporaryMobilityConfig = read_json(path)?;
    definition.validate()?;
    Ok(definition)
}

pub(crate) fn validate_temporary_mobility_settings(
    settings: &EnsembleRunSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(definition) = &settings.temporary_mobility {
        let evidence = settings
            .spatial
            .as_ref()
            .and_then(|spatial| spatial.evidence.as_ref());
        definition.validate_evidence_context(evidence)?;
    }
    Ok(())
}

'''
text = replace_once(text, needle, insert + needle, "ensemble loader insertion")
old = '''    let config = ExperimentConfig::new(seed, settings.years)
        .with_world(WorldConfig::new(
            settings.world_width,
            settings.world_height,
        ))
        .with_population(
            PopulationConfig::new(settings.population)
                .with_target_household_size(settings.household_size)
                .with_max_person_records(settings.max_person_records),
        )
        .with_resources(resources)
        .with_migration(migration);

    settings
        .spatial
        .as_ref()
        .and_then(|spatial| spatial.evidence.as_ref())
        .map_or(config.clone(), |evidence| {
            config.with_evidence(evidence.clone())
        })
'''
new = '''    let mut config = ExperimentConfig::new(seed, settings.years)
        .with_world(WorldConfig::new(
            settings.world_width,
            settings.world_height,
        ))
        .with_population(
            PopulationConfig::new(settings.population)
                .with_target_household_size(settings.household_size)
                .with_max_person_records(settings.max_person_records),
        )
        .with_resources(resources)
        .with_migration(migration);
    if let Some(temporary_mobility) = &settings.temporary_mobility {
        config = config.with_temporary_mobility(temporary_mobility.clone());
    }
    if let Some(evidence) = settings
        .spatial
        .as_ref()
        .and_then(|spatial| spatial.evidence.as_ref())
    {
        config = config.with_evidence(evidence.clone());
    }
    config
'''
text = replace_once(text, old, new, "ensemble experiment config")
text = replace_once(
    text,
    "    let runtime_landscape = load_runtime_landscape(directory, &settings)?;\n",
    "    validate_temporary_mobility_settings(&settings)?;\n    let runtime_landscape = load_runtime_landscape(directory, &settings)?;\n",
    "ensemble preflight",
)
text = replace_once(
    text,
    "        EvidenceRecord, EvidenceSource, GridGeometry, LandscapeLayer, LandscapeLayerRole,\n        LandscapeValueDomain, NoDataPolicy, ParameterProvenance, SpatialFieldTransform,\n        SpatialTargetField, TransformDirection,\n",
    "        EvidenceRecord, EvidenceSource, FocalRegion, FocalRegionSource, GridGeometry,\n        LandscapeLayer, LandscapeLayerRole, LandscapeValueDomain, NoDataPolicy,\n        ParameterProvenance, SpatialFieldTransform, SpatialTargetField, TemporaryMobilitySchedule,\n        TemporaryTravelModel, TemporaryTriggerTiming, TransformDirection,\n        ids::CellId,\n",
    "ensemble test imports",
)
text = replace_once(
    text,
    "            disable_migration: false,\n            migration_radius: 3,\n            spatial: None,\n",
    "            disable_migration: false,\n            migration_radius: 3,\n            temporary_mobility: None,\n            spatial: None,\n",
    "ensemble small settings",
)
# Append acceptance tests immediately before the final module brace.
acceptance = r'''

    fn temporary_mobility_definition() -> TemporaryMobilityConfig {
        let region = FocalRegion::new(
            "cli-ensemble-region",
            FocalRegionSource::Synthetic,
            vec![CellId::new(4)],
        )
        .expect("region");
        let schedule = TemporaryMobilitySchedule::new(
            "cli-ensemble-schedule",
            TemporaryTriggerTiming::DepartureDay,
            vec![100],
            5,
        )
        .expect("schedule");
        TemporaryMobilityConfig::new(
            region,
            schedule,
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .expect("temporary mobility")
    }

    #[test]
    fn ensemble_derives_configured_temporary_mobility_from_each_stored_world() {
        let root = temp_path("m9-6-ensemble-temporary-mobility");
        let definition = temporary_mobility_definition();
        let mut settings = small_settings();
        settings.years = 1;
        settings.world_width = 4;
        settings.world_height = 1;
        settings.population = 24;
        settings.annual_food_need = 0;
        settings.disable_migration = true;
        settings.temporary_mobility = Some(definition.clone());
        let seeds = vec![96_201, 96_202];

        execute_ensemble(&root, settings, seeds.clone(), false).expect("ensemble");

        for seed in seeds {
            let run_dir = root.join(run_relative_dir(seed));
            let world: World = read_json(&run_dir.join("world.json")).expect("world");
            let checkpoint: SimulationCheckpoint =
                read_json(&run_dir.join("checkpoint.json")).expect("checkpoint");
            assert_eq!(
                checkpoint.experiment.temporary_mobility.as_ref(),
                Some(&definition)
            );
            let expected = definition
                .derive_program(&world)
                .expect("program from this run world");
            assert_eq!(checkpoint.temporary_mobility.program(), Some(&expected));
        }

        fs::remove_dir_all(root).expect("cleanup");
    }
'''
if acceptance.strip() in text:
    raise SystemExit("ensemble acceptance already present")
pos = text.rfind("\n}")
if pos == -1:
    raise SystemExit("ensemble final module brace not found")
text = text[:pos] + acceptance + text[pos:]
save(path, text)


# sweep.rs: preflight and provenance acceptance. Expanded points already clone base settings.
path = "crates/anthrosim-cli/src/sweep.rs"
text = load(path)
text = replace_once(
    text,
    "    ensemble::{EnsembleRunSettings, execute_ensemble},\n",
    "    ensemble::{\n        EnsembleRunSettings, execute_ensemble, validate_temporary_mobility_settings,\n    },\n",
    "sweep import preflight",
)
text = replace_once(
    text,
    "    let expected = build_sweep_manifest(base_settings, seeds, dimensions)?;\n",
    "    validate_temporary_mobility_settings(&base_settings)?;\n    let expected = build_sweep_manifest(base_settings, seeds, dimensions)?;\n",
    "sweep preflight",
)
text = replace_once(
    text,
    "            disable_migration: false,\n            migration_radius: 3,\n            spatial: None,\n",
    "            disable_migration: false,\n            migration_radius: 3,\n            temporary_mobility: None,\n            spatial: None,\n",
    "sweep small settings",
)
# Test without adding new imports: reuse the definition through the ensemble module path.
sweep_test = r'''

    #[test]
    fn temporary_mobility_is_preserved_across_points_and_changes_sweep_identity() {
        use anthrosim_core::{
            FocalRegion, FocalRegionSource, TemporaryMobilityConfig, TemporaryMobilitySchedule,
            TemporaryTravelModel, TemporaryTriggerTiming, ids::CellId,
        };

        let definition = TemporaryMobilityConfig::new(
            FocalRegion::new(
                "sweep-temporary-region",
                FocalRegionSource::Synthetic,
                vec![CellId::new(4)],
            )
            .expect("region"),
            TemporaryMobilitySchedule::new(
                "sweep-temporary-schedule",
                TemporaryTriggerTiming::DepartureDay,
                vec![100],
                5,
            )
            .expect("schedule"),
            TemporaryTravelModel::synthetic_validation_v1(),
        )
        .expect("temporary mobility");
        let mut settings = small_settings();
        settings.temporary_mobility = Some(definition.clone());
        let first = build_sweep_manifest(settings.clone(), vec![3, 7], dimensions()).expect("sweep");
        assert!(
            first
                .points
                .iter()
                .all(|point| point.settings.temporary_mobility.as_ref() == Some(&definition))
        );

        settings
            .temporary_mobility
            .as_mut()
            .expect("temporary mobility")
            .schedule
            .stay_duration_days += 1;
        let changed = build_sweep_manifest(settings, vec![3, 7], dimensions()).expect("sweep");
        assert_ne!(first.sweep_id, changed.sweep_id);
    }
'''
if sweep_test.strip() in text:
    raise SystemExit("sweep acceptance already present")
pos = text.rfind("\n}")
if pos == -1:
    raise SystemExit("sweep final module brace not found")
text = text[:pos] + sweep_test + text[pos:]
save(path, text)


# main.rs: one versioned file input for Run, Ensemble and Sweep.
path = "crates/anthrosim-cli/src/main.rs"
text = load(path)
text = replace_once(
    text,
    "    EnsembleRunSettings, execute_ensemble, experiment_config, load_spatial_run_settings,\n    resolve_ensemble_seeds,\n",
    "    EnsembleRunSettings, execute_ensemble, experiment_config, load_spatial_run_settings,\n    load_temporary_mobility_config, resolve_ensemble_seeds,\n",
    "main ensemble imports",
)
# Run arg.
text = replace_once(
    text,
    "        /// Manhattan-radius local knowledge used for migration destination discovery.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional path to write the JSON run manifest (legacy single-file mode).\n",
    "        /// Manhattan-radius local knowledge used for migration destination discovery.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional versioned M9 temporary-mobility definition JSON.\n        #[arg(long)]\n        temporary_mobility: Option<PathBuf>,\n\n        /// Optional path to write the JSON run manifest (legacy single-file mode).\n",
    "run temporary arg",
)
# Ensemble arg.
text = replace_once(
    text,
    "        /// Manhattan-radius local knowledge used for migration destination discovery.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional normalized M8.1 LandscapeBundle JSON; requires --mechanisms.\n",
    "        /// Manhattan-radius local knowledge used for migration destination discovery.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional versioned M9 temporary-mobility definition JSON shared by every run.\n        #[arg(long)]\n        temporary_mobility: Option<PathBuf>,\n\n        /// Optional normalized M8.1 LandscapeBundle JSON; requires --mechanisms.\n",
    "ensemble temporary arg",
)
# Sweep arg.
text = replace_once(
    text,
    "        /// Base migration radius when its sweep dimension is not supplied.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional normalized M8.1 LandscapeBundle JSON shared by every point; requires --mechanisms.\n",
    "        /// Base migration radius when its sweep dimension is not supplied.\n        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n\n        /// Optional versioned M9 temporary-mobility definition JSON shared by every point/run.\n        #[arg(long)]\n        temporary_mobility: Option<PathBuf>,\n\n        /// Optional normalized M8.1 LandscapeBundle JSON shared by every point; requires --mechanisms.\n",
    "sweep temporary arg",
)
# Destructuring and settings: Run.
text = replace_once(
    text,
    "            disable_migration,\n            migration_radius,\n            output,\n",
    "            disable_migration,\n            migration_radius,\n            temporary_mobility,\n            output,\n",
    "run destructure",
)
text = replace_once(
    text,
    "        } => {\n            let settings = EnsembleRunSettings {\n                years,\n",
    "        } => {\n            let temporary_mobility = temporary_mobility\n                .as_deref()\n                .map(load_temporary_mobility_config)\n                .transpose()?;\n            let settings = EnsembleRunSettings {\n                years,\n",
    "run load temporary",
)
text = replace_once(
    text,
    "                disable_migration,\n                migration_radius,\n                spatial: None,\n",
    "                disable_migration,\n                migration_radius,\n                temporary_mobility,\n                spatial: None,\n",
    "run settings temporary",
)
# Ensemble destructure/load/settings.
text = replace_once(
    text,
    "            disable_migration,\n            migration_radius,\n            landscape,\n            mechanisms,\n            evidence,\n            run_dir,\n            retry,\n        } => {\n",
    "            disable_migration,\n            migration_radius,\n            temporary_mobility,\n            landscape,\n            mechanisms,\n            evidence,\n            run_dir,\n            retry,\n        } => {\n",
    "ensemble destructure",
)
text = replace_once(
    text,
    "            let settings = EnsembleRunSettings {\n                years,\n                world_width,\n                world_height,\n                population,\n                household_size,\n                max_person_records,\n                resource_productivity_scale_permille,\n                resource_seasonality_scale_permille,\n                annual_food_need,\n                disable_migration,\n                migration_radius,\n                spatial,\n            };\n            execute_ensemble(&run_dir, settings, seeds, retry)?;\n",
    "            let temporary_mobility = temporary_mobility\n                .as_deref()\n                .map(load_temporary_mobility_config)\n                .transpose()?;\n            let settings = EnsembleRunSettings {\n                years,\n                world_width,\n                world_height,\n                population,\n                household_size,\n                max_person_records,\n                resource_productivity_scale_permille,\n                resource_seasonality_scale_permille,\n                annual_food_need,\n                disable_migration,\n                migration_radius,\n                temporary_mobility,\n                spatial,\n            };\n            execute_ensemble(&run_dir, settings, seeds, retry)?;\n",
    "ensemble settings/load",
)
# Sweep destructure/load/settings.
text = replace_once(
    text,
    "            disable_migration,\n            migration_radius,\n            landscape,\n            mechanisms,\n            evidence,\n            sweep_population,\n",
    "            disable_migration,\n            migration_radius,\n            temporary_mobility,\n            landscape,\n            mechanisms,\n            evidence,\n            sweep_population,\n",
    "sweep destructure",
)
text = replace_once(
    text,
    "            let settings = EnsembleRunSettings {\n                years,\n                world_width,\n                world_height,\n                population,\n                household_size,\n                max_person_records,\n                resource_productivity_scale_permille,\n                resource_seasonality_scale_permille,\n                annual_food_need,\n                disable_migration,\n                migration_radius,\n                spatial,\n            };\n            let dimensions = SweepDimensions {\n",
    "            let temporary_mobility = temporary_mobility\n                .as_deref()\n                .map(load_temporary_mobility_config)\n                .transpose()?;\n            let settings = EnsembleRunSettings {\n                years,\n                world_width,\n                world_height,\n                population,\n                household_size,\n                max_person_records,\n                resource_productivity_scale_permille,\n                resource_seasonality_scale_permille,\n                annual_food_need,\n                disable_migration,\n                migration_radius,\n                temporary_mobility,\n                spatial,\n            };\n            let dimensions = SweepDimensions {\n",
    "sweep settings/load",
)
save(path, text)


# anthrosim-landscape: expose the same definition for single evidence-bound runs.
path = "crates/anthrosim-cli/src/bin/anthrosim-landscape.rs"
text = load(path)
text = replace_once(
    text,
    "    SpatialLandscapeRecordedRun, SpatialLandscapeSimulation, SpatialMechanismConfig, World,\n    WorldConfig, validate_landscape_recorded_run_invariants,\n",
    "    SpatialLandscapeRecordedRun, SpatialLandscapeSimulation, SpatialMechanismConfig,\n    TemporaryMobilityConfig, World, WorldConfig, validate_landscape_recorded_run_invariants,\n",
    "landscape imports",
)
text = replace_once(
    text,
    "        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n        /// Controlled output directory containing core and landscape-bound artifacts.\n",
    "        #[arg(long, default_value_t = 3)]\n        migration_radius: u16,\n        /// Optional versioned M9 temporary-mobility definition JSON.\n        #[arg(long)]\n        temporary_mobility: Option<PathBuf>,\n        /// Controlled output directory containing core and landscape-bound artifacts.\n",
    "landscape temporary arg",
)
text = replace_once(
    text,
    "            disable_migration,\n            migration_radius,\n            run_dir,\n",
    "            disable_migration,\n            migration_radius,\n            temporary_mobility,\n            run_dir,\n",
    "landscape destructure",
)
text = replace_once(
    text,
    "            let evidence = evidence\n                .as_deref()\n                .map(read_json::<EvidenceCatalog>)\n                .transpose()?;\n            let config = experiment_config(\n",
    "            let evidence = evidence\n                .as_deref()\n                .map(read_json::<EvidenceCatalog>)\n                .transpose()?;\n            let temporary_mobility = temporary_mobility\n                .as_deref()\n                .map(read_json::<TemporaryMobilityConfig>)\n                .transpose()?;\n            if let Some(definition) = &temporary_mobility {\n                definition.validate()?;\n            }\n            let config = experiment_config(\n",
    "landscape load temporary",
)
text = replace_once(
    text,
    "                migration_radius,\n                evidence,\n            );\n",
    "                migration_radius,\n                evidence,\n                temporary_mobility,\n            );\n",
    "landscape config call",
)
text = replace_once(
    text,
    "    migration_radius: u16,\n    evidence: Option<EvidenceCatalog>,\n) -> ExperimentConfig {\n",
    "    migration_radius: u16,\n    evidence: Option<EvidenceCatalog>,\n    temporary_mobility: Option<TemporaryMobilityConfig>,\n) -> ExperimentConfig {\n",
    "landscape config signature",
)
old = "    let config = ExperimentConfig::new(seed, years)\n        .with_world(WorldConfig::new(width, height))\n        .with_population(\n            PopulationConfig::new(population)\n                .with_target_household_size(household_size)\n                .with_max_person_records(max_person_records),\n        )\n        .with_resources(resources)\n        .with_migration(migration);\n    evidence.map_or(config.clone(), |catalog| config.with_evidence(catalog))\n"
new = "    let mut config = ExperimentConfig::new(seed, years)\n        .with_world(WorldConfig::new(width, height))\n        .with_population(\n            PopulationConfig::new(population)\n                .with_target_household_size(household_size)\n                .with_max_person_records(max_person_records),\n        )\n        .with_resources(resources)\n        .with_migration(migration);\n    if let Some(temporary_mobility) = temporary_mobility {\n        config = config.with_temporary_mobility(temporary_mobility);\n    }\n    if let Some(catalog) = evidence {\n        config = config.with_evidence(catalog);\n    }\n    config\n"
text = replace_once(text, old, new, "landscape config build")
save(path, text)

print("patched M9.6 CLI/ensemble/sweep temporary-mobility integration")
