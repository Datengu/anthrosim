# v0.1 migration-model provenance and evidence boundary

**Status:** M4 synthetic validation baseline, executable timing refined by the [M3/M4 response-time contract v1](m3-response-time-contract-v1.md)  
**Scientific status:** unvalidated  
**Runtime empirical dataset:** none

This document records the assumptions and interpretation boundary of the first executable AnthroSim migration model. M4 exists to establish bounded, causal and inspectable movement rather than to encode an ethnographic or prehistoric mobility schedule.

The executable preset is named `synthetic_validation_v1` and carries provenance `synthetic_validation`.

## What M4 is intended to verify

M4 should make it possible to ask engineering/model questions such as:

- can deteriorating local conditions create relocation pressure without a scripted destination?
- can households compare only a bounded local neighbourhood rather than globally optimizing the world?
- can the selected destination be explained factor-by-factor after the run?
- can stochastic choice remain exactly replayable from configuration and seed?
- can co-resident household members relocate together without an allocation-heavy object graph?
- can travel impose an explicit cost rather than make movement free?
- can migration metrics expose distance, direction, origin/destination conditions and spatial fragmentation?
- can permanent-migration opportunity frequency be specified independently of M3 resource-settlement resolution?

Passing those checks does not establish that the current thresholds, decision frequency or utility weights describe any real hunter-gatherer population.

## Decision schedule

M4 permanent migration has its own explicit fixed decision clock, `migration.decisionPeriodsPerYear`. The synthetic validation default is four opportunities per 365-day model year. This clock is intentionally independent of M3 `resources.periodsPerYear`: increasing resource-settlement resolution must not silently create more permanent-migration opportunities.

The authoritative scheduler merges the M3 and M4 fixed clocks. On a day where both are due, elapsed M3 resource settlement/condition/scarcity processing occurs first, then due M9 temporary-mobility processing, then M4. On an M4-only decision day, M4 can therefore observe the current resource/condition state without requiring a new M3 settlement on that same day. The annual M2 demographic transition follows the completed subannual schedule at the year boundary. Only surviving households eligible under the current temporary-presence rules evaluate permanent movement.

M4's resource-support cue allocates annual food need over the M4 decision interval itself, using the same cumulative elapsed-day annual-quantity rule used by M3. It does not assume that a decision boundary is also an M3 resource boundary. The runtime checks that the declared M4 decision index, configured decision frequency and actual decision day agree; inconsistency fails closed.

All households at one migration boundary make decisions against the same pre-move snapshot. Planned household relocations are then applied simultaneously in a single packed population pass. This avoids a household-ID ordering artefact in which an early mover changes the state observed by later decision makers.

Movement currently completes at that decision boundary rather than creating a persistent en-route state. The completed move applies a condition cost proportional to distance. This is an explicit computational approximation, not a claim that real movement is instantaneous.

The decision clock itself remains a scientific assumption. Separating it from M3 removes an accidental numerical coupling; it does not establish that four permanent-relocation opportunities per year are empirically correct.

## Bounded local knowledge

A household can inspect destination cells within a Manhattan-distance radius around its current cell. The default radius is three cells. The origin is not considered a move candidate; remaining at the origin is represented by an explicit **stay utility** against which candidate improvement is compared.

For an interior cell the number of candidates is bounded by:

`2r(r + 1)`

so the default radius `r = 3` exposes at most 24 destination cells regardless of total world size. Edge/corner households see fewer candidates.

There is no lookup of the globally best cell, no historical migration target, no hard-coded route and no knowledge of cells outside the configured local radius.

The current radius is a synthetic information horizon. It is not an empirical estimate of forager geographic knowledge.

## Migration pressure

A household's relocation pressure is computed from two explicit deficits:

1. mean living-member condition below a configured condition threshold;
2. local resource support below a configured resource threshold.

The deficits are added and bounded to 0..1000 permille. Under the default synthetic preset, a healthy and locally well-supported household has no migration pressure, while deteriorating condition and/or resource support increases pressure.

This is a trigger mechanism, not a calibrated psychological, social or ethnographic model of migration motivation.

## Residence utility and relocation action costs

M4 separates two different scientific quantities when comparing staying with moving.

### Residence-state terms

The following describe the state of residing at a cell and therefore apply both to the current residence and to candidate residences:

| Factor | Current interpretation | Evidence status |
|---|---|---|
| Resource score | Dynamic M3 food stock relative to the M4 decision interval's allocated annual demand after adding the moving household where applicable | Synthetic validation proxy |
| Water/security score | Weighted water accessibility plus inverse environmental stress | Synthetic validation proxy |
| Kin score | Presence of a reciprocal cross-household living parent-child tie to a household persistently resident at the evaluated cell | Minimal first-degree genealogical proxy |

### Relocation-only action costs

The following describe the act/uncertainty/risk of relocating and therefore apply **only to candidate moves**:

| Factor | Current interpretation | Evidence status |
|---|---|---|
| Travel penalty | Manhattan distance plus the **destination** cell's terrain movement-cost excess | Synthetic validation proxy |
| Uncertainty penalty | Deterministic stochastic candidate penalty from a named migration RNG stream | Synthetic validation proxy |
| Relocation-risk penalty | Base relocation penalty plus distance-dependent relocation penalty | Synthetic validation proxy |

The stay action has exactly zero travel penalty, zero candidate uncertainty and zero relocation-risk penalty. In particular:

- the origin's terrain movement cost cannot lower the utility of remaining in place;
- the base relocation-risk penalty is not charged to staying and therefore cannot cancel out of the move-versus-stay comparison;
- increasing relocation risk can only make relocation less attractive under otherwise equal residence terms;
- destination movement cost represents the current simplified cost of moving to/through that destination, not a cost of occupying rough terrain while stationary.

Positive residence factors are combined with configurable integer weights. Candidate travel, uncertainty and relocation-risk factors subtract from relocation utility. A destination must exceed the explicit stay utility by a configured minimum improvement before it can be selected.

The exact functional form and all default weights are placeholders. They must not be interpreted as measured human preferences.

## Kin scope

M4 uses only genealogical information already present in the model. Under the v13 contract, each living parent-child relation that crosses household boundaries creates a **reciprocal** spatial tie: the child's household receives the parent household's persistent residence as a kin anchor, and the parent's household receives the child household's persistent residence. Female- and male-parent links use the same rule. A parent-child relation inside one household creates no spatial anchor because M4 moves that household as a unit and the relationship therefore does not distinguish staying from relocating.

The score is deliberately a binary per-cell presence proxy: a residence receives `250` kin-score permille when at least one reciprocal cross-household living parent-child tie points to that cell and `0` otherwise. Multiple first-degree relatives at the same cell do not stack. All unique resulting cells are retained; there is no fixed first-N cap, so packed person-record/birth order cannot decide which kin locations exist in M4 utility.

This reciprocal rule is important because newborns normally join the female parent's household. The earlier parent-only external rule therefore made an apparently neutral kin term behave predominantly like attraction toward external male-parent/father locations. v13 represents the cross-household genealogical edge in both directions instead of turning maternal household inheritance into a one-way social preference. It also avoids treating a co-resident parent's old cell as a reason to stay when that parent would move with the household. Reproductive sex retains only its limited M2 biological meaning and is **not** a model of social gender, patrilocality or descent.

This is deliberately narrow. It is **not** a model of siblings, clans, descent groups, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial communities or culturally defined kin obligations. Those would require additional social state and evidence. The normative symmetry and ordering contract is [`m4-kin-proxy-v1.md`](m4-kin-proxy-v1.md).

## Deterministic stochastic choice

Candidates that strictly clear the minimum utility improvement receive a stochastic weight equal to their positive integer utility improvement above that required threshold. There is no `+1` pseudocount: improvements `[1, 2]` produce weights `[1, 2]`, `[1, 10]` produces `[1, 10]`, equal improvements receive equal weights, and multiplying every eligible improvement by one common positive factor preserves the relative selection probabilities. One destination is then drawn from those weights using the named `migration/choice` random stream. Candidate uncertainty uses the independent `migration/uncertainty` stream.

This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable. `MigrationDecisionTrace` preserves the stable candidate-order table of every eligible candidate's cell, utility and exact weight, together with the selected weight, total move weight and choice draw. The exact probability assigned to every eligible alternative is therefore reconstructible as `candidateWeight / totalMoveWeight` for every retained trace.

## Household-coordinated movement

The household is the M4 movement unit. When a household relocates, all currently living members move to the selected cell together and the household's current location is updated. Persistent records of people who died before the move retain their location at death; they are not retroactively relocated with living household members.

This preserves a useful distinction between current co-residence and persistent individual history. It does not imply that real households are universally the correct migration decision unit.

## Travel cost and relocation risk

A completed move requests a nominal per-person condition decrement according to distance and the configured per-cell travel-condition cost. Each mover's actual decrement is bounded by their available condition, so the realized loss can be smaller when condition saturates at zero. Authoritative migration events and retained traces therefore preserve separate nominal-per-person and realized-per-move quantities: event JSON follows the established snake_case event-field convention (`nominal_travel_condition_cost_per_person`, `realized_travel_condition_loss_total`), while retained trace JSON uses camelCase (`nominalTravelConditionCostPerPerson`, `realizedTravelConditionLossTotal`). The manifest reports the all-move realized `travelConditionCostTotal`.

Relocation risk is currently a **candidate-action decision penalty** rather than an additional movement-injury or movement-mortality draw. The base term is paid once for taking the relocation action; the per-cell term increases that anticipated penalty with distance. Neither applies to staying.

Thus M4 represents the cost/risk of moving in two different simplified ways:

- anticipated relocation risk and candidate travel cost reduce relocation utility;
- completed travel produces condition loss.

Neither term is calibrated to measured energetic expenditure, injury, mortality or journey duration.

## Decision traces

To keep normal manifests bounded, M4 records only the first configured number of completed moves (default 256) as detailed decision traces. Each trace includes:

- decision/completion day;
- household and number of living movers;
- origin and destination;
- Manhattan distance;
- relocation pressure;
- number of locally visible candidates;
- factor-by-factor **stay** utility for the origin (with relocation-only costs explicitly zero);
- factor-by-factor selected-destination relocation utility;
- best visible candidate and its utility;
- every eligible candidate's cell, utility and exact stochastic-choice weight, plus the selected weight, total move weight and choice draw;
- nominal travel-condition cost per person;
- exact realized bounded travel-condition loss for that completed household move.

Aggregate migration totals continue to include all moves even after the detailed-trace cap is reached.

A trace explains the implemented model decision. It should not be mistaken for a reconstruction of a real person's motives.

## Aggregate migration metrics

The M4 summary reports decision boundaries, households evaluated, households under pressure, completed household moves, people moved, total Manhattan distance, north/east/south/west step totals, realized travel condition loss, mean origin/destination resource and water/security scores, the migration-attributable change in occupied-cell count, bounded decision traces and a deterministic digest.

The occupied-cell delta is a spatial fragmentation/concentration indicator attributable to the simultaneous move boundary. It is not a settlement typology.

## Current default synthetic parameters

`synthetic_validation_v1` currently uses:

| Parameter | Default |
|---|---:|
| Migration enabled | true |
| Decision periods per year | 4 |
| Candidate radius | 3 cells |
| Condition pressure threshold | 900 permille |
| Resource pressure threshold | 850 permille |
| Minimum utility improvement | 150 utility units |
| Resource weight | 5 |
| Water/security weight | 2 |
| Kin weight | 1 |
| Travel-cost weight | 2 |
| Maximum uncertainty penalty | 100 permille |
| Base relocation-risk penalty | 50 permille |
| Relocation-risk penalty per cell | 25 permille |
| Travel condition cost per cell | 10 permille |
| Detailed move traces retained | 256 |

These values were chosen to exercise the mechanism across local surplus/scarcity conditions and are **not empirical estimates**.

## Verification claims M4 may make

Once the milestone acceptance tests and CI pass, it is legitimate to say that the implementation verifies properties such as:

- destination discovery is spatially bounded and independent of total-world search;
- no historical destination or migration route is encoded;
- each recorded move can be decomposed into explicit implemented residence and relocation-action utility factors;
- the stay comparator has zero travel, uncertainty and relocation-risk costs;
- increasing base relocation risk can reduce move eligibility rather than cancelling against the stay action;
- changing only the origin movement cost cannot penalize a zero-distance stay action through a travel term;
- increasing only candidate travel/terrain cost cannot make that candidate more attractive;
- identical configuration/seed yields identical migration decisions and traces;
- worsened local resource/condition state directionally increases relocation pressure under otherwise equal inputs;
- changing only M3 resource-period count does not multiply the configured M4 decision-opportunity count;
- cross-household living parent-child ties are reciprocal and independent of female/male parent role, while same-household ties add no spatial preference;
- permuting otherwise-equivalent person/birth record order cannot remove or substitute represented kin-location anchors;
- changing `migration.decisionPeriodsPerYear` changes M4 opportunity frequency independently of M3 settlement resolution;
- living household members relocate together and packed population/occupancy invariants continue to reconcile;
- selected moves impose explicit condition costs;
- migration remains benchmarkable at the v0.1 population target.

It is **not** legitimate to infer that the current move frequency, distances, directions, utility trade-offs or survival effects reproduce a specific prehistoric population.

## Evidence required before an empirical migration preset

A research-capable migration configuration should be tied to a bounded research question and may require, depending on context:

1. archaeological evidence on mobility scale, settlement duration and relocation frequency;
2. ethnographic mobility evidence used with explicit discussion of analogy limits;
3. palaeoenvironmental/resource uncertainty at matching spatial and temporal scales;
4. travel energetic and terrain-cost models with real units where appropriate;
5. water availability and environmental-risk evidence rather than a generic security proxy;
6. social/kinship structure appropriate to the population under study;
7. information horizons, landscape familiarity and route-memory assumptions;
8. relocation risk and journey-duration evidence;
9. calibration targets separated from validation targets;
10. sensitivity and uncertainty analysis demonstrating whether conclusions survive plausible alternative mobility assumptions.

## M4 interpretation rule

Results produced by `synthetic_validation_v1` should be described conditionally:

> Under the stated synthetic mobility assumptions, worsening local conditions increased relocation pressure and produced movement pattern Y within the model.

They should not be promoted to claims such as:

> Hunter-gatherers moved X cells when resources fell below Y, or this route reconstructs a prehistoric migration.

That boundary remains mandatory until a configuration has appropriate empirical provenance, calibration, validation and uncertainty analysis for the specific question being asked.
