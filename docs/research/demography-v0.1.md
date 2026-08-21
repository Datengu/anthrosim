# v0.1 Demography: Evidence, Assumptions and Parameter Provenance

Status: research baseline for M2. This document constrains implementation; it does **not** claim that a single demographic schedule represents prehistoric *Homo sapiens*.

## 1. Purpose

M2 introduces persistent people, ageing, births, deaths, parentage and co-residence. Demography is scientifically consequential: a small change to fertility or mortality can dominate population size, age structure, kin availability, migration pressure and later cultural transmission.

The v0.1 objective is therefore not to choose one supposedly "realistic hunter-gatherer" demographic profile. It is to build a demographic engine that can represent a documented range of regimes, with every empirical preset traceable to evidence and every simplifying assumption clearly labelled.

## 2. Core scientific rule

**Extant hunter-gatherers are comparative evidence, not direct stand-ins for Palaeolithic populations.**

Sear, Lawson and Kaplan (2020) explicitly warn against treating any one living forager population as a proxy for prehistoric demography. Their central recommendation is to use the substantial *within- and between-population diversity* among extant hunter-gatherers to inform plausible ranges and uncertainty.

Source: Sear R, Lawson DW, Kaplan H. 2020. *Reconstructing prehistoric demography: What role for extant hunter-gatherers?* Evolutionary Anthropology. DOI: 10.1002/evan.21869. https://pubmed.ncbi.nlm.nih.gov/33103830/

Implications for AnthroSim:

- no preset may be called `prehistoric`, `ancestral`, or `natural`;
- population-specific schedules must retain their population/source identity;
- an engine-validation baseline must be labelled synthetic even when its range is evidence-informed;
- uncertainty should be explored through ensembles and sensitivity analysis rather than collapsed into false precision;
- later claims about deep prehistory require archaeological/palaeodemographic evidence in addition to ethnographic analogues.

## 3. What the comparative evidence says

### 3.1 Mortality and longevity

Gurven and Kaplan's cross-cultural synthesis shows a broadly similar human adult mortality shape across multiple small-scale populations, while also documenting substantial differences in mortality level. Among traditional hunter-gatherer samples in their analysis:

- life expectancy at birth varies roughly **21-37 years**;
- the proportion surviving to age 45 varies roughly **26%-43%**;
- conditional on reaching age 15, about **64% on average** survive to age 45;
- life expectancy conditional on reaching age 45 is roughly **14-24 additional years** across the traditional hunter-gatherer samples;
- substantial post-reproductive survival is therefore compatible with low life expectancy at birth;
- their broader argument places characteristic adult longevity/modal adult death well beyond the misleading idea that most adults died in their thirties.

Source: Gurven M, Kaplan H. 2007. *Longevity Among Hunter-Gatherers: A Cross-Cultural Examination.* Population and Development Review 33(2):321-365. DOI: 10.1111/j.1728-4457.2007.00171.x. https://doi.org/10.1111/j.1728-4457.2007.00171.x

A useful implementation lesson follows: **life expectancy at birth must never be interpreted as an adult maximum age.** High infant/child mortality can coexist with many adults surviving into their sixties and seventies.

### 3.2 Fertility differs markedly between populations

Blurton Jones et al. found the Hadza demographic regime differed materially from the !Kung/Ju/'hoansi comparison. Their 1985 Hadza census and longitudinal checks supported higher fertility and faster population growth; the paper reports a total fertility estimate around **6.15 births per woman** compared with about **4.7** for the !Kung comparison, and a mean age of childbearing of **30.9 years** in the Hadza sample.

Source: Blurton Jones NG, Smith LC, O'Connell JF, Hawkes K, Kamuzora CL. 1992. *Demography of the Hadza, an increasing and high density population of Savanna foragers.* American Journal of Physical Anthropology 89(2):159-181. DOI: 10.1002/ajpa.1330890204. https://pubmed.ncbi.nlm.nih.gov/1443092/

Comparative reproductive summaries likewise show large variation. A later synthesis of published hunter-gatherer data reports approximate values including:

| Population/sample | Completed family size | Interbirth interval | Age at first birth |
| --- | ---: | ---: | ---: |
| Ache forest | 8.15 | 3.1 y | 19.5 y |
| Hiwi | 5.13 | 3.76 y | 20.5 y |
| Ju/'hoansi | 4.7 | 4.12 y | 18.8 y |
| Agta | 7.6 | 2.82-2.85 y | 19.5 y |
| Hadza | — | — | ~19 y |
| Baka | 7.3 | 2.77 y | 18 y |

Source table and underlying citations: Ramirez Rozzi FV et al. 2018. *Reproduction in the Baka pygmies and drop in their fertility with the arrival of alcohol.* PNAS/related open-access record. https://pmc.ncbi.nlm.nih.gov/articles/PMC6142234/

These values are **not** a parameter prescription. They demonstrate why M2 must support fertility schedules rather than one fixed birth probability.

### 3.3 Birth spacing is mechanistic, not merely a timer

Konner and Worthman's !Kung work linked frequent nursing, suppressed gonadal function and birth spacing, illustrating that interbirth intervals can arise from interacting physiology and behaviour rather than a culturally universal fixed interval.

Source: Konner M, Worthman C. 1980. *Nursing frequency, gonadal function, and birth spacing among !Kung hunter-gatherers.* Science 207(4432):788-791. DOI: 10.1126/science.7352291. https://pubmed.ncbi.nlm.nih.gov/7352291/

v0.1 will not model endocrine physiology. It should therefore represent postpartum/birth-spacing effects as an explicit simplified mechanism and label that mechanism as a modelling abstraction.

### 3.4 Demography responds to ecology and mode of life

Agta data provide a useful warning against treating fertility and mortality as fixed species constants. Colleran et al. found sedentarization/agricultural involvement associated with higher fertility while settled camps also experienced worse health and higher child mortality. In their sample, 19% of reported live births died before age one, with additional mortality between ages 1-5 and 5-15, and settlement status was associated with mortality differences.

Source: Colleran H et al. 2016. *Reproductive trade-offs in extant hunter-gatherers suggest adaptive mechanism for the Neolithic expansion.* PNAS 113(17):4694-4699. DOI: 10.1073/pnas.1524031113. https://pmc.ncbi.nlm.nih.gov/articles/PMC4855554/

For AnthroSim this means M2 should initially expose demographic schedules as configurable baseline hazards. M3 and later modules may modify those hazards through nutrition, disease, workload or environmental stress. M2 should not prematurely bake environmental effects into opaque age rules.

### 3.5 Observed short-run growth rates cannot simply be extrapolated for millennia

Comparative small-scale human life tables can imply strongly positive annual intrinsic growth rates. One synthesis reports examples from roughly **0.17%/year (Ju/'hoansi)** through **2.64%/year (Ache)**, with Hadza around **1.38%/year** in the cited baseline datasets. Sustained for millennia, such rates would produce impossible population explosions.

Source: Tallavaara/Lenski et al. discussion in *Periodic catastrophes over human evolutionary history are necessary to explain the forager population paradox.* 2019. https://pmc.ncbi.nlm.nih.gov/articles/PMC6600907/

This is a critical research-design point. An observed demographic window is not automatically a long-run equilibrium. Regulation, environmental shocks, migration, disease, conflict, changing fertility, sampling period and other processes matter.

Therefore v0.1 must **not tune the engine until a 10,000-year run merely "looks right."** Long-run population regulation should emerge from explicit mechanisms introduced in M3+ or from deliberately defined experimental constraints.

### 3.6 Sex ratio at live birth

Large human birth datasets generally find a modest male excess, commonly around **1.05 male births per female birth** (about 51.2% male / 48.8% female), while acknowledging some biological and environmental variation.

Example source: Orzack/related clinical evidence summarized in *Sex ratio is remarkably constant.* https://pubmed.ncbi.nlm.nih.gov/19159875/

For M2, `0.512 male` is a defensible neutral live-birth baseline for engine validation, but it is not treated as a deep-prehistory-specific estimate. Sensitivity runs should permit modest variation.

## 4. v0.1 implementation strategy

### 4.1 Use schedules, not magic constants

The engine should represent mortality and fertility as versioned age-specific schedules. The model code should consume schedules; it should not contain hidden anthropological constants.

Recommended shape:

- mortality hazard by age interval;
- fertility opportunity/hazard by female reproductive age interval;
- postpartum minimum/reduced-fecundity period;
- optional parity dependence deferred unless justified;
- sex ratio at live birth;
- initialization age distribution;
- explicit maximum supported age for storage/schedule bounds, not as a forced death age.

This lets future research presets reproduce a published life table without rewriting simulation logic.

### 4.2 Suggested engine-validation envelope

The following is an **evidence-informed engineering envelope**, not a prehistoric parameter set:

| Quantity | v0.1 engineering envelope | Baseline suggestion | Status |
| --- | --- | --- | --- |
| Age at first reproductive exposure | 16-22 y | 19 y | comparative envelope; culturally/biologically contingent |
| Effective end of female reproduction | ~40-50 y | declining schedule to ~45-50 y | comparative human life-history feature |
| Interbirth spacing | ~2.7-4.2 y | 3.5 y | supported by several forager datasets; mechanism simplified |
| Completed fertility/TFR orientation | ~4-8 births | **not a direct input target** | validation/output orientation only |
| Life expectancy at birth orientation | ~21-37 y | **not directly calibrated initially** | traditional-HG comparative range |
| Survival to age 45 from birth | ~26%-43% | validation target, not input | traditional-HG comparative range |
| Additional life expectancy at age 45 | ~14-24 y | validation target, not input | traditional-HG comparative range |
| Male fraction at live birth | ~0.51 | 0.512 | generic human baseline, not HG-specific |
| Long-run population growth | uncertain/contextual | no universal target | must emerge from explicit model/context |

The baseline suggestion is for software/model validation only. A publishable experiment should identify a specific empirical calibration target or deliberately sample across the envelope.

## 5. Mortality model recommendation

For M2, prefer a transparent **piecewise age-specific annual hazard table** over a more elegant fitted equation.

Reasons:

1. published life tables can later map into it directly;
2. every age-band assumption is inspectable;
3. no fitted coefficients need to be invented merely to make a Siler/Gompertz model run;
4. sensitivity analysis is straightforward;
5. the engine remains agnostic about which empirical population is being represented.

A Siler three-component hazard model is scientifically relevant and is used in comparative demographic work such as Gurven & Kaplan, but should be added only when we have a documented fitting/calibration workflow rather than guessed coefficients.

Expected qualitative mortality shape for validation:

- high mortality in infancy/early childhood;
- declining mortality through later childhood;
- relatively low young-adult baseline risk;
- accelerating senescent mortality later in adulthood;
- no deterministic "old-age cutoff" that kills everyone at the same age.

## 6. Fertility model recommendation

M2 should model **birth hazard/opportunity**, not assign every woman a predetermined completed family size.

Minimum state/mechanisms:

- reproductive sex;
- age;
- alive status;
- time since previous birth;
- age-specific fertility schedule;
- postpartum spacing suppression;
- stochastic conception/live-birth draw from a named deterministic RNG stream;
- parent IDs on successful birth.

M2 should initially avoid:

- universal marriage rules;
- monogamy/polygyny assumptions;
- culturally specific mate-selection institutions;
- household = nuclear family assumptions;
- inferring social gender roles from reproductive sex;
- fertility effects from food/health before M3 explicitly models those links.

Mate/parent selection in the first implementation may be deliberately simple and local, but the simplification must be documented and replaceable.

## 7. Household/co-residence stance

A `Household` in v0.1 is an **engineering/social co-residence and resource-sharing container**, not an anthropological claim that prehistoric people universally lived in Western-style nuclear households.

Parentage and household membership must therefore be independent relationships:

- a biological parent may reside in another household;
- children may change household without changing parentage;
- household dissolution/formation can later be modelled without rewriting genealogy;
- marriage should not be required to represent reproduction.

Until stronger cross-cultural kinship modules exist, initialization should create simple co-resident units while explicitly marking the formation rule as synthetic.

## 8. Initialization problem

A random uniform age distribution is scientifically poor because it creates an artificial demographic transient.

M2 should support at least two initialization modes:

1. **Synthetic test population** — deterministic distributions designed for unit/benchmark tests.
2. **Schedule-consistent population** — age structure sampled from a configured stable/quasi-stable demographic schedule or from a documented empirical age distribution.

The run manifest must record which initialization method and schedule version were used.

## 9. Validation targets for M2

Before coupling demography to resources, M2 should demonstrate:

### Verification

- exact population accounting: `initial + births - deaths = living`;
- no birth to dead/nonexistent parents;
- no impossible self-parent or duplicate-parent references;
- ages advance monotonically;
- death occurs once;
- parent IDs remain stable after death;
- same build + config + seed reproduces the same trajectory;
- named demographic random streams do not perturb world generation.

### Behavioural validation/sanity

For empirical or evidence-informed presets, report rather than hide:

- age-specific mortality;
- survivorship curve;
- life expectancy at birth;
- survival to ages 5, 15 and 45;
- age distribution;
- age-specific fertility;
- mean age at childbearing;
- interbirth interval distribution;
- completed fertility distribution;
- annual growth rate;
- dependency ratios.

A preset should fail validation if it cannot reproduce the empirical quantities it claims to represent within declared tolerances/uncertainty.

## 10. Sensitivity requirements

At minimum, later M7 ensembles should vary:

- infant/child mortality level;
- young-adult mortality level;
- senescent mortality slope/late-age schedule;
- age at reproductive onset;
- fertility schedule magnitude;
- postpartum spacing;
- sex ratio at birth;
- initialization age structure.

Results that disappear under small plausible changes must be reported as parameter-sensitive rather than robust findings.

## 11. Evidence categories used by AnthroSim

Every demographic parameter/preset should carry one of these provenance statuses:

- **EMPIRICAL_DIRECT** — value/schedule taken directly from a cited dataset or published table;
- **EMPIRICAL_DERIVED** — calculated from cited empirical data with documented transformation;
- **EVIDENCE_INFORMED** — chosen within a cited comparative range but not directly observed as that exact value;
- **SYNTHETIC_VALIDATION** — deliberately artificial value used for software/model tests;
- **UNRESOLVED** — required by the model but not yet defensibly parameterized.

A parameter should never silently move from `SYNTHETIC_VALIDATION` to an empirical scientific claim.

## 12. Known limitations of this v0.1 research baseline

- extant foragers have experienced contact, disease, displacement, state pressure, trade and ecological changes;
- published samples are geographically and culturally non-representative of all hunter-gatherers;
- small population sizes create sampling uncertainty;
- age estimation and retrospective reproductive histories may contain error;
- fertility, mortality and migration are not independent;
- deep-prehistoric demographic regimes may include combinations absent from ethnographic datasets;
- archaeology/palaeodemography constrain different quantities and contain their own preservation/estimation biases;
- M2 alone lacks resource limitation, disease, environmental mortality shocks and migration feedback, so long-run population trajectories are not yet expected to be realistic.

## 13. Sources to retain in the provenance set

1. Sear R, Lawson DW, Kaplan H. 2020. *Reconstructing prehistoric demography: What role for extant hunter-gatherers?* DOI 10.1002/evan.21869. https://pubmed.ncbi.nlm.nih.gov/33103830/
2. Gurven M, Kaplan H. 2007. *Longevity Among Hunter-Gatherers: A Cross-Cultural Examination.* DOI 10.1111/j.1728-4457.2007.00171.x. https://doi.org/10.1111/j.1728-4457.2007.00171.x
3. Blurton Jones NG et al. 1992. *Demography of the Hadza, an increasing and high density population of Savanna foragers.* DOI 10.1002/ajpa.1330890204. https://pubmed.ncbi.nlm.nih.gov/1443092/
4. Konner M, Worthman C. 1980. *Nursing frequency, gonadal function, and birth spacing among !Kung hunter-gatherers.* DOI 10.1126/science.7352291. https://pubmed.ncbi.nlm.nih.gov/7352291/
5. Colleran H et al. 2016. *Reproductive trade-offs in extant hunter-gatherers suggest adaptive mechanism for the Neolithic expansion.* DOI 10.1073/pnas.1524031113. https://pmc.ncbi.nlm.nih.gov/articles/PMC4855554/
6. Ramirez Rozzi FV et al. 2018. *Reproduction in the Baka pygmies and drop in their fertility with the arrival of alcohol.* Comparative reproductive table and primary references. https://pmc.ncbi.nlm.nih.gov/articles/PMC6142234/
7. *Periodic catastrophes over human evolutionary history are necessary to explain the forager population paradox.* 2019. Comparative vital rates and long-run growth problem. https://pmc.ncbi.nlm.nih.gov/articles/PMC6600907/
8. *Sex ratio is remarkably constant.* PMID 19159875. https://pubmed.ncbi.nlm.nih.gov/19159875/

## 14. Decision for M2 implementation

Proceed with a **schedule-driven demographic engine** and a clearly named `synthetic_validation_v1` preset. Do not encode a generic `hunter_gatherer` preset.

The first M2 implementation may be scientifically modest, but its architecture must allow later empirical schedules to be substituted without changing the demographic engine itself. This keeps the distinction between **software mechanism** and **scientific parameterization** explicit from the beginning.
