# AV3-006 independent-arm Monte Carlo repair

This note records the post-audit-v3 repair rationale for #410 / implementation task #443.

The `difference_in_means` precision method is valid only for genuinely independent Monte Carlo arms. The repaired plan contract therefore requires two separately predeclared `groupSeedBatches` schedules whose seed identities are disjoint and whose samples are evaluated at the same declared batch boundary. A shared or overlapping seed identity fails closed before the independent two-sample variance estimator is used.

Same-seed contrasts remain represented by `paired_mean_difference`, which computes uncertainty from per-seed differences and therefore retains the observed cross-arm covariance. In the AV3-006 anti-correlated 20-seed adversary the paired 95% half-width is approximately `5.185577281736`, so a `4.5` precision threshold is insufficient; the previously reported independent half-width was approximately `3.666756860283`.

This is analysis-layer statistical semantics only. It does not change simulator causal dynamics or `MODEL_SEMANTICS_ID`, which remains v22 during this repair.
