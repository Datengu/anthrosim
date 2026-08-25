# AnthroSim ODD+D human decision-making supplement

**Protocol:** ODD+D (Müller et al. 2013)  
**AnthroSim baseline:** v0.3.0 / completed M9  
**Status:** formal living supplement to [`odd.md`](odd.md)  
**Scientific status:** current human-decision mechanisms are synthetic / unvalidated

ODD+D extends ODD so assumptions about human decision-making are not hidden inside equations or generic terms such as “agent behaviour”. AnthroSim uses this document to state what its people/households are actually assumed to know, choose, optimize, learn and socially respond to — and, equally importantly, what is **not** represented.

The current baseline contains only a narrow explicit decision model in M4 permanent migration. M9 temporary mobility is a generic configurable journey/aggregation mechanism, not yet an empirically validated cognitive or social theory of why people decide to gather. M2 fertility/parentage is a demographic mechanism and must not be interpreted as a model of conscious reproductive choice.

---

## I. Overview

### I.i Purpose

The decision-model purpose is to provide transparent null/baseline mechanisms that can be replaced, compared or falsified without embedding a desired historical outcome.

Current decision-related questions are methodological:

- Can a household under declared resource/condition pressure make a bounded local permanent relocation without global optimization or scripted destinations?
- Can temporary presence at a focal region be represented separately from persistent residence, while leaving the motive for participation explicitly open unless a study-specific hypothesis supplies one?

The model is **not** intended, in its present form, to reconstruct Iron Age cognition, household politics, social norms, ritual motivation, strategic planning, territorial institutions or culturally specific movement rules.

### I.ii Entities, state variables, scales and exogenous factors

Decision-relevant entities/state include:

- living people and their household/genealogical state;
- household persistent residence and mean member condition;
- local cell resource support and model-facing environmental fields;
- bounded M4 candidate cells;
- focal-region identity and temporary-journey configuration for M9;
- stochastic uncertainty/choice streams;
- externally supplied spatial transformations and experiment configuration.

Important exogenous/model-fixed factors include utility weights, pressure thresholds, candidate radius, minimum utility improvement, uncertainty/risk parameters, travel-cost semantics and M9 participation/timing configuration. These are not learned by agents in the current baseline.

### I.iii Process overview and scheduling

At eligible M4 boundaries, households are first tested for relocation pressure. Pressured households compare the utility of staying with bounded local alternatives using a shared pre-move snapshot. Alternatives that exceed the configured minimum improvement become eligible; one is selected through weighted deterministic stochastic choice. Selected moves are then applied simultaneously.

M9 journey starts/transitions occur according to the declared temporary-mobility configuration and travel semantics. M9 does not currently simulate a cognitive deliberation in which a household weighs social motives and decides whether to attend.

Scheduling details are part of the model and are specified in [`odd.md`](odd.md), [`../scientific-model.md`](../scientific-model.md) and the M9 research contracts.

---

## II. Design concepts

### II.i Theoretical and empirical background

#### General model basis

AnthroSim currently uses **synthetic bounded-decision/null-model rules**, not a single established anthropological theory of decision-making.

M4 is closest to a bounded-rationality heuristic: agents have limited spatial information, compare a small set of alternatives, require improvement over the status quo, and make stochastic weighted choices among acceptable alternatives. This resemblance is descriptive; the current weights/thresholds are not claimed to be empirically fitted measures of bounded rationality.

The model intentionally separates:

- **mechanism structure** — e.g. local information, pressure-triggered comparison, multiple acceptable destinations;
- **parameterisation** — currently synthetic unless a study explicitly evidence-grounds it;
- **historical interpretation** — always downstream and question-specific.

#### Behavioural-theory status

| Mechanism | Current theoretical/empirical status |
|---|---|
| M4 relocation pressure | Synthetic heuristic based on condition/resource deficits. |
| M4 candidate utility | Synthetic multi-factor utility proxy; not validated psychology. |
| M4 local information horizon | Explicit bounded-information assumption; current radius synthetic unless evidence-grounded. |
| M4 kin contribution | Narrow genealogical proxy; not a theory of kinship, alliance or social obligation. |
| M4 uncertainty | Stochastic proxy; not calibrated perception or risk cognition. |
| M9 temporary participation | Generic configured mechanism; motive/decision theory intentionally absent. |
| M2 reproduction/parent selection | Demographic mechanism; not conscious choice or marriage model. |

Future decision submodels must state whether they derive from behavioural theory, ethnographic analogy, empirical statistical relationship, mechanistic assumption, heuristic/null rule, or a declared combination.

### II.ii Individual/household decision-making

#### Subjects and objects of decisions

The current explicit decision subject is the **household** in M4. The object is whether to remain at the current persistent residence or relocate permanently to one acceptable candidate cell.

Individual household members do not vote, bargain or act independently in the baseline M4 decision.

M9 temporary travel is also applied at household level, but its current start/participation logic should be understood as a configured mechanism rather than a validated model of household deliberation.

#### Levels of decision-making

Only a local household-level decision is represented. There is no village, lineage, polity, council, leader or inter-household collective decision layer.

#### Objectives / rationality

M4 households do not maximize a globally known landscape. They compare staying against bounded candidates using a synthetic utility function combining:

- resource support;
- water/security proxy;
- narrow kin/parent-location proxy;
- travel/terrain cost;
- uncertainty penalty;
- relocation-risk penalty.

A destination must exceed the status quo by a configured minimum improvement. Multiple qualifying alternatives may compete stochastically with weight proportional to the declared improvement semantics.

This is an **instrumental model rule**, not evidence that historical people consciously optimized a scalar utility function.

#### Decision rules and adaptation

Positive relocation pressure activates the M4 decision opportunity. The decision rule itself is fixed during a run. Households adapt by changing residence, but do not adapt the decision algorithm, weights, thresholds or information strategy.

#### Social norms and cultural values

No explicit social norms, cultural values, taboos, status systems, territorial rights, ritual obligations, inheritance rules, prestige competition or political authority affect M4/M9 decisions in the baseline.

The absence of these processes defines the model's current domain of applicability. It must not be reinterpreted as a claim that such processes were historically absent.

#### Spatial aspects

M4 information and alternatives are spatially bounded by the candidate radius and world boundary. Travel/terrain cost contributes to candidate evaluation. M9 uses declared focal-region/travel semantics.

For evidence-grounded studies, the translation from cell units to physical distance and the influence of raster resolution/extent are part of model evaluation, not merely GIS preprocessing.

#### Temporal aspects

M4 opportunities occur at eligible resource boundaries rather than continuously. M9 follows configured journey timing. The frequency of decision opportunities can alter system behaviour and therefore requires temporal-resolution/scheduling sensitivity for relevant claims.

#### Uncertainty

M4 includes an explicit stochastic uncertainty penalty and stochastic destination choice. This is not an empirical cognitive error model. Other unknowns are represented through parameter uncertainty/sensitivity at the experiment level rather than being assumed to exist inside an agent's mind.

### II.iii Learning

No persistent individual or collective learning is implemented.

Agents do not currently:

- remember past candidate quality;
- learn routes or travel times;
- update beliefs from outcomes;
- imitate successful households;
- transmit cultural knowledge;
- adapt utility weights or thresholds;
- develop habitual/seasonal movement traditions.

If a research result depends on repeated experience or cultural transmission, the current model is incomplete for that claim unless sensitivity/alternative-model work shows learning is unnecessary to the inference.

### II.iv Individual sensing

M4 sensing is represented by access to declared candidate-state proxies inside the bounded local radius. The household does not observe the entire world.

The baseline does not distinguish different members' perceptions or noisy measurement of individual environmental cues beyond the explicit uncertainty mechanism. It does not model scouts, oral reports, maps, rumours or hidden information.

### II.v Individual prediction

Households do not make explicit forecasts of future population, weather, resource regeneration, crowding or other households' choices.

M4 uses current/proxy candidate state. Simultaneous movers do not anticipate one another's same-boundary arrival. This is a consequential null assumption and should be sensitivity-tested where crowding/coordination matters.

### II.vi Interaction

Decision-relevant interaction occurs indirectly through:

- shared household condition/resources;
- same-cell competition for resources;
- local crowding consequences;
- direct-parent location used by the narrow kin proxy;
- simultaneous relocation outcomes that affect later periods.

There is no negotiation, signalling, exchange, coalition formation, imitation, conflict or strategic game among households.

### II.vii Collectives

Households are the only decision-bearing collective in the baseline. Focal regions are spatial targets, not social institutions.

No higher-order decision collective exists. Any future lineage, settlement council, polity, ritual group, market or military organization must be introduced as an explicit new entity/process rather than inferred from household aggregation.

### II.viii Heterogeneity

Households can differ because their members, condition, residence, genealogy and local environment differ. The baseline does not assign stable personality, preference type, risk attitude, wealth class, social status or cultural identity distributions.

Decision heterogeneity therefore emerges from state/context and stochastic choice rather than from empirically parameterized behavioural types.

### II.ix Stochasticity

M4 uses named deterministic random streams for uncertainty and weighted destination choice. Stochasticity provides reproducible variation, not epistemic justification.

If stochastic parameters are later interpreted empirically, their distribution and evidence basis must be documented separately from the mere fact that the RNG is deterministic/reproducible.

### II.x Observation (including emergence)

Decision traces, events and downstream observability can be inspected to determine which alternatives were considered/selected and what spatial outcomes emerged.

Interpretive labels are not automatic. A cluster of households, repeated visits or a focal-region peak does not by itself establish a social institution, ritual motive or political centre.

---

## III. Details

### III.i Implementation details

The detailed scientific semantics are in [`../scientific-model.md`](../scientific-model.md). Principal decision-related implementation lives in the M4/M9 core modules, including:

- `crates/anthrosim-core/src/migration.rs`
- `crates/anthrosim-core/src/temporary_mobility.rs`
- `crates/anthrosim-core/src/temporary_travel.rs`
- related configuration/spatial modules.

Implementation names should remain close to scientific terms so reviewers can trace ODD+D concepts into code.

### III.ii Initialization

Decision parameters are fixed by the versioned experiment configuration unless an explicit future adaptive mechanism says otherwise. Founder/residence/environment initialization can affect subsequent decision opportunities and must be included in initialization-transient analysis where relevant.

### III.iii Input data

Evidence-grounded decision parameters must use the same evidence/provenance system as other scientific parameters. Ethnographic analogy must state applicability limits rather than being silently generalized to prehistory.

Evidence used to construct or calibrate a decision rule must be distinguished from evidence reserved for independent corroboration; see [`trace.md`](trace.md).

### III.iv Submodels

The current decision-related submodels are:

1. M4 relocation-pressure calculation;
2. M4 candidate enumeration/information horizon;
3. M4 candidate utility and stay comparison;
4. M4 stochastic destination choice;
5. M4 simultaneous move application/travel condition effect;
6. M9 configured participation/start semantics;
7. M9 travel/duration/temporary-presence lifecycle.

Each submodel's equations and exact semantics remain normative in the detailed scientific/model-specific documents rather than being duplicated here.

---

## ODD+D decision audit checklist

Any new human-behaviour mechanism must answer all applicable questions below before being called research-ready:

- Who is the decision subject and what is being decided?
- At what social level is the decision made?
- What alternatives exist, and which alternatives are perceived?
- What information can the decision-maker sense, and with what uncertainty?
- What objective, heuristic or theory produces the decision?
- Is the rule empirically/theoretically based, evidence-informed, synthetic or unresolved?
- What social norms/cultural values are represented or deliberately omitted?
- What spatial and temporal limits constrain the decision?
- Does the agent predict future states?
- Can the agent learn or adapt the decision rule?
- How do agents/households interact strategically or indirectly?
- What heterogeneity exists among decision-makers?
- What stochasticity exists and what does it mean scientifically?
- Which observed patterns should this decision mechanism reproduce?
- Which competing decision formulation has been considered?
- Which findings are sensitive to the decision model?

A missing process may be a valid null assumption. It must be declared, and the affected interpretation must remain bounded accordingly.

## Reference

Müller, B. et al. (2013). *Describing human decisions in agent-based models – ODD + D, an extension of the ODD protocol.* Environmental Modelling & Software 48:37–48. DOI: `10.1016/j.envsoft.2013.06.003`.
