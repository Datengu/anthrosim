# Audit v4 Area A — fertility label-order adversary

Target: immutable `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / `anthrosim-model-semantics-v25`.

## Adversarial hypothesis

Canonical person labels are bookkeeping identity, not a causal fertility or spatial-location variable. Two founder populations that are identical after erasing those person labels should therefore not attach the same-seed fertility draws to different fixed cells merely because the household-local pairs receive different canonical IDs.

## Construction

Evidence PR #485 constructs two declared-founder arms with:

- the same two households at cells 1 and 2;
- one 30-year-old female and one 30-year-old male in each household;
- mortality disabled;
- fertility probability 500,000 per million;
- zero birth spacing;
- zero resource need;
- permanent migration disabled;
- one-year horizon;
- identical seed;
- the only scientific-state transformation is exchanging the canonical person labels assigned to the two otherwise equivalent household-local female/male pairs.

The adversary is specified for seeds 1..1000 but fails immediately at seed 1.

## Exact observed failure

Workflow run `33687262609`, evidence head `3168cd5547952c8eb2ae715447252785584bb84e`:

```text
assertion `left == right` failed: scientifically identical unlabeled founder states diverged under person-label permutation at seed 1: A=[CellId(1)], B=[CellId(2)]
  left: [CellId(1)]
 right: [CellId(2)]
```

The test compiled successfully under the pinned Rust 1.97.1 toolchain; the failure is the intended scientific assertion, not a build/setup failure.

## Initial interpretation

The annual M2 fertility loop consumes a shared sequential fertility RNG while iterating canonical population records. A pure label permutation can therefore assign the same random fertility realization to a different fixed spatial household. The total number of first-year births is unchanged in this minimal construction, but their causal spatial attribution changes. Since newborn residence feeds later resource demand, household structure, migration and other spatial outputs, the arbitrary label can propagate into scientific outcomes.

This is a demonstrated Audit-v4 finding candidate. Preserve it before any production repair; Audit-v4 discovery continues against immutable v0.3.4/v25.
