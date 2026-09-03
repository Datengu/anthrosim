#!/usr/bin/env python3
"""Keep living model-semantics labels synchronized with executable provenance."""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PROVENANCE = ROOT / "crates" / "anthrosim-core" / "src" / "provenance.rs"
CURRENT_DOCS = (
    ROOT / "docs" / "scientific-model.md",
    ROOT / "docs" / "research" / "odd.md",
    ROOT / "docs" / "research" / "odd-d.md",
)
V032_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.2.md"
V033_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.3.md"
V034_RELEASE_DOC = ROOT / "docs" / "releases" / "v0.3.4.md"
VERSIONING_DOC = ROOT / "docs" / "release-versioning.md"

V032_SEMANTICS_ID = "anthrosim-model-semantics-v19"
V032_SHORT = "v19"
V033_SEMANTICS_ID = "anthrosim-model-semantics-v21"
V033_SHORT = "v21"
V034_SEMANTICS_ID = "anthrosim-model-semantics-v25"
V034_SHORT = "v25"
CURRENT_SEMANTICS_ID = "anthrosim-model-semantics-v29"
CURRENT_SHORT = "v29"


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


def main() -> None:
    current_id = current_semantics_id()
    current_short = short_version(current_id)
    if current_id != CURRENT_SEMANTICS_ID:
        raise AssertionError(
            f"post-v0.3.4 remediation guard expects {CURRENT_SEMANTICS_ID}, got {current_id}"
        )
    if current_short != CURRENT_SHORT:
        raise AssertionError(f"current short semantics should be {CURRENT_SHORT}, got {current_short}")

    current_phrase = f"current model semantics {current_short}"
    release_phrase = f"immutable v0.3.4 release baseline: {V034_SHORT}"
    prior_release_phrase = f"immutable v0.3.3 release baseline: {V033_SHORT}"

    for path in CURRENT_DOCS:
        text = path.read_text(encoding="utf-8")
        if current_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not identify the executable current semantics "
                f"as {current_short} from MODEL_SEMANTICS_ID"
            )
        if release_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not distinguish the immutable v0.3.4 "
                f"release baseline ({V034_SHORT}) from the current remediation line"
            )
        if prior_release_phrase not in text:
            raise AssertionError(
                f"{path.relative_to(ROOT)} does not preserve the immutable v0.3.3 "
                f"release baseline ({V033_SHORT}) distinction"
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

    versioning_text = VERSIONING_DOC.read_text(encoding="utf-8")
    if (
        "v0.3.2`**: documentation-convergence maintenance patch over the v19 model semantics "
        "preserved by the immutable `v0.3.2` tag"
        not in versioning_text
    ):
        raise AssertionError(
            "docs/release-versioning.md does not identify v0.3.2 as the preserved v19 baseline"
        )
    if (
        "v0.3.3`**: post-scientific-audit-v2 hardening/convergence patch preserving the repaired "
        "v21 model-semantics baseline"
        not in versioning_text
    ):
        raise AssertionError(
            "docs/release-versioning.md does not identify v0.3.3 as the preserved v21 baseline"
        )
    if (
        "v0.3.4`**: post-Scientific-Audit-v3 convergence patch preserving the fully remediated "
        "and independently reverified v25 model-semantics baseline"
        not in versioning_text
    ):
        raise AssertionError(
            "docs/release-versioning.md does not identify v0.3.4 as the preserved v25 baseline"
        )

    stale_current_patterns = (
        "v0.3.2 package / post-M9 scientific-hardening line / model semantics v15",
        "v0.3.2 / completed M9 / post-M9 v15 scientific-hardening semantics",
        "same v15 model semantics",
        "`MODEL_SEMANTICS_ID` v15",
    )
    checked = "\n".join(
        path.read_text(encoding="utf-8")
        for path in (
            *CURRENT_DOCS,
            V032_RELEASE_DOC,
            V033_RELEASE_DOC,
            V034_RELEASE_DOC,
            VERSIONING_DOC,
        )
    )
    for stale in stale_current_patterns:
        if stale in checked:
            raise AssertionError(f"stale current/release semantics label remains: {stale}")


if __name__ == "__main__":
    main()
