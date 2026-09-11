# Exploratory model-characterisation experiments

**Status:** exploratory / non-validating  
**Evidence role:** model understanding, hypothesis generation and future sensitivity-test design  
**Not:** archaeological validation, empirical calibration, a canonical regression benchmark, or evidence for a historical claim

This directory records deliberately exploratory experiments used to learn how AnthroSim behaves under its current declared synthetic assumptions.

The purpose is to answer questions such as:

- which mechanisms dominate outcomes inside the model?
- where do qualitative regime changes or thresholds appear?
- which parameters or assumptions deserve targeted sensitivity analysis?
- do apparently reasonable mechanisms create unexpectedly large effects?
- which behaviours may later deserve formal benchmark, calibration, validation or adversarial treatment?

These records must use conditional model language. A result such as "migration buffered resource scarcity" means only that the implemented migration mechanism did so under the stated configuration and model semantics. It is not evidence that a real prehistoric population behaved the same way.

Exploratory numerical results are **not automatically regression expectations**. Future scientific repairs may legitimately change them. Promotion into a canonical benchmark requires a separate explicit decision, a justified oracle, reproducible checked-in execution definition/artifacts, and the relevant scientific gates.

## Register

| Date | Experiment | Model target | Status |
|---|---|---|---|
| 2026-09-11 | [Local migration as a resource-scarcity buffer](2026-09-11-migration-resource-buffer.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [Where does the migration buffer fail?](2026-09-11-migration-buffer-transition.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [How does migration search radius affect survival?](2026-09-11-migration-search-radius.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [Does travel-condition cost create the search-radius optimum?](2026-09-11-migration-travel-cost-sensitivity.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [How does migration decision frequency affect survival?](2026-09-11-migration-decision-frequency.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [Do M3 and M4 perform best when their clocks align?](2026-09-11-m3-m4-clock-alignment.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [What happens when migration opportunity frequency is separated from planning horizon?](2026-09-11-migration-fixed-planning-horizon.md) | v37-derived explicit counterfactual intervention from commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [How sensitive is M3 timing with permanent migration disabled?](2026-09-11-m3-only-timing.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [Does M3 remain timing-stable at moderate productivity?](2026-09-11-m3-only-moderate-timing.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |
| 2026-09-11 | [Does removing seasonality remove the P12 timing advantage?](2026-09-11-m3-only-moderate-no-seasonality.md) | `anthrosim-model-semantics-v37`, commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` | exploratory complete |

## Provenance note

The first three records were executed from the CI-built release binary produced by the code-changing `main` run for commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` (`anthrosim-model-semantics-v37`). The fourth, fifth, sixth, eighth, ninth and tenth required configuration dimensions or observability that the ordinary CLI does not expose directly, so they used temporary compact experimental runners that checked out the same exact v37 core source commit and called the unchanged core simulation path while retaining only the measurements needed for each sensitivity question.

The seventh is deliberately different: it is a **counterfactual intervention** that changes one M4 model rule in order to separate decision opportunity frequency from the resource-support planning horizon. Its experimental build uses a distinct semantics identity and must not be represented as authoritative v37 behaviour. The record documents the intervention and its exact v37 base explicitly.

Experiments 1–6 and 8–10 therefore characterize historical v37 model semantics. Experiment 7 diagnoses a causal coupling by comparing v37 against a controlled v37-derived alternative. None characterizes whatever semantics are currently at repository `main` when this document is read unless `main` happens to match the cited source.

The exploratory outputs have not been promoted into repository-authoritative study bundles. The records preserve the experiment designs, integrity checks and reported aggregate results. If any result becomes claim-driving, it should be rerun from a checked-in experiment definition with repository-authoritative provenance and artifacts rather than relying on these summaries alone.
