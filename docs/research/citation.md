# Citing AnthroSim

AnthroSim carries machine-readable citation metadata in the repository-root `CITATION.cff` file. GitHub and other tooling can use that file to present a standard software citation. When GitHub recognises the file, its **Cite this repository** interface can generate common citation formats from the same metadata.

## Released software

When referring to AnthroSim as software, cite the released version that was used. The software version identifies the named public release baseline and should not be treated as a substitute for exact experiment provenance.

The current repository citation metadata identifies AnthroSim version `0.3.4` and the Apache-2.0 software licence. That citation version refers to the immutable named `v0.3.4` release baseline. Living protected `main` can contain later compatible or scientifically versioned development work without retroactively changing what the `v0.3.4` tag represents; consult `docs/release-versioning.md` for that distinction.

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
