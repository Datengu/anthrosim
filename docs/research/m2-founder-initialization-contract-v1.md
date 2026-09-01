# M2 founder initialization contract v1

**Status:** normative scientific contract for M2 initialization  
**Applies to:** post-M9 scientific-hardening line; founder-history reconciliation hardened at model semantics v23  
**Issues repaired:** #192, #396  
**Scientific status:** mechanism/initial-state verification only; not empirical validation

## 1. Purpose

This contract defines what AnthroSim is allowed to assume about people who already exist at simulation day 0.

The problem addressed by #192 is not merely that the old synthetic founder age distribution was simple. The more serious problem was that every founder also began with an implicit **zero pre-simulation reproductive and genealogical history**. That can create artificial early-run behaviour:

- a female founder can appear immediately eligible for a birth-spacing decision because no earlier birth is remembered;
- M4's direct-parent kin proxy can begin empty even when the represented population is intended to have pre-existing living kin;
- early model behaviour can therefore reflect the arbitrary start date rather than the represented system.

The repair separates two scientifically different initialization modes rather than silently making the synthetic mode more elaborate.

Audit-v3 issue #396 closes a second boundary problem inside the declared mode: explicit child genealogy and `lastBirthDay` are not independent facts. A declaration must not claim an old "last" birth while simultaneously representing a later child of that same female, and omission of the optional field must not erase a birth already made explicit by the genealogy.

## 2. Initialization modes

### `synthetic_validation_v1`

This remains the frozen engineering/validation founder generator.

It uses deterministic synthetic rules for founder age, reproductive sex, household grouping and household location. Its purpose is software verification, regression testing, benchmarks and explicitly synthetic/null-model experiments.

It is **not** an empirical population reconstruction and must not be treated as a neutral prehistoric prior. In particular, its founders still have no declared pre-run reproductive history or living direct-parent genealogy.

Existing synthetic serialized experiment identity and deterministic trajectories are intentionally preserved by this repair wherever the new mode is not selected.

### `declared_founder_state_v1`

This mode accepts an explicit `FounderPopulationDefinition` and materializes that state exactly rather than drawing founder age, sex, household or residence from the synthetic initializer.

It is an **initial-state transport contract**, not an equilibrium generator. AnthroSim does not infer that the supplied founder state is representative, stable, empirically supported or historically correct merely because it is structurally valid.

A declared founder state becomes suitable for a real research configuration only when the procedure that produced the declaration is itself justified and its uncertainty is analysed.

## 3. Authoritative declared state

A founder declaration contains:

- a versioned schema;
- a non-empty `initializationId` identifying the initialization definition;
- a coarse `ParameterProvenance` label;
- a genealogy-completeness statement;
- canonical one-based household IDs and their world-cell residences;
- canonical one-based person IDs;
- signed epoch-relative birth days;
- reproductive sex used by the current M2 reproduction mechanism;
- household membership;
- optional living female-parent and male-parent links;
- optional signed pre-run `lastBirthDay`, which may represent a birth whose child is not one of the living founder records;
- initial condition in permille;
- a deterministic serialized `contentDigest64` over all of the preceding scientifically consequential founder-definition content.

The declared list must contain exactly the configured initial population and must fit the persistent person-record ceiling.

Synthetic-only fields such as `targetHouseholdSize`, `syntheticMaxAgeYears` and `syntheticMalePermille` do not modify a declared founder population.

A standalone founder-definition JSON may omit `contentDigest64`; loading it seals the declaration to the content observed at that input boundary. Serialization into an experiment, run manifest or checkpoint always writes the current deterministic digest.

## 4. Signed chronology

Simulation day 0 is the epoch.

Founder birth days are signed integers:

- `0` is permitted for a founder born exactly at the epoch;
- negative values represent days before the run;
- positive founder birth days are invalid.

A declared pre-run `lastBirthDay` must be strictly negative and strictly later than the founder's own birth day. It must also occur at a female age supported by the experiment's declared fertility schedule, as specified below.

Explicit living child records are also authoritative reproductive chronology. When founder child `C` names founder female `F` as `femaleParent`, `C.birthDay` proves that `F` had a birth on that day. Therefore:

- if `F.lastBirthDay` is present, it must be **equal to or later than** the latest explicitly represented child birth of `F`;
- equality means the latest known birth is that represented child;
- a later `lastBirthDay` is permitted and explicitly means that a more recent birth is known even though that child is not represented among the living founder records (for example because the child is dead, outside the represented population, or otherwise unrepresented);
- omission remains permitted when no additional unrepresented birth history is being asserted, but omission does **not** erase the birth dates already established by explicit child genealogy.

A `lastBirthDay` older than a later explicit child is internally contradictory and fails closed.

This signed chronology exists because pre-simulation events cannot be represented truthfully by the runtime fields that record events occurring during the simulation.

### Schedule-relative reproductive-age validity

Declared pre-run reproductive chronology is validated against the **same `DemographyConfig` carried by the experiment**. AnthroSim does not introduce a separate universal human reproductive-age constant at the founder boundary.

For a declared birth event, age is the exact signed-day difference between the founder birth day and the declared event day, interpreted in completed 365-day model years. The event is structurally valid only when:

- a declared **female parent** is in a configured fertility age band whose `annualProbabilityPerMillion` is greater than zero at the child's birth day;
- a declared **male parent** is within `[maleParentMinAgeYears, maleParentMaxAgeYearsExclusive)` at the child's birth day; and
- a female founder's declared pre-run **`lastBirthDay`** is in a configured fertility age band with positive fertility support.

This is a schedule-consistency and biological-plausibility boundary, not a claim that the configured age ranges are universal or empirically correct. A research configuration that changes reproductive-age assumptions changes which founder histories are admissible, and those assumptions retain the provenance of the declared demographic schedule. The check also does not claim that a pre-run event would have been generated on an exact annual M2 scheduler boundary; founder chronology predates model execution and is validated for reproductive-age support at the declared event day.

The default synthetic validation schedule therefore accepts female reproductive events from completed age 18 through the day before completed age 45, and male parentage from completed age 18 through the day before completed age 70. A one-day-old parent is invalid.

## 5. Reproductive-history semantics

Pre-run birth timing affects the first and later M2 spacing decisions without creating fictitious child records.

For each living female at an annual demographic boundary:

1. if the runtime population contains a successful model-period birth for that person, that model-period birth day is the authoritative spacing reference;
2. otherwise the founder declaration supplies the **latest known founder birth**: the later of the optional `lastBirthDay` and the birth day of any explicitly represented founder child naming that female as `femaleParent`;
3. if neither source contains a known birth, there is no known prior-birth spacing constraint.

Because contradictory stale `lastBirthDay` values are rejected during founder validation, the second rule never uses an older optional field to override a newer explicit child. If the optional field is later than every represented child, that later day remains authoritative and represents an unrepresented prior child.

Once a model-period birth occurs, it naturally supersedes founder pre-run timing for later spacing decisions.

The declaration does **not** increment `birthsSinceStart`, does not emit a `Birth` event and does not invent an unobserved pre-run child. An explicitly represented founder child is already part of the day-0 population, not a model-period birth event. This preserves the distinction between declared initial conditions and simulated history.

## 6. Genealogy and M4 kin semantics

The current M4 kin term is a deliberately narrow proxy based on living direct-parent locations. It is not a full kinship network, lineage system or anthropological theory of kin relations.

`FounderGenealogyStatus` therefore has two meanings:

### `unspecified`

Missing founder parent links are epistemically unknown for the purpose of the current kin mechanism. They must not be interpreted as evidence that no living direct parent exists in the represented founder population.

If permanent migration is enabled with non-zero `kinWeight`, a declared founder run with `unspecified` genealogy fails closed before execution.

### `complete_living_direct_parents`

For the scope of M4's current direct-parent proxy, omitted parent links explicitly mean that no living direct parent represented in this founder population is declared for that parent role.

This statement is deliberately narrow. It does **not** claim:

- complete ancestry;
- complete sibling/cousin/affinal relationships;
- a culturally meaningful kin category;
- known dead ancestors;
- a complete historical genealogy outside the living founder population.

Declared direct-parent links are materialized into authoritative Population state at day 0. M4 therefore sees them on its first eligible migration boundary rather than waiting for model-born generations to create kin state.

## 7. Household, residence and condition semantics

Declared households and their residences are exact initial conditions. Every declared household must be used by at least one founder and must reference a valid world cell.

Living founder person residence is derived from the declared household residence at initialization.

Initial `conditionPermille` is also materialized exactly. This does not resolve the broader scientific interpretation of the shared `condition` state; research use remains subject to the active condition/resource semantics and their own validation requirements.

## 8. Provenance and evidence

`initializationId`, `ParameterProvenance` and the complete founder contents are part of immutable experiment identity. The full founder declaration is serialized with the experiment and therefore preserved in manifests/checkpoints.

The provenance enum is descriptive metadata, not proof. A declaration labelled `empirical_direct`, `empirical_derived` or `evidence_informed` is not automatically research-valid merely because that label is present.

Evidence closure for empirical parameters/initial conditions remains part of the broader research-readiness/evidence work tracked separately (including #181). A real study should record:

- the data or model used to derive founder ages/sexes/households/locations/condition;
- how pre-run reproductive histories were estimated or sampled;
- what direct-parent information is genuinely known versus assumed;
- transformations and uncertainty;
- whether the founder state was calibrated to any output later used for validation;
- alternative plausible founder states for initialization sensitivity.

## 9. Determinism, content identity and checkpoint/resume

Declared founder initialization uses no synthetic founder RNG draws. Given the same world and declaration, materialization is deterministic.

The full declaration remains embedded in `ExperimentConfig`, which is persisted into run manifests and checkpoints. Checkpoint resume revalidates the declaration against the reconstructed world and continues to use its reconciled latest-known-birth history where no later model-period birth has superseded it.

`contentDigest64` is computed deterministically from the founder-definition schema, initialization ID, provenance, genealogy-completeness status, household IDs/residences, person IDs, birth chronology, reproductive sex, household membership, parent links, pre-run birth history and initial condition. The digest field itself is excluded from its own calculation. The effective latest-known birth is derived entirely from already-digested fields: explicit child `birthDay`/`femaleParent` relationships and optional `lastBirthDay`.

A definition loaded from serialized form remembers the digest supplied by that artifact. If its otherwise-valid founder content is later changed without the corresponding integrity metadata being deliberately rewritten, validation fails with a content-identity mismatch. This closes the checkpoint loophole in which future-causal pre-run history could otherwise be altered while ordinary runtime Population state remained unchanged.

The digest is a compact deterministic reproducibility/integrity identity. It is **not** a cryptographic signature, authenticity proof or protection against a person deliberately rewriting both content and integrity metadata.

Audit-v3 #396 changes the causal continuation of some previously accepted declared-founder checkpoints: a known explicit child can now constrain first-boundary spacing even when `lastBirthDay` was omitted, and contradictory stale values are rejected. `MODEL_SEMANTICS_ID` therefore advances from v22 to **v23**. A v22 checkpoint cannot silently continue under v23 as though scientific semantics were unchanged.

The repair does not alter the existing synthetic founder RNG mapping or legacy synthetic runtime state digest when `synthetic_validation_v1` is selected.

## 10. Fail-closed rules

The core rejects the following configuration mismatches:

- `declared_founder_state_v1` without a founder definition;
- `synthetic_validation_v1` carrying a founder definition;
- a founder definition whose counts/IDs/chronology/parent relationships/households/locations/condition are invalid;
- a declared parent or pre-run birth event whose parent age lies outside the experiment's configured reproductive-age support;
- a female founder `lastBirthDay` that predates a later explicitly represented child naming her as mother;
- a serialized founder definition whose remembered content identity no longer matches its scientifically consequential contents;
- active non-zero M4 kin weighting with declared genealogy marked `unspecified`.

These rules prevent a malformed or incomplete research-facing initialization from silently falling back to synthetic/zero-history behaviour or internally contradictory reproductive chronology.

## 11. Verification evidence required by this contract

The implementation must retain regression tests demonstrating at least:

- synthetic initialization remains deterministic and available as an explicitly synthetic mode;
- declared founders materialize exact declared ages, sexes, households, residences, condition and parent links;
- changing synthetic-only founder knobs does not change declared state;
- declared mode cannot silently fall back to synthetic initialization;
- invalid founder chronology/genealogy is rejected, including female/male parent ages immediately below, at and above configured reproductive-age boundaries;
- pre-run `lastBirthDay` is rejected below/above configured female fertility support and accepted on supported boundaries;
- a `lastBirthDay` older than the latest explicitly represented child of that female is rejected;
- `lastBirthDay` equal to the latest represented child remains valid;
- a later `lastBirthDay` remains valid as an explicitly declared unrepresented birth;
- omission of `lastBirthDay` still uses a known represented child's birth day for first-boundary spacing;
- custom fertility schedules change founder reproductive-history acceptance consistently rather than being overridden by a hidden universal age constant;
- a sufficiently recent known pre-run birth blocks an otherwise certain first-boundary fertility opportunity;
- a sufficiently distant pre-run birth permits that opportunity;
- no fictitious pre-run birth appears in runtime birth accounting/events;
- declared living direct-parent state can affect M4 on its first migration boundary;
- kin-sensitive declared runs fail closed when genealogy completeness is unspecified;
- checkpoint/resume preserves and revalidates the reconciled declared founder history and matches uninterrupted execution;
- serialized founder content identity changes when genealogy, residence, condition or pre-run birth history changes;
- valid post-load mutation of sealed founder content is rejected rather than silently changing future behaviour.

These are implementation/conceptual verification tests. They are not empirical demographic validation.

## 12. What this repair does not solve

`declared_founder_state_v1` deliberately does not solve every initialization problem.

Still outside this contract are:

- automatic derivation of a stable/quasi-stable age structure from a demographic schedule;
- burn-in generation of endogenous prehistory;
- uncertainty distributions over founder state inside one declaration;
- complete kinship/household lifecycle reconstruction beyond the explicitly declared founder links;
- inference of unrepresented births beyond an explicitly supplied `lastBirthDay`;
- observation/taphonomic inference from archaeological evidence to individual founder records;
- proof that any supplied founder population is historically representative;
- general empirical evidence closure and study-specific research readiness.

A future schedule-consistent generator may produce a `FounderPopulationDefinition`, but its scientific validity would depend on the generator's own documented assumptions and validation.

## 13. Research-use rule

For inferential work, the question is no longer "did AnthroSim secretly assume every founder had no past?" The declared mode can now represent relevant pre-run state explicitly, and known genealogy cannot be silently contradicted or ignored by the separate `lastBirthDay` field.

The remaining question is scientifically harder and must stay visible:

> **Why is this particular founder state a defensible representation of the uncertainty at the study boundary, and does the conclusion survive other plausible initial states?**

A declared founder state is therefore a necessary mechanism for removing hidden zero-history assumptions, not a certificate of empirical adequacy.
