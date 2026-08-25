# M2 demography: evidence, assumptions and parameter provenance

Status: research/evidence baseline for M2. The executable timing semantics are governed by [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md), founder-state semantics by [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md), and demographic analysis semantics by [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md).

This document does **not** claim that one demographic schedule represents prehistoric *Homo sapiens*. It records the comparative evidence, modelling boundaries and validation quantities that should constrain future research parameterizations.

## 1. Purpose

M2 introduces persistent people, ageing, births, deaths, parentage and co-residence. Demography is scientifically consequential: small changes to mortality, fertility, birth spacing, initial age structure or parent availability can alter population size, survivorship, age structure, genealogy, resource demand and later mobility.

The objective is therefore not to choose one supposedly realistic hunter-gatherer profile. AnthroSim should support documented demographic regimes while keeping empirical evidence, executable model semantics and synthetic engineering defaults visibly separate.

## 2. Core scientific rule

**Extant hunter-gatherers are comparative evidence, not direct stand-ins for Palaeolithic populations.**

Sear, Lawson and Kaplan (2020) warn against treating one living forager population as a proxy for prehistoric demography and emphasize substantial within- and between-population diversity.

Implications for AnthroSim:

- no generic preset should be labelled `prehistoric`, `ancestral` or `natural`;
- population-specific schedules must retain source/provenance identity;
- engine-validation values remain synthetic even when selected inside evidence-informed ranges;
- uncertainty should be explored through ensembles and sensitivity analysis rather than collapsed to one precise value;
- archaeological/palaeodemographic evidence is required for claims about a particular prehistoric context.

## 3. Comparative evidence orientation

### 3.1 Mortality and longevity

Gurven and Kaplan (2007) document both broadly human mortality structure and substantial variation among small-scale populations. Their traditional hunter-gatherer samples include roughly:

- life expectancy at birth around **21–37 years**;
- survival from birth to age 45 around **26%–43%**;
- about **64% average survival from age 15 to age 45**;
- roughly **14–24 additional years of life expectancy conditional on reaching age 45**.

The implementation lesson is important: low life expectancy at birth does not imply that most adults die in their thirties. High early mortality can coexist with substantial survival into later adulthood.

### 3.2 Fertility varies substantially

Blurton Jones et al. (1992) report a Hadza regime with materially higher fertility than the !Kung/Ju/'hoansi comparison, including total fertility around **6.15 births per woman** versus about **4.7** in that comparison and a mean age of childbearing around **30.9 years** in the Hadza sample.

A later comparative synthesis (Ramirez Rozzi et al. 2018 and underlying sources) illustrates the range rather than prescribing inputs:

| Population/sample | Completed family size | Interbirth interval | Age at first birth |
| --- | ---: | ---: | ---: |
| Ache forest | 8.15 | 3.1 y | 19.5 y |
| Hiwi | 5.13 | 3.76 y | 20.5 y |
| Ju/'hoansi | 4.7 | 4.12 y | 18.8 y |
| Agta | 7.6 | 2.82–2.85 y | 19.5 y |
| Hadza | — | — | ~19 y |
| Baka | 7.3 | 2.77 y | 18 y |

These quantities are validation/orientation evidence, not a universal parameter set.

### 3.3 Birth spacing is mechanistic

Konner and Worthman (1980) link nursing frequency, reproductive physiology and birth spacing among !Kung, illustrating that interbirth intervals emerge from interacting physiology and behaviour rather than from a culturally universal timer.

AnthroSim does not currently model endocrine physiology. Birth spacing is therefore an explicit simplified scheduling constraint whose executable meaning must be stated honestly.

### 3.4 Ecology and mode of life can alter demography

Colleran et al. (2016) report associations between settlement/agricultural involvement, fertility, health and child mortality among Agta. The broader lesson is that fertility and mortality are not fixed species constants.

AnthroSim should not hide ecological effects inside opaque age schedules. When resource, disease, workload or environmental mechanisms modify demographic outcomes, those links should be explicit, evidence-bearing and independently testable.

### 3.5 Short-run growth cannot be extrapolated indefinitely

Published small-scale demographic windows can imply positive annual intrinsic growth rates that would produce impossible population explosions if held constant for millennia. Long-run regulation can involve resource limits, migration, disease, conflict, shocks, changing fertility and sampling-period effects.

AnthroSim must therefore not tune a demographic schedule merely until a very long run “looks right”. Regulation should arise from declared mechanisms or experimental constraints.

### 3.6 Sex ratio at live birth

Large human datasets generally show a modest male excess near 51%. A synthetic baseline around `0.512` male births is reasonable for engine validation, but it is not a deep-prehistory-specific estimate and should remain sensitivity-testable.

## 4. Current executable M2 semantics

The current model is an **annual discrete-time demographic transition**, not a continuous-time hazard model.

For an annual boundary at day `t`, M2 represents the elapsed interval `[t-365, t)`.

- Mortality and fertility age bands are selected using age at the **start** of that interval.
- Mortality is drawn first.
- Fertility probability is a conditional annual live-birth opportunity among females that survive that M2 mortality transition, then pass spacing and local-male eligibility.
- Ignoring spacing and male availability, the current contract is `P(birth at t) = (1-q) * p`, where `q` is the annual demographic mortality probability and `p` the conditional fertility probability.
- No within-year birth or demographic-death timestamp is sampled.
- A model-born child is exposed to the age-0 mortality band over its first elapsed year.

These are declared coarse model semantics, not statements about real within-year biological event ordering. An empirical probability table must be transformed/estimated consistently with this conditioning before use.

## 5. Executable birth spacing

`minimum_birth_spacing_days` is a requested lower bound, but births currently occur only at annual M2 boundaries. The executable lower bound is therefore:

`effective_spacing_days = ceil(requested_days / 365) * 365`

with zero mapping to zero.

Examples:

| Requested | Executable |
| ---: | ---: |
| 0 d | 0 d |
| 365 d | 365 d |
| 366 d | 730 d |
| 730 d | 730 d |
| 731 d | 1095 d |
| 1278 d (~3.5 y) | 1460 d (4 y) |
| 1460 d | 1460 d |

The old evidence-oriented **3.5-year** suggestion must therefore not be described as an exactly executable current-model spacing. The synthetic baseline may request `1278` days, but the run-facing observability report records that its current annual scheduler executes `1460` days.

If a study needs genuinely subannual interbirth timing, M2 requires a new event-time/subannual design rather than pretending that the current annual scheduler provides that precision.

## 6. Parentage locality and initialization

At an annual boundary shared with M4 relocation, M2 parentage locality uses persistent residence immediately **before** the same-day M4 move. A destination occupied for zero elapsed days cannot create a parentage relationship for the preceding annual interval. The newborn nevertheless joins the female parent's current boundary-state household/residence after M4.

M9 visitor/transit presence does not alter M2 parent eligibility.

Founder state is also scientifically consequential. AnthroSim now distinguishes:

1. `synthetic_validation_v1` — a deterministic engineering/null-model founder generator with no claim to realistic prehistory;
2. `declared_founder_state_v1` — an explicit versioned founder population with signed birth chronology, residence/household, condition, optional pre-run last-birth timing and optional living direct-parent links.

Declared founder history removes the requirement that research-facing reproductive/kin state begin implicitly at zero. It does **not** create a stable population automatically; the derivation of a founder declaration and initialization/burn-in sensitivity remain study-specific obligations.

## 7. Evidence-informed engineering envelope

The following ranges are orientation for testing and future parameterization, not a prehistoric preset:

| Quantity | Comparative/engineering orientation | Current-model note |
| --- | --- | --- |
| Age at first reproductive exposure | ~16–22 y | executable through age schedule |
| Effective end of female reproduction | ~40–50 y | executable through declining/zero schedule |
| Interbirth spacing | ~2.7–4.2 y | current annual scheduler quantizes to whole 365-day boundaries |
| Completed fertility | roughly ~4–8 births in several comparative samples | output/validation quantity, not forced target |
| Life expectancy at birth | roughly ~21–37 y in cited traditional-HG samples | output/validation quantity |
| Survival to age 45 from birth | roughly ~26%–43% | output/validation quantity |
| Additional life expectancy at 45 | roughly ~14–24 y | output/validation quantity |
| Male fraction at live birth | ~0.51 | synthetic baseline ~0.512 |
| Long-run population growth | contextual/uncertain | no universal target |

A publishable experiment should identify a specific empirical calibration/validation target or deliberately sample across a justified range.

## 8. Mortality and fertility representation

For the current model, prefer transparent **piecewise age-specific annual transition-probability schedules** over fitted equations whose coefficients would otherwise be guessed.

This makes schedule assumptions inspectable and allows published life-table information to be transformed into explicit executable inputs when a defensible workflow exists.

The expected qualitative mortality shape for synthetic validation is:

- high early-life mortality;
- declining mortality through later childhood;
- lower young-adult baseline risk;
- increasing senescent mortality later in adulthood;
- no deterministic old-age cutoff.

Fertility is represented as an age-specific **conditional annual live-birth opportunity** plus explicit spacing and local male eligibility. M2 does not assign a predetermined completed family size.

The baseline intentionally avoids universal marriage, monogamy/polygyny, culturally specific mate-selection institutions, social gender rules, or fertility effects from condition/resources unless those links are explicitly introduced and justified.

## 9. Demographic observability and validation

A total birth count is not sufficient to validate fertility. Realized births pass through mortality, age, spacing, locality/male availability, stochastic draw and operational record-limit filters.

`anthrosim-demography-observability` derives a version-bound report from authoritative initial state, checkpoint/event history and the independent fertility RNG stream. It exposes, by schedule band where appropriate:

- mortality exposures and demographic deaths;
- surviving females entering fertility;
- non-zero age-schedule eligibility;
- spacing eligibility;
- local eligible-male availability;
- fertility draws attempted;
- stochastic draw successes/failures;
- successful births;
- record-limit blocking;
- requested versus executable birth spacing;
- model-period interbirth intervals;
- declared pre-run-to-first-birth intervals;
- completed fertility with explicit censoring.

The replay reconciles against final Population state. This is a **verification/analysis surface**, not empirical validation.

For empirical/evidence-informed presets, future validation should additionally report and compare as appropriate:

- survivorship and age-specific mortality;
- life expectancy and survival to declared ages;
- age structure/dependency ratios;
- age-specific fertility and mean age at childbearing;
- interbirth interval distribution;
- completed fertility distribution;
- population growth.

Validation criteria/tolerances must be declared before interpreting success. A preset that fails its claimed empirical targets should report the failure rather than be invisibly tuned until it passes.

## 10. Sensitivity requirements

Demographic analysis should vary at least:

- infant/child mortality;
- young-adult mortality;
- late-age mortality schedule;
- reproductive onset/end;
- fertility magnitude;
- requested/effective spacing;
- sex ratio at birth;
- founder age structure;
- founder reproductive history/genealogy where relevant;
- structural assumptions such as annual versus future subannual demographic timing.

Results that disappear under small plausible changes must be described as parameter- or structure-sensitive rather than robust.

## 11. Provenance categories

Every demographic input/preset should retain a declared evidence status, for example:

- **EMPIRICAL_DIRECT** — taken directly from a cited dataset/table;
- **EMPIRICAL_DERIVED** — calculated from cited empirical data with a documented transformation;
- **EVIDENCE_INFORMED** — selected within a cited range but not itself directly observed;
- **SYNTHETIC_VALIDATION** — deliberately artificial for software/model testing;
- **UNRESOLVED** — required but not yet defensibly parameterized.

A value must never silently move from synthetic validation to empirical claim status.

## 12. Known limitations

- extant forager samples have contact histories, disease, displacement, state pressure, trade and ecological change;
- published samples are not representative of every hunter-gatherer population;
- small samples, age estimation and retrospective reproductive histories introduce uncertainty;
- fertility, mortality, mobility and ecology are causally interdependent;
- deep-prehistoric regimes may combine processes not observed in ethnographic datasets;
- archaeological/palaeodemographic evidence constrains different quantities and has preservation/sampling biases;
- current M2 is temporally coarse and cannot express within-year demographic event timing;
- current parentage is a minimal local eligibility model rather than a kinship/mating institution;
- the synthetic founder mode is not a stable/quasi-stable demographic initializer;
- demographic observability verifies/exposes the model's opportunity structure but does not itself demonstrate empirical validity.

## 13. Sources retained in the evidence set

1. Sear R, Lawson DW, Kaplan H. 2020. *Reconstructing prehistoric demography: What role for extant hunter-gatherers?* DOI `10.1002/evan.21869`.
2. Gurven M, Kaplan H. 2007. *Longevity Among Hunter-Gatherers: A Cross-Cultural Examination.* DOI `10.1111/j.1728-4457.2007.00171.x`.
3. Blurton Jones NG et al. 1992. *Demography of the Hadza, an increasing and high density population of Savanna foragers.* DOI `10.1002/ajpa.1330890204`.
4. Konner M, Worthman C. 1980. *Nursing frequency, gonadal function, and birth spacing among !Kung hunter-gatherers.* DOI `10.1126/science.7352291`.
5. Colleran H et al. 2016. *Reproductive trade-offs in extant hunter-gatherers suggest adaptive mechanism for the Neolithic expansion.* DOI `10.1073/pnas.1524031113`.
6. Ramirez Rozzi FV et al. 2018. *Reproduction in the Baka pygmies and drop in their fertility with the arrival of alcohol.* Comparative reproductive table and underlying sources.
7. *Periodic catastrophes over human evolutionary history are necessary to explain the forager population paradox.* 2019. Comparative vital rates/long-run growth problem.
8. *Sex ratio is remarkably constant.* PMID `19159875`.

## 14. Current implementation decision

AnthroSim retains a **schedule-driven demographic engine** and a clearly named `synthetic_validation_v1` preset. The engine is intentionally replaceable at the schedule/input layer, while its current annual timing/competition semantics remain explicit and versioned.

The architecture should permit later empirical schedules or alternative demographic-time models without disguising a change of scientific meaning as a parameter tweak. Verification, validation, sensitivity and evidence provenance remain separate obligations.
