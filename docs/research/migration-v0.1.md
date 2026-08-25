# v0.1 migration-model provenance and evidence boundary

**Status:** M4 synthetic validation baseline  
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

Passing those checks does not establish that the current thresholds or utility weights describe any real hunter-gatherer population.

## Decision schedule

Migration is evaluated after each M3 resource period, after resource regeneration, household acquisition/sharing, condition change and scarcity mortality. Only surviving households evaluate movement. The annual M2 demographic boundary follows the resource/migration periods.

All households at one migration boundary make decisions against the same pre-move snapshot. Planned household relocations are then applied simultaneously in a single packed population pass. This avoids a household-ID ordering artefact in which an early mover changes the state observed by later decision makers.

Movement currently completes at that decision boundary rather than creating a persistent en-route state. The completed move applies a condition cost proportional to distance. This is an explicit computational approximation, not a claim that real movement is instantaneous.

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
| Resource score | Dynamic M3 food stock relative to local period demand after adding the moving household where applicable | Synthetic validation proxy |
| Water/security score | Weighted water accessibility plus inverse environmental stress | Synthetic validation proxy |
| Kin score | Presence of a bounded set of known, living direct-parent locations outside the household | Minimal genealogical proxy |

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

M4 uses only genealogical information already present in the model. For a household, the first implementation records up to four unique cells containing living direct parents of living household members when those parents reside outside the household. A candidate receives a bounded kin-proximity contribution when it matches one of those cells.

This is deliberately narrow. It is **not** a model of clans, descent groups, bilateral kindreds, marriage alliances, friendship, ethnicity, territorial communities or culturally defined kin obligations. Those would require additional social state and evidence.

## Deterministic stochastic choice

Candidates that clear the minimum utility improvement receive a weight proportional to their utility improvement. One destination is then drawn from those weights using the named `migration/choice` random stream. Candidate uncertainty uses the independent `migration/uncertainty` stream.

This means movement is not deterministic optimization: a household may choose among several locally acceptable alternatives. It is nevertheless exactly replayable under the declared AnthroSim determinism boundary because the candidate order, integer utilities and RNG streams are stable.

## Household-coordinated movement

The household is the M4 movement unit. When a household relocates, all currently living members move to the selected cell together and the household's current location is updated. Persistent records of people who died before the move retain their location at death; they are not retroactively relocated with living household members.

This preserves a useful distinction between current co-residence and persistent individual history. It does not imply that real households are universally the correct migration decision unit.

## Travel cost and relocation risk

A completed move deducts condition from each living mover according to distance and the configured per-cell travel-condition cost, bounded by the condition scale. The manifest reports total realized travel-condition loss.

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
- selected and total stochastic-choice weights plus the choice draw;
- travel condition cost per person.

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
