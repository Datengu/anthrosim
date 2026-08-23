# Citing AnthroSim

AnthroSim carries machine-readable citation metadata in the repository-root `CITATION.cff` file. GitHub and other tooling can use that file to present a standard software citation.

## Released software

When referring to AnthroSim as software, cite the released version that was used. The software version identifies the public release line and should not be treated as a substitute for exact experiment provenance.

For v0.1, the citation metadata identifies AnthroSim version `0.1.0` and the Apache-2.0 software licence.

## Exact research source revision

A reproducible experiment should additionally preserve the exact Git commit recorded by AnthroSim's run/experiment provenance. Two commits can belong to the same software release line while differing in source, documentation, tooling or, where explicitly versioned, model semantics.

Published or archived research should therefore retain both:

- the human-facing AnthroSim release/version used for citation; and
- the exact Git commit/source identity recorded by the experiment artifacts for reproduction and audit.

Where checkpoint compatibility is relevant, the model-semantics identity is a separate compatibility concept and must not be replaced by either the citation version or Git revision.

## Data and external assets

The repository's Apache-2.0 licence applies to AnthroSim software. It does not automatically grant rights to third-party datasets, GIS layers, archaeological records, imagery or other external assets used by future evidence-grounded experiments. Such inputs must retain their own source, licence and reuse conditions alongside their scientific provenance.

If a future AnthroSim release bundles data or other non-software assets, their licensing must be stated explicitly rather than assumed to inherit the engine licence.
