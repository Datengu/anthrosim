# Household lifecycle structural sensitivity v1

> **Superseded historical contract.** Audit-v2 issue #324 demonstrated that the v1 stable-`PersonId` partition rule makes packed record/birth order an unintended cohort-composition rule. Current executable treatment semantics are defined in `household-lifecycle-structural-sensitivity-v2.md`. This file and its v1 reference result are retained only to preserve the provenance of the original #207 comparison.

## Status

This is a **synthetic structural-sensitivity contract**, not an ethnographic or archaeological
model of household formation. It addresses TRACE audit issue #207 by making the historical
fixed-founder household lifecycle testable against one deliberately neutral alternative.

## Baseline

`fixed_founder_v1` is the historical AnthroSim rule. Founder household IDs persist for the run;
births join the female parent's current household; M3 shares resources at household level; M4
permanently relocates the living household; M9 temporary mobility treats the household as one
participant. No fission, dissolution, adult departure or transfer occurs.

This must not be interpreted as evidence that real households were permanent descent groups.
The persistence is a null-model structural assumption.

## Alternative

`deterministic_size_fission_v1` is enabled only through the optional versioned
`householdLifecycle` experiment field. At each completed annual boundary, after M2 fertility:

- only households physically at residence are eligible, avoiding ambiguous division of an
  active M9 journey;
- a household above `maxLivingMembers` is divided into the minimum number of groups required to
  satisfy the ceiling;
- group sizes are balanced as evenly as possible;
- living members are partitioned in stable `PersonId` order;
- all daughter households begin at the same persistent residence;
- person identity, genealogy, condition and residence are unchanged;
- past M9 triggers are marked processed for newly created households, while future triggers
  treat them as independent participants;
- M4 non-persistent household scratch arrays expand deterministically before the next decision.

Stable-ID partitioning is intentionally simple. It is not a claim about marriage, inheritance,
post-marital residence, age at departure or culturally specific household composition. Its role
is to ask whether scientific conclusions survive removal of permanent founder-group topology.

## Observability

`anthrosim-household-observability` derives a versioned checkpoint report containing:

- total and active household records;
- the living household-size distribution and maximum;
- active-household age distributions reconstructed from authoritative fission creation events;
- living genealogical-generation-span distribution and multi-generational household count;
- exact uniform household age for the fixed-founder baseline.

Existing authoritative/derived reports continue to provide the other #207 comparison targets:
M3 unmet need and condition, M4 move frequency and people moved, M9 journey participation plus
visitor person-days/household-days/peak aggregation, terminal population and spatial occupancy.
Household-fission events are authoritative solely so ordinary derived M9 replay can reconstruct
dynamic household topology; no explorer-only state becomes authoritative.

## First paired comparison

The repository example `household_lifecycle_sensitivity` runs eight paired seeds for 40 years
with the same founder population, replacement-control demography, M3/M4 assumptions, annual M9
schedule and synthetic world dimensions in both arms. The only structural treatment is the
household lifecycle. The preserved result pools terminal household size/age/generation-span
distributions and derives M9 aggregation through the standard observability replay. The machine-readable first result is preserved in
`research/household-lifecycle-sensitivity-v1/reference-result.json`; the generated interpretation
is in `docs/research/household-lifecycle-structural-sensitivity-result.md`.

The comparison is diagnostic only. A material effect means household lifecycle remains a
scientific model choice that must be propagated in claims using household sharing, permanent
migration or temporary aggregation. Lack of an effect for this one alternative would establish
robustness only to this declared contrast, not validate either lifecycle historically.

## Compatibility / semantics review

The historical `None` lifecycle path executes the pre-#207 rule and preserves its serialized
omission. The new field is included in full experiment and continuation identity when enabled.
No existing parameter is reinterpreted, so the repository model-semantics identity is not
advanced solely for this opt-in structural treatment. Exact Git provenance and full experiment
identity distinguish runs, and continuation integrity binds the configured lifecycle.
