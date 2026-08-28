# Downstream analysis provenance v1

Status: normative research contract for GitHub issue #232.

AnthroSim simulation provenance answers **which model execution produced the preserved run data**. This contract answers the separate question **which executable analysis produced a reported scientific result from those data**.

A deterministic simulation is not by itself a reproducible inference. Filtering, censoring, analysis windows, aggregation, statistical methods, observation-model choices, package versions, analysis RNG, and manual edits can all change a result after the underlying runs have been frozen.

## Scope and boundary

This layer is downstream of:

- `anthrosim-research`, which freezes and executes the scientific parameter/seed design;
- `anthrosim-study`, which freezes the #230 study protocol and binds it to the completed research execution;
- #219 analysis-window provenance where a study excludes burn-in/transient periods.

It does **not** alter authoritative simulation state, RNG streams, checkpoints, run identities, or `MODEL_SEMANTICS_ID`.

The v1 analysis provenance wrapper is:

```text
scripts/research-analysis-provenance.py
```

It supports four scientific operations:

```text
run      execute a declared scripted analysis and publish its provenance record
capture  bind an already-produced external/manual result while making that fact explicit
replay   reproduce a wrapper-run analysis in an isolated temporary root and require exact output bytes
verify   verify the current canonical files against an existing provenance record
```

## Required analysis definition

Schema v1 is `anthrosim-analysis-definition`.

A definition declares:

- `analysisId`;
- `analysisStatus`: `exploratory` or `confirmatory`;
- `executionMode`: `scripted` or `external_or_manual`;
- analysis working directory;
- exact command argv;
- machine-readable analysis arguments/configuration;
- analysis RNG seeds;
- runtime/dependency description;
- `reproductionCriterion`;
- exact input artifacts;
- exact implementation artifacts (scripts/notebooks/helpers);
- exact environment artifacts (lockfiles, environment exports, container recipes, etc.);
- exact output artifacts;
- any unavoidable manual steps;
- optional observation-model identity.

Schema v1 deliberately supports one reproduction criterion:

```text
exact_output_bytes
```

That means the canonical machine-readable outputs must reproduce with identical SHA-256 digests. A future schema may add explicitly defined numerical-tolerance comparison, but v1 does not silently weaken exactness.

## Confirmatory firewall

A definition labelled `confirmatory` is accepted only when `study-result-binding.json` says that:

- the frozen study itself is confirmatory;
- its protocol was bound before execution;
- it remains eligible for a pre-result confirmatory claim.

An exploratory downstream analysis may reuse a confirmatory study result, but the new analysis remains labelled exploratory. Changing the analysis definition changes its content identity.

This prevents a post-hoc analysis from inheriting the word “confirmatory” merely because its input trajectories came from a confirmatory experiment.

## Scripted execution

For canonical scripted work:

```text
python scripts/research-analysis-provenance.py run \
  study/example \
  analysis-definition.json
```

Before the command executes, the wrapper SHA-256 fingerprints:

- `study-result-binding.json`;
- every declared analysis input;
- every declared implementation artifact;
- every declared environment artifact.

It then executes the exact declared command in the declared working directory.

After successful execution it fingerprints those source artifacts again. If any input, script, environment file, or study binding changed during execution, provenance publication fails closed.

All declared outputs must then exist as regular files. Their exact SHA-256 digests and sizes are bound into:

```text
study/example/analysis/analysis-provenance.json
```

The record also embeds the complete normalized analysis definition and preserves:

- `definitionIdentity`;
- `provenanceIdentity`;
- study result identity;
- study execution identity;
- frozen protocol identity and revision;
- research execution identity;
- source revision;
- scientific status;
- execution status;
- every source/output artifact path, role, digest, and size.

The provenance identity is SHA-256 over the canonical complete record with the identity field blanked. Changing analysis configuration, scripts, lockfiles, RNG seeds, observation-model identity, input data, outputs, or the upstream frozen study therefore changes or invalidates the provenance identity.

## Replay

For a result originally executed by the wrapper:

```text
python scripts/research-analysis-provenance.py replay study/example
```

Replay first verifies the canonical provenance record. It then creates an isolated temporary analysis root containing only the declared:

- frozen study-result binding;
- inputs;
- implementation files;
- environment files.

It reruns the declared command there and requires the regenerated output artifact bytes to match the canonical output digests exactly.

This is intentionally strict. If the analysis depended on an undeclared helper/input file, the isolated replay should fail so that the missing dependency becomes visible rather than remaining accidental hidden state.

The runtime/dependency environment itself must still be reconstructed from the declared environment artifacts and runtime description. A lockfile digest proves which environment specification was used; it does not magically install that environment.

## External or manual analysis

Sometimes a canonical result cannot be produced entirely through this wrapper. Use:

```text
python scripts/research-analysis-provenance.py capture \
  study/example \
  manual-analysis-definition.json
```

`executionMode` must be `external_or_manual`, `command` must be empty, and `manualSteps` must explicitly describe the external/manual transformations.

The result receives the same input/code/environment/output digests, but its record says:

```text
executionStatus = captured_external_or_manual
```

Such a result must not be described as wrapper-reexecuted. This is preferable to allowing an unexplained spreadsheet/notebook edit to become a canonical result with no visible lineage.

## Verification

```text
python scripts/research-analysis-provenance.py verify study/example
```

Verification recomputes:

- the embedded definition identity;
- the complete provenance identity;
- current `study-result-binding.json`;
- every declared input digest;
- every implementation digest;
- every environment digest;
- every output digest.

Any mutation fails verification.

## Final archive integrity

Analysis provenance records execution lineage. The existing research-integrity layer provides the final archive-wide SHA-256 seal:

```text
python scripts/research-integrity.py create study/example
python scripts/research-integrity.py verify study/example
```

The intended chain for a canonical quantitative result is therefore:

```text
frozen StudyProtocol (#230)
    -> immutable anthrosim-research execution (#205)
    -> declared analysis window where applicable (#219)
    -> executable downstream analysis definition
    -> analysis-provenance.json (#232)
    -> archive-wide integrity-manifest.json
```

Each layer answers a different reproducibility question and none substitutes for the others.

## Synthetic end-to-end verification

`scripts/test-research-analysis-provenance.py` constructs an isolated synthetic study with a frozen study-result binding and immutable derived run table. It then:

1. executes a real downstream Python analysis through `run`;
2. verifies study/input/code/environment/output digests;
3. replays the analysis in an isolated temporary root and requires exact output bytes;
4. rejects output tampering;
5. rejects source mutation during execution;
6. rejects confirmatory analysis against an ineligible frozen study;
7. proves analysis-definition changes change provenance identity;
8. verifies explicitly labelled external/manual capture;
9. rejects provenance-record tampering;
10. seals the complete synthetic study with `research-integrity.py` and proves later output mutation breaks archive verification.

A Rust integration-test wrapper executes this Python suite in the normal workspace CI.

## Scientific interpretation

A valid v1 provenance record establishes that a named output is cryptographically linked to one frozen study result and one declared executable analysis lineage.

It does not establish that the statistical method is scientifically appropriate, sufficiently powered, identifiable, unbiased, or archaeologically valid. Those remain separate research questions addressed by issues such as #209, #217, #229, and #231.

The purpose of #232 is narrower and essential: a reader should never have to guess which script, configuration, environment, random seed, filtering input, or manual transformation actually produced the reported number.
