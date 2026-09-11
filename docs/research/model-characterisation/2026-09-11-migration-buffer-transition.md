# Exploratory model-characterisation: where does the migration buffer fail?

**Date:** 2026-09-11  
**Status:** exploratory complete  
**Evidence role:** model characterisation only  
**Model target:** AnthroSim `0.3.6`, `anthrosim-model-semantics-v37`  
**Executable source commit:** `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`  
**Empirical claim status:** none

## Question

How poor can the synthetic resource environment become before bounded local household migration can no longer compensate for it?

The experiment followed directly from the first migration/resource-buffer result. Rather than asking only whether migration helps, it searched for the transition between three broad model regimes:

1. migration largely maintains the population;
2. migration still helps, but substantial long-run contraction occurs;
3. even migration is no longer sufficient to prevent frequent extinction.

## Design

The same broad 100-year synthetic setup was retained while productivity scale was progressively reduced. Replication used 30 runs at the reported points, and low-resource comparison arms were also run with permanent migration disabled. The complete exploratory sweep comprised **1,320 simulations**.

Productivity scale is an abstract multiplier on AnthroSim's synthetic resource productivity. For readability, this record also expresses it as a percentage of the synthetic `1000` baseline. It is not an empirical percentage of real ecological productivity.

## Migration-enabled results

Mean living population after 100 years and observed extinction counts at the reported points:

| Productivity (% of synthetic baseline) | Mean final living population | Extinct runs / 30 | Plain-language regime |
|---:|---:|---:|---|
| 100% | 707 | 0 | baseline-like survival |
| 50% | 688 | 0 | little demographic effect |
| 30% | 663 | 0 | migration still compensates strongly |
| 25% | 652 | 0 | migration still compensates strongly |
| 20% | 615 | 0 | clear contraction begins |
| 15% | 523 | 0 | substantial contraction |
| 12.5% | 415 | 0 | serious contraction |
| 10% | 280 | 0 | severe contraction |
| 7.5% | 129 | 0 | near-collapse regime |
| 5% | 19 | 0 | very small surviving populations |
| 4.5% | 9 | 1 | extinction begins to appear |
| 4% | 4 | 5 | extinction increasingly common |
| 3.5% | 2 | 10 | one-third extinct |
| 3% | 0.3 | 22 | extinction dominant |
| 2.5% | approximately 0 | 29 | near-universal extinction |
| 0% | 0 | 30 | universal extinction |

The result therefore does not support one exact "failure threshold". Instead it shows a broad transition zone.

### Practical transition 1: loss of population stability

Around **20–25%** of the synthetic baseline, migration ceases to keep the final population close to the higher-productivity result. Below this region, long-run demographic contraction becomes progressively stronger.

### Practical transition 2: frequent extinction

Frequent extinction appears much lower, around **3–4.5%** of the synthetic baseline in this setup. The extinction transition is gradual rather than a hard deterministic boundary:

- 4.5%: `1/30` extinct;
- 4%: `5/30` extinct;
- 3.5%: `10/30` extinct;
- 3%: `22/30` extinct;
- 2.5%: `29/30` extinct.

## Comparison with migration disabled

In the low-resource comparison sweep, all `30/30` migration-disabled populations became extinct within 100 years from `0%` through `27.5%` of the synthetic productivity baseline. At `30%`, `29/30` migration-disabled runs became extinct.

At the same `30%` setting, migration-enabled runs retained a mean final living population of approximately **663**.

This demonstrates that permanent migration is not merely a small modifier under this synthetic configuration; it is one of the dominant causal mechanisms controlling long-run survival across these settings.

## Movement intensity

Reported mean completed-household-move counts also showed a non-monotonic pattern:

| Productivity | Mean household moves over 100 years |
|---:|---:|
| 100% | ~355 |
| 30% | ~5,822 |
| 20% | ~10,885 |
| 15% | ~15,731 |
| 10% | ~21,887 |
| 7.5% | ~22,515 |
| 5% | ~16,709 |
| 2.5% | ~9,178 |
| 0% | 0 |

Within the model, worsening resource conditions initially increase relocation pressure and therefore mobility. At extreme scarcity, movement declines again because useful alternatives disappear and populations become very small. At zero productivity, there is no resource-based better destination to discover.

This is an emergent model pattern under the current rules, not evidence for a historical mobility curve.

## Interpretation

A concise conditional statement supported by this experiment is:

> Under the stated synthetic v37 assumptions, bounded local permanent migration acts as a strong buffer against spatial resource scarcity, but the buffer progressively fails when useful nearby resource differences become insufficient; population contraction becomes clear well before extinction becomes common.

The experiment also distinguishes two concepts that should not be collapsed into one "threshold":

- **loss of stability:** migration no longer keeps population near the high-resource outcome;
- **loss of persistence:** migration no longer reliably prevents extinction over the 100-year horizon.

## Model-development implications

The strength of the observed effect means future work should test which assumptions generate it. High-priority follow-up axes include:

- migration information/search radius;
- migration decision frequency;
- minimum utility improvement required to move;
- travel-condition cost;
- relocation-risk penalties;
- resource weighting in destination utility;
- spatial heterogeneity of productivity;
- run duration and initial population;
- interactions between scarcity, condition and mortality.

A useful next experiment is to vary the migration search radius (`3 -> 2 -> 1 -> disabled`) while holding other assumptions fixed. This would test whether the strong survival benefit arises smoothly with access to more nearby alternatives or depends disproportionately on the current radius-three information horizon.

## Limitations and preservation rule

This sweep characterizes one historical executable, `anthrosim-model-semantics-v37` at commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30`. It should not be silently generalized to later semantics.

The conversational exploratory runtime directories were not retained as repository-authoritative study bundles. The aggregate results above are therefore a model-characterisation record rather than a canonical reproducibility package. Any result promoted into a scientific claim, formal benchmark or regression oracle must first be independently rerun under a checked-in experiment definition with authoritative provenance, complete outputs and the relevant statistical/scientific gates.
