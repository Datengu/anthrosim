# AV4-002 mortality person-relabel evidence

Frozen target: `v0.3.4` / `8996e99ffc4c5b91b9e00d1048eedd4227ea1d09` / `anthrosim-model-semantics-v25`.

Evidence PR: #487. Dedicated workflow run: `33689235132`. Evidence head: `c6f48d587fa7e469c97c4594c0e92d46176a004d`.

The fresh adversary exchanged only the canonical `PersonId` labels attached to two otherwise equivalent 30-year-old male founders living in fixed one-person households at `CellId(1)` and `CellId(2)`. Both arms used the same seed, 500,000-per-million background mortality, zero fertility, zero resource need, and disabled migration.

At seed 1 the same-seed mortality realization changed spatial attribution solely under the label permutation:

```text
A=[CellId(1)]
B=[CellId(2)]
```

The pinned Rust 1.97.1 build succeeded; the dedicated test then failed at the intended scientific equality assertion. This document preserves evidence only. Audit-v4 discovery remains in progress and no repair is authorized before the A–N discovery pass is complete.
