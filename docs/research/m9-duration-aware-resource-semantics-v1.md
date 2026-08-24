# M9.5 duration-aware resource semantics v1

**Milestone:** M9.5  
**Status:** implemented semantics  
**Scientific status:** synthetic/null-model accounting semantics; unvalidated for any real population  
**Parent contract:** `docs/research/temporary-mobility-v1.md`

## Purpose

M9.5 removes a known semantic error at the boundary between M3 resources and M9 temporary mobility. A household's ordinary resource-period need must not be charged wholly to whichever place happens to be relevant at one resource boundary. Short visits must affect destination demand in proportion to their duration, including visits that begin and end entirely between resource boundaries.

This document freezes the deterministic accounting details required by the M9.0 contract. It does not introduce storage, foraging, pack animals, diet, household bargaining, calibrated energetics or any archaeological interpretation.

## Presence-duration ledger

When temporary mobility is enabled, AnthroSim maintains one compact authoritative duration ledger per household for the current resource period. The event-driven scheduler accrues elapsed integer days before applying a temporary-mobility transition.

The ledger distinguishes:

- days at residence;
- outbound-transit days;
- visiting days;
- return-transit days;
- the visitor destination cell when visiting days are non-zero.

Intervals use the same half-open convention as M9 journey timing. A transition on day `d` changes the state used for the interval beginning at `d`; elapsed time before that transition is attributed to the previous state. No per-person daily loop is introduced.

At every resource boundary, each household's four duration counts must sum exactly to the elapsed resource-period length. The ledger is consumed and reset only after that period has been presented to M3 resource accounting.

## Home-provisioning proxy

M9 v1 attributes resource demand by physical-presence duration with one explicit exception inherited from the M9.0 contract:

- `at_residence` days -> residence cell;
- `visiting` days -> visitor destination cell;
- outbound and return transit days -> residence cell.

The two transit categories therefore form part of a **home-provisioning** duration bucket. This means travel provisions are treated as originating from home. It is a model assumption and future sensitivity dimension, not a claim about prehistoric provisioning behaviour.

## Exact need partition

M9.5 does not change the existing M3 calculation of ordinary household period need. Living-member count and per-person period need are calculated exactly as before.

For a household with total period need `N`, period duration `D`, home-provisioning days `H` and visiting days `V`, where `H + V = D`, provisional attributed needs are:

```text
home_floor  = floor(N * H / D)
visit_floor = floor(N * V / D)
```

Any remaining unit caused by integer division is assigned by descending fractional remainder. An exact tie is resolved to the home-provisioning claim first. This stable rule conserves `N` exactly without stochastic rounding.

Claims with zero attributed need are omitted. A period with no visiting days therefore produces exactly one residence claim, matching the legacy M3 shape.

## Cell competition and household satisfaction

After duration-weighted claims are produced, M3 retains its existing cell-stock competition semantics:

1. claims at a cell are summed to cell demand;
2. available stock defines the cell target harvest;
3. target harvest is allocated proportionally across claims;
4. bounded integer allocation remainder is resolved in stable claim order: household ID, then home claim before visitor claim;
5. claim allocations are summed back to one household harvest total.

A household's overall resource-satisfaction fraction and condition update use its **total household harvest divided by its unchanged total household need**. A household drawing from two cells therefore receives one reconciled condition consequence; demand is neither created nor destroyed by travel.

`household_periods_with_unmet_need` remains a household-period count, not a claim count.

## Resource-boundary ordering

The M9.0 ordering remains authoritative:

1. accrue presence duration up to the resource boundary, excluding transitions at that boundary;
2. settle resource demand for the elapsed period;
3. reset the duration ledger at the boundary;
4. complete/start temporary transitions due on that day;
5. evaluate M4 migration for eligible households;
6. run annual M2 demography when applicable.

This prevents a same-day departure from changing the preceding resource period and prevents a short visit crossing a boundary from being treated as a whole-period destination stay.

## Disabled-mode compatibility

When temporary mobility is not configured, no M9 resource ledger is active and M3 follows the legacy single-residence-claim path. M9.5 must not introduce a second rounding path for disabled M9 runs.

## Determinism, checkpoint and provenance

The duration ledger is authoritative model state while M9 is enabled. It is serialized and included in deterministic state identity. At supported annual checkpoint boundaries the immediately preceding resource period has already been settled, so the ledger is expected to be reset at that boundary; resumed execution must nevertheless validate and preserve the serialized ledger exactly.

M9.5 changes authoritative resource attribution. The implementation therefore advances `MODEL_SEMANTICS_ID` to `anthrosim-model-semantics-v5` and the checkpoint schema to v9 while leaving the package version unchanged during M9 development.

## Acceptance

Implemented tests cover:

- one-day and five-day visits;
- visits entirely between resource boundaries;
- visits spanning a resource boundary;
- outbound and return transit attributed to home provisioning;
- exact household-demand conservation under integer rounding;
- deterministic remainder ties;
- a household drawing supply from both residence and visitor cells;
- condition/scarcity consequences based on the reconciled household supply fraction;
- same-day resource/temporary-transition ordering through the scheduler boundary;
- deterministic replay and checkpoint/resume through the existing M9 integration coverage;
- disabled-M9 compatibility with the legacy M3 path through the preserved single-claim execution path and regression suite.

The public M9.5 acceptance test additionally verifies that an otherwise identical one-day visit removes exactly one unit of destination stock and a five-day visit removes exactly five units under a one-unit-per-day synthetic fixture, demonstrating that visits wholly between resource boundaries do not disappear from M3 accounting.

Passing these tests validates the accounting capability, not the empirical correctness of the home-provisioning assumption.
