# M2 demographic observability contract v1

Status: versioned derived-analysis contract for the current annual M2 model. This report improves implementation verification and demographic model analysis; it is **not empirical validation**.

## Purpose

A total birth count is not enough to explain M2 behavior. Under the current annual model, a female record may fail to produce a birth because she does not survive the M2 mortality transition, is outside a non-zero fertility schedule band, is still inside the executable birth-spacing interval, has no eligible male in the relevant persistent-residence pool, fails the stochastic fertility draw, or reaches the operational person-record ceiling.

`DemographyObservabilityReport` preserves those pathways separately so a researcher cannot silently compensate for one bottleneck by inflating another parameter.

The report is derived **downstream** from authoritative run artifacts. It does not add causal state and it does not emit one event for every rejected opportunity.

## Version binding

Report schema v1 is bound to the repository's current `MODEL_SEMANTICS_ID`. The derivation fails closed when asked to interpret a checkpoint created under another model-semantics identity.

This is deliberate. Re-implementing old M2 rules inside a current analysis binary would create a second, potentially drifting model.

## Inputs and reconstruction

The derivation consumes:

- the exact day-zero Population;
- the immutable experiment configuration contained in the checkpoint;
- authoritative EventLog history through the checkpoint boundary; and
- final Population state from the checkpoint.

It reconstructs persistent household movement, deaths and births through time. At each annual M2 boundary it independently applies the v1 demographic-time contract:

1. count mortality exposures using age at the start of `[t-365,t)`;
2. apply recorded M2 demographic deaths and verify their recorded schedule probability;
3. identify surviving female fertility exposures;
4. apply non-zero age-schedule eligibility;
5. apply requested-to-executable birth-spacing eligibility;
6. reconstruct the pre-same-day-M4 persistent-residence parentage pool;
7. identify local eligible-male availability;
8. replay the independent `demography/fertility` RNG stream for the exact attempted draws;
9. reconcile draw success/failure with authoritative Birth events and any person-record-limit block; and
10. reconcile the reconstructed demographic history with final Population state.

A report that cannot reconcile these artifacts is an error rather than a best-effort estimate.

## Fertility opportunity funnel

The report exposes, globally and by configured fertility age band:

- `survivingFemaleExposures` — female records alive after that boundary's M2 mortality transition;
- `ageScheduleEligible` — those exposures whose configured fertility probability is non-zero;
- `spacingEligible` — those whose previous model-period or declared pre-run birth is at least the executable spacing away;
- `localMaleEligible` — those with at least one eligible living male in the parentage locality defined by the M2 time contract;
- `fertilityDrawsAttempted`;
- `fertilityDrawSuccesses`;
- `stochasticDrawFailures`;
- `successfulBirths`; and
- `recordLimitBlockedBirths`.

For an untruncated run the following accounting identities should hold:

`fertilityDrawsAttempted = fertilityDrawSuccesses + stochasticDrawFailures`

`fertilityDrawSuccesses = successfulBirths + recordLimitBlockedBirths`

The first stages are nested opportunity denominators rather than mutually exclusive rejection counts. Differences between adjacent stages show the corresponding structural suppression.

## Mortality observability

For every configured mortality age band, the report records:

- number of annual exposures; and
- number of M2 demographic deaths.

Age is evaluated at interval start, matching the authoritative M2 model rather than recomputing age at the boundary end.

Resource-scarcity deaths remain a separate M3 cause and are not counted as M2 demographic deaths.

## Birth spacing

The report always surfaces both:

- `requestedBirthSpacingDays`; and
- `effectiveBirthSpacingDays`.

Under the current annual scheduler:

`effective = ceil(requested / 365) * 365`, with zero mapping to zero.

Thus the current synthetic request of `1278` days is explicitly reported alongside its executable `1460`-day minimum. The report must not describe the model as executing an exact 3.5-year postpartum interval.

## Parentage locality

On an annual boundary that also contains an M4 relocation, male availability is reconstructed from the household's persistent residence immediately **before** that same-day M4 move. A zero-duration destination therefore cannot create or erase the preceding interval's parentage opportunity.

A relocation on an earlier, non-annual M4 boundary changes subsequent parentage locality normally because time has elapsed in the destination before the next M2 boundary.

M9 temporary visitor/transit presence remains excluded.

## Interbirth intervals

Two distributions are kept distinct:

- `modelPeriodInterbirthIntervals` — intervals between two births both created by this run; and
- `declaredPrerunToFirstBirthIntervals` — the interval from an explicitly declared founder `lastBirthDay` to that female's first model-period birth.

Synthetic founders without declared reproductive history cannot supply the second quantity and must not be treated as if their unobserved pre-run history were zero births.

## Completed fertility and censoring

The v1 completed-fertility distribution is intentionally conservative. A female is counted as **uncensored** only when:

- she was born during the modeled period rather than entering as a pre-run founder; and
- her observation extends through the end of the configured age range having non-zero fertility probability.

All founders and model-born females whose reproductive window is incomplete are reported as censored and excluded from the completed-fertility distribution.

This prevents partial reproductive histories from being silently interpreted as completed family size. It does not solve survivorship-selection or empirical sampling questions; those belong to study-specific validation.

## Operational truncation

If a successful fertility draw encounters the configured persistent-person-record ceiling, the report records an operationally blocked birth and marks fertility-stage truncation. That ceiling is an engineering stop, not demographic regulation.

Runs truncated operationally must not be used as ordinary demographic outcomes without explicit handling.

## CLI and run-bundle pathway

A normal run directory can be analyzed with:

```text
anthrosim-demography-observability --run-dir <run-directory>
```

By default this writes `demography-observability.json` inside the run directory.

`demography-observability.json` is a recognized optional AnthroSim run-bundle artifact. When present, bundle validation does not merely include the file by name: it reconstructs the exact day-zero population from the immutable experiment identity, regenerates the report deterministically from the checkpoint/events, and requires exact equality before the artifact is accepted for packing. Declared-founder runs are reconstructed through their embedded `founderPopulation` definition rather than through the synthetic initializer.

An existing report can also be independently checked against the preserved artifacts with:

```text
anthrosim-demography-observability --run-dir <run-directory> --check <report.json>
```

The check is exact: any report difference or replay/reconciliation failure is an error.

## Verification versus validation

This report can establish that:

- the executable spacing is visible rather than hidden;
- M2 age-band exposures follow the declared interval semantics;
- mortality/fertility competing-transition behavior is quantitatively inspectable;
- locality/spacing/stochastic/operational suppression can be separated;
- the derived analysis agrees with authoritative state and events;
- a preserved report remains tied to the exact run artifacts that generated it.

It cannot establish that:

- any mortality or fertility schedule is correct for a real population;
- the local-male parentage rule is anthropologically realistic;
- annual discrete competing transitions are an adequate approximation for a particular study;
- the current synthetic founder distribution represents a real community;
- observed simulated fertility, survivorship or growth has been empirically validated.

Those require explicit empirical targets, uncertainty analysis, sensitivity analysis and appropriate corroboration under the TRACE research-readiness process.
