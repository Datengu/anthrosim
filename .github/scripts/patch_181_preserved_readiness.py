from pathlib import Path


def replace_exact(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected exactly one match, found {count}: {old!r}")
    p.write_text(text.replace(old, new, 1))


replace_exact(
    "crates/anthrosim-core/src/simulation.rs",
    "    provenance::{MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, SourceRevisionIdentity},\n    resources::{",
    "    provenance::{MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, SourceRevisionIdentity},\n    research_readiness::assess_evidence_closure,\n    resources::{",
)
replace_exact(
    "crates/anthrosim-core/src/simulation.rs",
    "            experiment: self.config.clone(),\n            artifact_schemas: ArtifactSchemas::current(),",
    "            experiment: self.config.clone(),\n            evidence_closure: assess_evidence_closure(&self.config),\n            artifact_schemas: ArtifactSchemas::current(),",
)

replace_exact(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    "    provenance::{MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, SourceRevisionIdentity},\n    resources::{",
    "    provenance::{MODEL_SEMANTICS_ID, ResumeBoundary, ResumeLineage, SourceRevisionIdentity},\n    research_readiness::assess_evidence_closure,\n    resources::{",
)
replace_exact(
    "crates/anthrosim-core/src/spatial_simulation.rs",
    "            experiment: self.config.clone(),\n            artifact_schemas: ArtifactSchemas::current(),",
    "            experiment: self.config.clone(),\n            evidence_closure: assess_evidence_closure(&self.config),\n            artifact_schemas: ArtifactSchemas::current(),",
)

replace_exact(
    "crates/anthrosim-core/src/invariants.rs",
    "    provenance::{MODEL_SEMANTICS_ID, SourceRevisionIdentity},\n    resources::{ResourceConfigError, ResourceError, ResourceSummary, validate_resource_config},",
    "    provenance::{MODEL_SEMANTICS_ID, SourceRevisionIdentity},\n    research_readiness::assess_evidence_closure,\n    resources::{ResourceConfigError, ResourceError, ResourceSummary, validate_resource_config},",
)
replace_exact(
    "crates/anthrosim-core/src/invariants.rs",
    "        || manifest.experiment != checkpoint.experiment\n        || manifest.start_time != SimTime::ZERO",
    "        || manifest.experiment != checkpoint.experiment\n        || manifest.evidence_closure != assess_evidence_closure(&checkpoint.experiment)\n        || manifest.start_time != SimTime::ZERO",
)
