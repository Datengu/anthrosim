#!/usr/bin/env python3
"""Keep living current-state documentation synchronized with executable provenance."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROVENANCE = ROOT / "crates" / "anthrosim-core" / "src" / "provenance.rs"

# Current-facing surfaces. Historical audit/release/evidence records are deliberately excluded:
# they must retain the identity they actually evaluated rather than being rewritten to latest main.
CURRENT_SEMANTICS_DOCS = (
    ROOT / "README.md",
    ROOT / "docs" / "architecture.md",
    ROOT / "docs" / "roadmap.md",
    ROOT / "docs" / "scientific-model.md",
    ROOT / "docs" / "research" / "README.md",
    ROOT / "docs" / "research" / "trace.md",
    ROOT / "docs" / "research" / "odd.md",
    ROOT / "docs" / "research" / "odd-d.md",
)
CURRENT_STATUS_DOCS = (
    ROOT / "README.md",
    ROOT / "docs" / "roadmap.md",
    ROOT / "docs" / "research" / "README.md",
    ROOT / "docs" / "research" / "trace.md",
)
ADR_SEMANTICS = ROOT / "docs" / "adr" / "0004-model-semantics-compatibility-identity.md"
M8_RESULT_DOC = ROOT / "docs" / "research" / "m8-first-evidence-grounded-benchmark-result.md"
M8_REFERENCE = ROOT / "examples" / "m8-first-evidence-grounded-benchmark" / "reference-result.json"
M9_RESULT_DOC = ROOT / "docs" / "research" / "m9-controlled-aggregation-benchmark-result.md"
M9_REFERENCE = ROOT / "examples" / "m9-controlled-aggregation-benchmark" / "reference-result.json"
V032_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.2.md"
V033_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.3.md"
V034_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.4.md"
V035_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.5.md"
VERSIONING_DOC = ROOT / "docs" / "release-versioning.md"
AGENT_GUIDANCE = ROOT / "AGENTS.md"
VISION_DOC = ROOT / "docs" / "vision.md"
CITATION_GUIDANCE = ROOT / "docs" / "research" / "citation.md"
CITATION_CFF = ROOT / "CITATION.cff"
CARGO_TOML = ROOT / "Cargo.toml"
CI_PATH_DOC = ROOT / "docs" / "ci-path-classification.md"
REQUIRED_CHECKS_DOC = ROOT / "docs" / "required-status-checks.md"
M9_IMPLEMENTATION_PLAN = ROOT / "docs" / "research" / "m9-implementation-sequence.md"
M9_ACCEPTANCE_RECORD = ROOT / "docs" / "research" / "m9-6-acceptance.md"
M9_INTEGRATION_RECORD = ROOT / "docs" / "research" / "m9-6-integration-audit.md"
AUDIT_V3_CHARTER = ROOT / "docs" / "research" / "audit-v3" / "README.md"
AUDIT_V4_CHARTER = ROOT / "docs" / "research" / "audit-v4" / "README.md"

V032_SEMANTICS_ID = "anthrosim-model-semantics-v19"
V032_SHORT = "v19"
V033_SEMANTICS_ID = "anthrosim-model-semantics-v21"
V033_SHORT = "v21"
V034_SEMANTICS_ID = "anthrosim-model-semantics-v25"
V034_SHORT = "v25"
V035_SEMANTICS_ID = "anthrosim-model-semantics-v33"
V035_SHORT = "v33"


def current_semantics_id() -> str:
    source = PROVENANCE.read_text(encoding="utf-8")
    match = re.search(r'MODEL_SEMANTICS_ID:\s*&str\s*=\s*"([^"]+)"', source)
    if match is None:
        raise AssertionError(f"could not find MODEL_SEMANTICS_ID in {PROVENANCE}")
    return match.group(1)


def short_version(identity: str) -> str:
    match = re.fullmatch(r"anthrosim-model-semantics-v(\d+)", identity)
    if match is None:
        raise AssertionError(f"unexpected model-semantics identity format: {identity}")
    return f"v{match.group(1)}"


def reference_semantics(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    match = re.search(r'"modelSemanticsId"\s*:\s*"([^"]+)"', text)
    if match is None:
        raise AssertionError(f"could not find modelSemanticsId in {path.relative_to(ROOT)}")
    return match.group(1)


def assert_no_wrong_current_semantics(path: Path, current_short: str) -> None:
    text = path.read_text(encoding="utf-8")
    for match in re.finditer(r"current model semantics v(\d+)", text, flags=re.IGNORECASE):
        found = f"v{match.group(1)}"
        if found != current_short:
            raise AssertionError(
                f"{path.relative_to(ROOT)} calls {found} current; executable provenance is {current_short}"
            )


def main() -> None:
    current_id = current_semantics_id()
    current_short = short_version(current_id)
    current_phrase = f"current model semantics {current_short}"

    cargo_text = CARGO_TOML.read_text(encoding="utf-8")
    workspace_section = re.search(r"\[workspace\.package\]\n(.*?)(?:\n\[|\Z)", cargo_text, flags=re.DOTALL)
    if workspace_section is None:
        raise AssertionError("Cargo.toml is missing [workspace.package]")
    workspace_version_match = re.search(r'^version\s*=\s*"([^"]+)"', workspace_section.group(1), flags=re.MULTILINE)
    if workspace_version_match is None:
        raise AssertionError("Cargo.toml workspace.package is missing version")
    current_software_version = workspace_version_match.group(1)
    current_release_tag = f"v{current_software_version}"
    release_phrase = f"immutable v0.3.4 release baseline: {V034_SHORT}"
    prior_release_phrase = f"immutable v0.3.3 release baseline: {V033_SHORT}"

    for path in CURRENT_SEMANTICS_DOCS:
        text = path.read_text(encoding="utf-8")
        assert_no_wrong_current_semantics(path, current_short)
        if current_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not identify executable current semantics as {current_short}"
            )
        if current_release_tag not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not identify current software release line as {current_release_tag}"
            )

    # Formal living model descriptions must preserve the immutable release distinction explicitly.
    for path in (
        ROOT / "docs" / "scientific-model.md",
        ROOT / "docs" / "research" / "odd.md",
        ROOT / "docs" / "research" / "odd-d.md",
        ROOT / "docs" / "research" / "trace.md",
    ):
        text = path.read_text(encoding="utf-8")
        if release_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not distinguish immutable v0.3.4/{V034_SHORT} from living main"
            )
        if prior_release_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not preserve immutable v0.3.3/{V033_SHORT} history"
            )

    for path in CURRENT_STATUS_DOCS:
        text = path.read_text(encoding="utf-8")
        stale_status = (
            "Scientific Audit v4 remediation in progress",
            "entered remediation; AV4-001",
            "Repair and independent post-merge re-verification take priority",
        )
        for stale in stale_status:
            if stale in text:
                raise AssertionError(f"stale Audit-v4 current-status text remains in {path.relative_to(ROOT)}: {stale}")

    adr_text = ADR_SEMANTICS.read_text(encoding="utf-8")
    if "currently `anthrosim-model-semantics-v1`" in adr_text:
        raise AssertionError("ADR 0004 still presents its original v1 value as the current executable identity")
    if "crates/anthrosim-core/src/provenance.rs" not in adr_text:
        raise AssertionError("ADR 0004 must point to executable provenance as current MODEL_SEMANTICS_ID authority")

    m8_reference_id = reference_semantics(M8_REFERENCE)
    m8_reference_short = short_version(m8_reference_id)
    m8_text = M8_RESULT_DOC.read_text(encoding="utf-8")
    if f"Current machine-readable reference: `{m8_reference_id}`" not in m8_text:
        raise AssertionError(
            f"M8.6 result does not identify checked-in current reference semantics {m8_reference_short}"
        )

    m9_reference_id = reference_semantics(M9_REFERENCE)
    m9_reference_short = short_version(m9_reference_id)
    m9_text = M9_RESULT_DOC.read_text(encoding="utf-8")
    if f"Current machine-readable reference: `{m9_reference_id}`" not in m9_text:
        raise AssertionError(
            f"M9.7 result does not identify checked-in current reference semantics {m9_reference_short}"
        )

    v032_release_text = V032_RELEASE_DOC.read_text(encoding="utf-8")
    expected_v032_identity = f'`MODEL_SEMANTICS_ID = "{V032_SEMANTICS_ID}"`'
    if expected_v032_identity not in v032_release_text:
        raise AssertionError(
            "docs/releases/v0.3.2.md does not preserve the immutable release semantics "
            f"identity {V032_SEMANTICS_ID}"
        )

    v033_release_text = V033_RELEASE_DOC.read_text(encoding="utf-8")
    expected_v033_identity = f'`MODEL_SEMANTICS_ID = "{V033_SEMANTICS_ID}"`'
    if expected_v033_identity not in v033_release_text:
        raise AssertionError(
            "docs/releases/v0.3.3.md does not preserve the audited release semantics "
            f"identity {V033_SEMANTICS_ID}"
        )

    v034_release_text = V034_RELEASE_DOC.read_text(encoding="utf-8")
    if f"Model semantics: `{V034_SEMANTICS_ID}`" not in v034_release_text:
        raise AssertionError(
            "docs/releases/v0.3.4.md does not identify the release semantics "
            f"as {V034_SEMANTICS_ID}"
        )

    v035_release_text = V035_RELEASE_DOC.read_text(encoding="utf-8")
    if "Software version: `0.3.5`" not in v035_release_text or f"Model semantics: `{V035_SEMANTICS_ID}`" not in v035_release_text:
        raise AssertionError("docs/releases/v0.3.5.md does not identify the v0.3.5/v33 release identity")

    versioning_text = VERSIONING_DOC.read_text(encoding="utf-8")
    if (
        "v0.3.2`**: documentation-convergence maintenance patch over the v19 model semantics "
        "preserved by the immutable `v0.3.2` tag"
        not in versioning_text
    ):
        raise AssertionError("docs/release-versioning.md does not preserve v0.3.2/v19 identity")
    if (
        "v0.3.3`**: post-scientific-audit-v2 hardening/convergence patch preserving the repaired "
        "v21 model-semantics baseline"
        not in versioning_text
    ):
        raise AssertionError("docs/release-versioning.md does not preserve v0.3.3/v21 identity")
    if (
        "v0.3.4`**: post-Scientific-Audit-v3 convergence patch preserving the fully remediated "
        "and independently reverified v25 model-semantics baseline"
        not in versioning_text
    ):
        raise AssertionError("docs/release-versioning.md does not preserve v0.3.4/v25 identity")
    if (
        "v0.3.5`**: post-Scientific-Audit-v4 repaired convergence patch preserving the fully "
        "remediated/reverified v33 model-semantics baseline"
        not in versioning_text
    ):
        raise AssertionError("docs/release-versioning.md does not preserve v0.3.5/v33 identity")
    if current_id not in versioning_text:
        raise AssertionError("release-versioning policy does not state the current model semantics")

    forbidden_current_status = {
        AGENT_GUIDANCE: ("M9 is the planned", "target named release is `v0.3.0`"),
        VISION_DOC: ("Evidence-grounded environments are a natural next step", "planned M8 boundary"),
        CI_PATH_DOC: ("is introducing conservative path-aware CI under issue #317", "Future #317 changes"),
        REQUIRED_CHECKS_DOC: (
            "After the aggregator is merged",
            "Until that administrative reconciliation is complete, issue #175 is not governance-complete",
        ),
    }
    for path, stale_phrases in forbidden_current_status.items():
        surface = path.read_text(encoding="utf-8")
        for stale in stale_phrases:
            if stale in surface:
                raise AssertionError(f"stale current-status text remains in {path.relative_to(ROOT)}: {stale}")

    cff_text = CITATION_CFF.read_text(encoding="utf-8")
    cff_version = re.search(r'^version:\s*["\']?([^"\'\s]+)', cff_text, flags=re.MULTILINE)
    if cff_version is None:
        raise AssertionError("CITATION.cff does not expose a version")
    if cff_version.group(1) != current_software_version:
        raise AssertionError("CITATION.cff version does not match Cargo workspace version")
    citation_text = CITATION_GUIDANCE.read_text(encoding="utf-8")
    if f"version `{cff_version.group(1)}`" not in citation_text:
        raise AssertionError("citation guidance does not match the current CITATION.cff release version")
    if "For v0.1, the citation metadata identifies AnthroSim version `0.1.0`" in citation_text:
        raise AssertionError("citation guidance still presents v0.1 metadata as current")

    historical_markers = {
        M9_IMPLEMENTATION_PLAN: "historical implementation plan",
        M9_ACCEPTANCE_RECORD: "**Historical record.**",
        M9_INTEGRATION_RECORD: "**Historical record.**",
    }
    for path, marker in historical_markers.items():
        if marker not in path.read_text(encoding="utf-8"):
            raise AssertionError(f"{path.relative_to(ROOT)} lacks an explicit historical-status marker")

    for path in (AUDIT_V3_CHARTER, AUDIT_V4_CHARTER):
        charter = path.read_text(encoding="utf-8")
        if "**Status: complete / historical charter.**" not in charter or "Do **not** restart or continue Audit" not in charter:
            raise AssertionError(f"{path.relative_to(ROOT)} can still be mistaken for an active audit charter")


if __name__ == "__main__":
    main()
