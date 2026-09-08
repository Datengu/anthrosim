# Citing AnthroSim

AnthroSim carries machine-readable citation metadata in the repository-root `CITATION.cff` file. GitHub and other tooling can use that file to present a standard software citation. When GitHub recognises the file, its **Cite this repository** interface can generate common citation formats from the same metadata.

## Released software

When referring to AnthroSim as software, cite the released version that was used. The software version identifies the named public release baseline and should not be treated as a substitute for exact experiment provenance.

The current repository citation metadata identifies AnthroSim version `0.3.6` and the Apache-2.0 software licence. The v0.3.6 release preserves `anthrosim-model-semantics-v35`, the fully remediated Audit-v5 state. The immutable `v0.3.5`/v33 release remains the Audit-v5 discovery target and is not rewritten by this patch release. Audit v5 was a non-clean convergence pass, so v0.3.6 is a repaired framework baseline for further independent auditing rather than evidence that the first empirical/site-specific study is ready to begin. Consult `docs/release-versioning.md` for the identity distinction.

If a study uses a different named release, cite that release rather than mechanically citing whatever version happens to appear in the current repository metadata.

## Exact research source revision

A reproducible experiment should additionally preserve the exact Git commit recorded by AnthroSim's run/experiment provenance. Two commits can belong to the same software release line while differing in source, documentation, tooling or, where explicitly versioned, model semantics.

Published or archived research should therefore retain both:

- the human-facing AnthroSim release/version used for citation; and
- the exact Git commit/source identity recorded by the experiment artifacts for reproduction and audit.

Where checkpoint compatibility or scientific interpretation is relevant, preserve the recorded model-semantics identity as a separate compatibility/scientific identity. It must not be replaced by either the citation version or Git revision.

If research deliberately uses an unreleased protected-`main` development state, do not invent a new release citation for it. Preserve the exact source revision and model-semantics identity and describe the named release/package line from which that development state derives.

## Data and external assets

The repository's Apache-2.0 licence applies to AnthroSim software. It does not automatically grant rights to third-party datasets, GIS layers, archaeological records, imagery or other external assets used by future evidence-grounded experiments. Such inputs must retain their own source, licence and reuse conditions alongside their scientific provenance.

If a future AnthroSim release bundles data or other non-software assets, their licensing must be stated explicitly rather than assumed to inherit the engine licence.
