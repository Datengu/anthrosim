# Model-semantics v29 reverification

Audit-v4 AV4-005 / issue #495 removes arbitrary canonical male `PersonId`/record ordering from M2 parentage RNG assignment. Residence-local living male candidates are now ordered by persistent person stochastic-coupling rank before the existing eligibility filter and uniform `demography/parentage` reservoir sample. Locality, age/sex eligibility, stream separation and uniform selection are unchanged.

Because this changes which represented kin role receives a same-seed parentage realization, the current remediation line advances from `anthrosim-model-semantics-v28` to `anthrosim-model-semantics-v29`; checkpoint schema advances from 16 to 17.

## Permanent regression

`crates/anthrosim-core/tests/parentage_label_invariance.rs` covers the original-style 1,000-seed genealogy-preserving male-label swap, a 256-seed three-role cyclic relabel, and a 256-seed two-year propagation check. The pinned Rust 1.97.1 production regression passed all three before final-candidate assembly.

## Frozen scientific surfaces reviewed

### Issue #304 demographic confirmation

The unchanged 3 × 2 × 130 design completed all **780** runs under v29 and all three predeclared precision gates returned `sufficient_stop`. The recommendation remains `no_universal_demographic_baseline`. Reviewed run `33813558679`, job `100840609676`, artifact `9915805924`, SHA-256 `607dbdf2e86db582fe7b519c1bf9ea1ad8d69ba02ffc282f934c4f5d4240d45c`. The checked-in current research identity is `research-execution-v1-e66b1372b97e7faf`.

### M8.6 terrain null model

Reviewed applicable-gates run `33813559006`, job `100840645788`, artifact `9915797546`, SHA-256 `27cf02539a53a4a21e2cd13e223f70ae00166ecc6972441500447d9248f52ef3`. The overall class remains `fragile_spatial_structure`; both terminal Herfindahl and terminal largest-cell share are fragile under v29. Aggregate canonical SHA-256: `978ed2342509d9cbca1a647055f1d794ba513bcb1fdaee01fd26f5e6c7ed4b44`.

### M9.7 controlled aggregation

Reviewed applicable-gates run `33813559006`, job `100840645688`, artifact `9915799402`, SHA-256 `8e14622e26728c6e4a300c6c834c6085aa9c2e84013704f237e4aaa4a1221a4c`. All 8/8 paired criteria, exact replay and active checkpoint/resume remain green and the class remains `capability_distinguished`; authoritative terminal state digests change under v29. Aggregate canonical SHA-256: `be17795b0ed35aba0c39a6c76b1d45934dd165d75199551464dcbdc589c9294b`.

These are reviewed upstream-semantics rebaselines, not new empirical validation. Historical v25–v28 results remain bound to their original semantics in Git history and the living result documents.
