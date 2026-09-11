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

## Provenance note

The first three records were executed from the CI-built release binary produced by the code-changing `main` run for commit `9f0914e2772cd2f0c6243ab4be8b296f3ce48f30` (`anthrosim-model-semantics-v37`). The fourth and fifth required varying migration parameters that the ordinary CLI does not expose, so they used temporary compact experimental runners that checked out the same exact v37 core source commit and called the unchanged core simulation path while retaining only the measurements needed for each sensitivity question. Each record describes its execution boundary explicitly.

These experiments therefore characterize that historical model semantics, not whatever semantics are currently at repository `main` when this document is read.

The exploratory outputs have not been promoted into repository-authoritative study bundles. The records preserve the experiment designs, integrity checks and reported aggregate results. If any result becomes claim-driving, it should be rerun from a checked-in experiment definition with repository-authoritative provenance and artifacts rather than relying on these summaries alone.
