# Area C — v21 household-fission confirmatory precision handoff

Issue: #324

The first post-repair #304 confirmation used the historical fixed 64-seed design (`3041001..3041064`). All 384 simulations completed under `anthrosim-model-semantics-v21`, but the predeclared positive-growth dependency-aware-fission extinction-probability precision gate failed closed.

Observed diagnostic from workflow run `33274098216`, artifact `9721002863`:

- replicates: 64;
- extinction estimate: `4/64 = 0.0625`;
- method: Wilson score probability interval;
- 95% interval: `[0.024571201396618017, 0.14997485092208662]`;
- realized declared-style half-width: `0.08747485092208662`;
- predeclared maximum half-width: `0.085`;
- decision: `insufficient_no_predeclared_additional_batch`.

The old fixed plan therefore provides no scientific permission to append seeds. Its insufficiency is preserved rather than weakened or retroactively redefined.

A new v2 confirmation was consequently predeclared with an entirely fresh fixed seed set `3042001..3042130`. The count `n=130` was selected before inspecting any outcomes from those seeds. It is the minimum integer count for which the two-sided 95% Wilson probability interval has maximum half-width no greater than `0.085` even at the worst-case Bernoulli probability `p=0.5`; the selection therefore does not depend on the observed `4/64` extinction rate.

The exact three v2 precision plans are checked into `research/general-demography-baseline-v1/precision-v2/` and validated before simulation execution. The #304 workflow must fail closed if their identities, fixed seed set, or thresholds are changed.

This revision changes stochastic-analysis design only. It does not alter AnthroSim causal model semantics beyond the already-declared v21 household-fission repair.
