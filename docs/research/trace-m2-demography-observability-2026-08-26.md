# TRACE change record — M2 demographic observability and acceptance closure

**Date:** 2026-08-26  
**Programme:** post-M9 scientific hardening / M2 repair completion  
**Entering model semantics:** `anthrosim-model-semantics-v7`  
**Scientific status:** implementation verification and model analysis; **not empirical validation**

## Purpose

This record documents the M2 observability/acceptance slice built after the demographic-time and founder-initialization repairs. It addresses the remaining need to explain *why* a realized birth did or did not occur without adding hidden calibration logic or hot-loop rejection events.

The normative analysis contract is [`m2-demography-observability-v1.md`](m2-demography-observability-v1.md). The underlying model remains defined by [`m2-demographic-time-contract-v1.md`](m2-demographic-time-contract-v1.md) and [`m2-founder-initialization-contract-v1.md`](m2-founder-initialization-contract-v1.md).

## 1. Problem formulation

The verification/analysis question is:

> Given an exact AnthroSim run, can a researcher separate mortality exposure, fertility age eligibility, executable birth spacing, local male availability, stochastic fertility outcome and operational truncation, while independently checking that the reconstructed M2 history agrees with authoritative state/events?

This is deliberately narrower than empirical demographic validation.

## 2. Model description

PR #236 does not change the v7 demographic transition equations or human decision rules. It adds a downstream, version-bound analysis surface.

The report reconstructs each annual M2 opportunity funnel from:

- day-zero Population;
- immutable experiment configuration;
- authoritative EventLog history;
- checkpoint/final Population state; and
- the independently seeded `demography/fertility` RNG stream.

It fails closed if the model-semantics identity does not match the analysis implementation or if replay cannot reconcile with final state.

Because this is observability rather than a causal-model change, `MODEL_SEMANTICS_ID` remains v7. ODD+D was reviewed and requires no semantic change in this slice. ODD/scientific documentation only needs to describe the new observation/verification pathway.

## 3. Data evaluation

No empirical demographic data are introduced, fitted or retuned.

The synthetic baseline continues to request `minimum_birth_spacing_days = 1278`, while the annual scheduler executes an effective lower bound of `1460` days. Both values are surfaced explicitly rather than allowing the evidence-oriented 3.5-year wording to masquerade as exact executable timing.

The revised demographic evidence baseline therefore distinguishes:

- comparative evidence ranges;
- raw/requested configuration values;
- executable annual-model meaning; and
- downstream empirical validation targets.

## 4. Conceptual model evaluation

The report exposes the structural filters that can suppress realized fertility:

1. M2 mortality;
2. female reproductive record/age-schedule eligibility;
3. requested-to-effective spacing;
4. pre-same-day-M4 local eligible-male availability;
5. stochastic fertility draw;
6. persistent-person-record ceiling.

This prevents a future calibration workflow from invisibly compensating for, for example, spatial male scarcity by inflating the fertility schedule.

Completed fertility is deliberately censored. Founders and model-born females whose configured reproductive window has not been fully observed are not treated as completed reproductive histories.

## 5. Implementation verification

The v1 report performs independent reconstruction rather than reading runtime counters. It:

- counts mortality exposures using interval-start age;
- verifies M2 death-event schedule probabilities;
- reconstructs annual same-day M4 origin locality;
- independently rebuilds eligible male sets;
- replays the fertility RNG stream only for true attempted draws;
- reconciles stochastic success/failure with authoritative Birth events;
- detects operational record-limit blocking; and
- reconciles reconstructed person chronology/residence/birth history with final Population state.

Acceptance tests added/strengthened in the same slice cover:

- model-born first-year and second-year mortality-band timing;
- founder mortality boundaries immediately below/on an age transition;
- fertility age-band boundary timing;
- annual relocation with an eligible male only at the origin;
- annual relocation with an eligible male only at the destination;
- equivalent non-annual relocation where elapsed destination residence legitimately changes parentage locality;
- 100% M2 mortality + 100% fertility, demonstrating that no dead female reaches the conditional fertility stage;
- ordinary synthetic-run replay and explicit `1278 -> 1460` spacing observability;
- end-to-end CLI derivation followed by exact `--check` verification.

Run-bundle integration is also part of this verification surface. `demography-observability.json` is a recognized optional bundle artifact; when present it is regenerated exactly before packing rather than trusted as an arbitrary sidecar. A tampered report is rejected.

During that integration review, #237 was discovered: completed-bundle founder reconstruction always used the synthetic initializer, which made a valid `declared_founder_state_v1` experiment fail the normal persisted run-directory validation path. PR #236 repairs reconstruction to dispatch by the experiment's initialization mode and requires the exact embedded declared-founder definition. An end-to-end declared-founder `--run-dir` regression now exercises the research-facing persistence path.

These tests are implementation/model-contract verification, not evidence that the demographic assumptions are empirically correct.

## 6. Model-output verification

The report now provides the denominators needed to diagnose age-specific mortality and fertility opportunity structure. It also exposes model-period interbirth intervals, declared founder-history-to-first-birth intervals and conservatively censored completed fertility.

This is necessary infrastructure for later comparison against empirical demographic targets, but no empirical target/tolerance is evaluated in this change. Therefore TRACE element 6 remains **not established empirically**.

## 7. Model analysis

This slice materially improves M2 model analysis because unexpected fertility can now be decomposed into structural and stochastic causes from preserved/versioned artifacts.

It does **not** complete the broader analysis programme. Still required for strong inference are, among other items:

- global/local sensitivity and interactions;
- uncertainty propagation;
- founder/initialization sensitivity;
- temporal structural sensitivity of annual versus future subannual demographic timing where consequential;
- spatial-resolution/boundary sensitivity;
- identifiability/equifinality analysis;
- empirical demographic output testing under predeclared targets.

## 8. Corroboration

None. No archaeological or anthropological corroboration is attempted or claimed.

## Issue-level closure interpretation

Once PR #236 is fully green on its latest head and merged, its evidence is sufficient to close the following implementation/research gates:

- **#179** — correct annual age interval plus full mortality/fertility boundary acceptance matrix;
- **#191** — annual birth-spacing quantization has one explicit executable meaning and requested/effective timing is machine-visible in ordinary run analysis; exact subannual 3.5-year execution is explicitly *not* claimed;
- **#193** — same-instant M4 relocation cannot create/erase preceding-interval parentage locality, with annual and non-annual controls;
- **#227** — mortality-first conditional-survival fertility is an explicit mathematical model contract and its opportunity consequences are observable/tested;
- **#228** — versioned M2 fertility/mortality opportunity denominators and rejection pathways are available from preserved run artifacts;
- **#237** — declared-founder day-zero state is reconstructed through the immutable founder definition in the completed-bundle pathway and has an end-to-end persisted-run regression.

**#201 remains open.** Its direct newborn-condition reset was repaired earlier, but its acceptance scope includes downstream M3 scarcity/condition and M4-pressure interactions and therefore belongs to the next resource/condition causal cluster rather than being declared complete here.

## Compatibility and references

This PR is intentionally semantics-neutral relative to v7. It adds downstream analysis, bundle validation and tests; it must not change authoritative event/state trajectories. Consequently:

- no model-semantics bump is appropriate;
- M7/M8/M9 exact references should remain unchanged;
- cross-platform determinism should remain unchanged;
- any authoritative-reference change discovered during this PR is a regression to investigate, not a reason to rebaseline.

## Remaining gate

M2 is substantially more interpretable after this slice, but AnthroSim is still not empirically research-ready. After the known P1 causal clusters are repaired, the corrected integrated model still requires post-fix adversarial audit convergence plus study-specific sensitivity, uncertainty, validation and corroboration before strong archaeological inference.
