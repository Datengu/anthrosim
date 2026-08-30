# Declared founder population CLI v1

**Status:** user-facing execution path for the M2 founder initialization contract  
**Normative semantics:** [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md)

## Purpose

The core `declared_founder_state_v1` mode can be supplied to an ordinary AnthroSim run without modifying Rust code.

The CLI intentionally accepts the complete versioned founder definition as JSON rather than adding separate command-line switches for ages, parent links or pre-run birth histories. Those values belong together as one auditable initial-state definition.

## Example

A small structurally valid synthetic example is stored at:

```text
examples/founder-population-declared-v1.json
```

It is an interface/example fixture only. Its values are **not** an empirical demographic recommendation.

From the repository root:

```text
cargo run -p anthrosim-cli -- run --founder-population examples/founder-population-declared-v1.json --world-width 1 --world-height 1 --years 1 --run-dir output/founder-example
```

The same option can be combined with the ordinary single-run controls for resources, migration, temporary mobility, checkpoints and outputs.

## CLI binding rules

When `--founder-population PATH` is omitted, `anthrosim run` keeps the existing `synthetic_validation_v1` behavior.

When it is supplied:

1. the JSON is deserialized as `FounderPopulationDefinition`;
2. the configured initial population count is derived from `people.length`, rather than from the default/supplied synthetic `--population` value;
3. the complete definition is attached to immutable `ExperimentConfig` identity;
4. `ExperimentConfig::with_founder_population` selects `declared_founder_state_v1`;
5. core validates schema, chronology, IDs, households, world locations, parent links, condition, record limits and active-kin genealogy completeness;
6. the initialized population written to `initial-population.json` or `--population-output` is the exact materialized declared state.

Synthetic-only founder-generation settings such as `--household-size` do not alter a declared founder definition.

## Fail-closed behavior

A malformed declaration or a declaration incompatible with the selected world fails before the run begins.

If permanent migration is enabled with non-zero kin weighting and the declaration says founder genealogy is `unspecified`, construction fails rather than treating unknown direct-parent links as evidence of no kin.

## Manifest and checkpoint provenance

The run manifest/checkpoint embeds the complete `founderPopulation` object inside the experiment configuration. A later checkpoint resume therefore carries the same pre-run founder identity/history without requiring the original input file to be supplied again.

The input file itself should still be retained with the study materials because file-level provenance and the scientific derivation procedure matter independently of the serialized values.

## Current orchestration boundary

This first research-facing pathway is available on the ordinary single-run `anthrosim run` command.

`ensemble` and `sweep` still expose the existing synthetic founder-population dimensions in this repair. This is an orchestration limitation, not a core scientific fallback: a declared founder definition cannot silently enter those commands through the synthetic initializer.

Before a real study relies on large declared-founder ensembles/sweeps, orchestration should gain an explicit immutable founder-definition binding and define how founder-state uncertainty is varied. That future work must distinguish:

- stochastic seeds with one fixed founder state;
- alternative plausible founder states representing epistemic initialization uncertainty;
- any generated/burn-in founder-state procedure.

## Research-use warning

The ability to load a declaration does not establish that its contents are defensible. A research application must document the evidence/generation procedure and test sensitivity to plausible alternatives as required by TRACE.


## Reproductive chronology validation

Founder parent ages and declared pre-run `lastBirthDay` are validated against the experiment's `DemographyConfig` before execution. Female events require positive fertility-band support at the declared event age; male parentage uses the configured male-parent age interval. These are experiment-declared assumptions, not universal anthropological constants.
