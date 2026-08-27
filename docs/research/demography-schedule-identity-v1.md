# Demography whole-schedule identity v1

Status: normative research-readiness contract for issue #181.

This contract defines how evidence can support an AnthroSim demographic schedule without relying on positional array indices or a human-readable schedule label.

## Scientific object

For research evidence closure, the complete executable `DemographyConfig` schedule is treated as one scientific object. Its content identity includes:

- demographic schema version;
- ordered mortality bands, including age bounds and annual probabilities;
- ordered fertility bands, including age bounds and annual probabilities;
- minimum birth spacing;
- male birth proportion;
- male-parent minimum age;
- male-parent maximum age.

The identity excludes `scheduleId` itself to avoid circularity. It also excludes `provenance`: provenance describes what kind of claim is being made about the schedule, not what schedule the engine executes.

## Content-bound schedule ID

`DemographyConfig::content_bound_schedule_id()` computes a deterministic domain-separated 64-bit content identity and renders it as:

`anthrosim-demography-schedule-v1-<16 lowercase hexadecimal digits>`

The compact digest is a reproducibility and integrity identity, not a cryptographic authenticity or collision-resistance claim.

Synthetic/null-model schedules may retain descriptive IDs such as `synthetic_validation_v1`; they do not need a content-bound ID because no empirical closure is claimed.

For `empirical_direct`, `empirical_derived`, or `evidence_informed` demographic provenance, research evidence closure requires the stored `demography.scheduleId` to exactly match the current executable schedule contents.

## Evidence addressing

Whole-schedule evidence binds through the stable scalar path:

`demography.scheduleId`

This is deliberately different from linking to `mortalityBands.0`, `fertilityBands.3`, or any other positional array member. The schedule ID represents the complete canonical executable object, so evidence never depends on collection order as an addressing convention.

The evidence record linked to `demography.scheduleId` must still satisfy the ordinary evidence-closure rules for compatible provenance, reproducible source identity, and an explicit transformation for `empirical_derived` claims.

## Mutation semantics

Researchers should finish configuring the demographic schedule and then bind its identity, for example with `with_content_bound_schedule_id()` or by assigning the result of `content_bound_schedule_id()`.

Any later change to an executable demographic field makes the stored ID stale. Research readiness must then return `not_closed` with `unsupported_schedule_identity` until a new schedule identity is deliberately bound and its evidence relationship is reassessed.

This prevents evidence for one schedule from silently following a modified schedule.

## Whole-object evidence boundary

A single whole-schedule evidence record is legitimate only when the declared source or documented derivation genuinely supports the complete schedule object under the stated provenance claim.

This contract does not imply that one source is always sufficient scientifically. Where mortality, fertility, birth spacing, sex ratio, or parent-age assumptions come from distinct evidence or derivations, the researcher should preserve those sources and transformations appropriately. Future finer-grained stable semantic member IDs may be added if a study needs member-level closure, but positional JSON indices remain invalid scientific identities.

## Relationship to execution validity

This identity is a research-readiness rule, not an ordinary execution requirement. An exploratory, unresolved, or synthetic schedule can remain executable with a descriptive `scheduleId`.

Therefore AnthroSim continues to distinguish:

- executable configuration;
- content-bound demographic schedule identity;
- evidence-closed research claim;
- empirical/historical validity.

Passing the schedule-identity gate proves only that the evidence claim is attached to the exact executable schedule under the declared policy. It does not prove that the schedule is historically correct or sufficiently constrained for a particular archaeological inference.
