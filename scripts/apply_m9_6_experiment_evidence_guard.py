from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    p = ROOT / path
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    p.write_text(text.replace(old, new, 1))


replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    "    events::{EventKind, EventLog, TemporaryJourneyIneligibility},\n    focal_region::{FocalRegion, FocalRegionError},\n",
    "    events::{EventKind, EventLog, TemporaryJourneyIneligibility},\n    evidence::EvidenceCatalog,\n    focal_region::{FocalRegion, FocalRegionError, FocalRegionSource},\n",
)

replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    "        self.travel_model.validate()?;\n        Ok(())\n    }\n\n    pub fn derive_program(\n",
    "        self.travel_model.validate()?;\n        Ok(())\n    }\n\n    /// Validate evidence provenance claimed by an evidence-bound focal-region source.\n    ///\n    /// A serialized M9 definition must not be able to claim that its region came from a\n    /// landscape mask while referring to an external evidence input that is absent from the\n    /// experiment catalogue. Synthetic regions do not require an evidence catalogue.\n    pub fn validate_evidence_context(\n        &self,\n        catalog: Option<&EvidenceCatalog>,\n    ) -> Result<(), TemporaryMobilityConfigError> {\n        self.validate()?;\n        let FocalRegionSource::LandscapeMask {\n            evidence_input_id, ..\n        } = &self.region.source\n        else {\n            return Ok(());\n        };\n        let catalog = catalog.ok_or_else(|| TemporaryMobilityConfigError::MissingEvidenceCatalog {\n            input_id: evidence_input_id.clone(),\n        })?;\n        if !catalog\n            .external_inputs\n            .iter()\n            .any(|input| input.input_id == *evidence_input_id)\n        {\n            return Err(TemporaryMobilityConfigError::UnknownEvidenceInput {\n                input_id: evidence_input_id.clone(),\n            });\n        }\n        Ok(())\n    }\n\n    pub fn derive_program(\n",
)

replace_once(
    "crates/anthrosim-core/src/temporary_mobility.rs",
    "    UnsupportedSchema { found: u32, supported: u32 },\n    #[error(transparent)]\n    Region(#[from] FocalRegionError),\n",
    "    UnsupportedSchema { found: u32, supported: u32 },\n    #[error(\"temporary-mobility focal region references evidence external input {input_id}, but no evidence catalogue was supplied\")]\n    MissingEvidenceCatalog { input_id: String },\n    #[error(\"temporary-mobility focal region references unknown evidence external input {input_id}\")]\n    UnknownEvidenceInput { input_id: String },\n    #[error(transparent)]\n    Region(#[from] FocalRegionError),\n",
)

for path in [
    "crates/anthrosim-core/src/simulation.rs",
    "crates/anthrosim-core/src/spatial_simulation.rs",
]:
    replace_once(
        path,
        "    if let Some(temporary_mobility) = &config.temporary_mobility {\n        temporary_mobility.validate()?;\n    }\n    if let Some(evidence) = &config.evidence {\n",
        "    if let Some(temporary_mobility) = &config.temporary_mobility {\n        temporary_mobility.validate_evidence_context(config.evidence.as_ref())?;\n    }\n    if let Some(evidence) = &config.evidence {\n",
    )

TEST = ROOT / "crates/anthrosim-core/tests/temporary_mobility_experiment_identity.rs"
text = TEST.read_text()
text = text.replace(
    "    DemographyConfig, EventKind, ExperimentConfig, FocalRegion, FocalRegionSource, MigrationConfig,\n    PopulationConfig, ResourceConfig, Simulation, SimulationError, TemporaryMobilityConfig,\n",
    "    DemographyConfig, EventKind, EvidenceCatalog, ExperimentConfig, ExternalInputEvidence,\n    FocalRegion, FocalRegionSource, MigrationConfig, PopulationConfig, ResourceConfig, Simulation,\n    SimulationError, TemporaryMobilityConfig, TemporaryMobilityConfigError,\n",
    1,
)
append = r'''

fn landscape_mask_definition(input_id: &str) -> TemporaryMobilityConfig {
    let region = FocalRegion::new(
        "evidence-bound-experiment-region",
        FocalRegionSource::LandscapeMask {
            layer_id: "aggregation-mask".to_owned(),
            evidence_input_id: input_id.to_owned(),
        },
        vec![CellId::new(4)],
    )
    .expect("evidence-bound region");
    let schedule = TemporaryMobilitySchedule::new(
        "evidence-bound-schedule",
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
    .expect("temporary mobility definition")
}

#[test]
fn landscape_mask_region_requires_an_evidence_catalog() {
    let configured = ExperimentConfig::new(96_106, 0)
        .with_world(WorldConfig::new(4, 1))
        .with_temporary_mobility(landscape_mask_definition("aggregation-mask-input"));

    assert!(matches!(
        Simulation::new(configured),
        Err(SimulationError::TemporaryMobilityConfig(
            TemporaryMobilityConfigError::MissingEvidenceCatalog { input_id }
        )) if input_id == "aggregation-mask-input"
    ));
}

#[test]
fn landscape_mask_region_rejects_unknown_evidence_external_input() {
    let catalog = EvidenceCatalog::new(Vec::new()).with_external_inputs(vec![ExternalInputEvidence {
        input_id: "different-input".to_owned(),
        evidence_id: "unused-in-this-preflight".to_owned(),
        format: "normalized-binary-mask".to_owned(),
        spatial_reference: None,
        content_digest: None,
    }]);
    let configured = ExperimentConfig::new(96_107, 0)
        .with_world(WorldConfig::new(4, 1))
        .with_temporary_mobility(landscape_mask_definition("aggregation-mask-input"))
        .with_evidence(catalog);

    assert!(matches!(
        Simulation::new(configured),
        Err(SimulationError::TemporaryMobilityConfig(
            TemporaryMobilityConfigError::UnknownEvidenceInput { input_id }
        )) if input_id == "aggregation-mask-input"
    ));
}
'''
if append.strip() in text:
    raise SystemExit("acceptance tests already appended")
TEST.write_text(text + append)

print("patched M9.6 temporary-mobility evidence context guard")
