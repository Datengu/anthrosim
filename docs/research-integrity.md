# Research archive integrity

AnthroSim distinguishes **simulation-state fingerprints** from **cryptographic file integrity**.

State digests such as the checkpoint/run digest exist to detect deterministic simulation-state differences. They are not cryptographic checksums and must not be used to claim that a copied, published or long-term-preserved file has remained byte-identical.

For preserved research material, `scripts/research-integrity.py` provides a separate SHA-256 integrity layer over an archive directory.

## When to use it

An integrity manifest is not required for every temporary local run. Ordinary run validation, checkpoint reconciliation and semantic bundle validation remain the appropriate correctness checks while developing or exploring locally.

Generate an integrity manifest when a directory is intended to become a stable research record, for example before:

- publishing or depositing an experiment/sweep archive;
- attaching files to a paper, report, DOI or external repository;
- transferring a canonical result between machines or collaborators;
- placing a completed research bundle into long-term storage.

The tool is deliberately generic: the root may be a single completed run, an M7 experiment/sweep tree, a publication staging directory containing ZIP files and documentation, or another deliberately assembled research archive.

## Create a manifest

From the repository root:

```text
python3 scripts/research-integrity.py create runs/my-study
```

On Windows PowerShell, use `python` instead of `python3` if that is the installed launcher:

```text
python scripts/research-integrity.py create runs/my-study
```

By default this writes:

```text
runs/my-study/integrity-manifest.json
```

A custom manifest path can be supplied with `--output`.

The version-1 manifest contains:

- `manifestType: "anthrosim-research-integrity"`;
- `schemaVersion: 1`;
- `algorithm: "sha256"`;
- canonical POSIX-style relative paths;
- exact byte size for every archived regular file;
- lower-case SHA-256 for every archived regular file.

Entries are sorted by relative path and the JSON has no timestamp or host-specific absolute root, so recreating a manifest for an unchanged archive produces the same manifest bytes.

When the manifest itself is inside the archive root, that file alone is excluded from its own file list; self-hashing would otherwise be recursive. All other regular files are included. Symbolic links and non-regular filesystem entries are rejected so the meaning of an archived relative path cannot depend on external filesystem state.

## Verify an archive

```text
python3 scripts/research-integrity.py verify runs/my-study
```

Verification fails if:

- a manifested file is missing;
- an extra regular file has appeared;
- a file size differs;
- a SHA-256 differs;
- the manifest contains duplicate, absolute, parent-traversing or otherwise non-canonical paths;
- a symbolic link or unsupported filesystem entry is present;
- the manifest schema/type/algorithm is unsupported.

Verification therefore checks the exact preserved file set, not merely the files that happen still to exist.

## Relationship to `anthrosim-pack`

`anthrosim-pack` remains the deterministic convenience tool for turning one semantically valid completed run into a single ZIP. ZIP CRC32 values are transport/error-detection checksums, not cryptographic research-integrity hashes.

Two useful archive patterns are:

1. **Per-artifact integrity:** create `integrity-manifest.json` in the canonical run/experiment/sweep directory before preservation. Keep that trusted manifest with the copied or deposited files so each underlying artifact can be verified.
2. **Publication-package integrity:** assemble the exact files being deposited (for example a deterministic run ZIP, source definition, results table and README) in one staging directory, then run `research-integrity.py create` on that directory. The resulting manifest hashes every deposited file, including the ZIP as one artifact.

The packer intentionally remains a semantic completed-run packer rather than becoming the authority for arbitrary research-directory contents.

## Trust boundary

SHA-256 detects changes relative to a **trusted copy of the integrity manifest**. The version-1 format does not sign the manifest and does not establish who created it. For publication, preserve the manifest through the same trusted repository/DOI/release record as the research archive. Signing can be added later if a concrete authenticity requirement justifies it.

This cryptographic layer does not replace:

- run/bundle semantic validation;
- deterministic state digests;
- source revision and model-semantics provenance;
- experiment definitions and evidence provenance.

Those answer different questions. The integrity manifest answers only: **are these preserved files exactly the files represented by this trusted manifest?**
