# Strict JSON Contract Policy v1

## Status

This document defines the target fail-closed parsing policy for AnthroSim machine-readable scientific inputs and versioned research artifacts. It advances issue #174 by fixing the interpretation contract before parser hardening is applied incrementally.

This policy does not by itself make every existing parser strict. Until the implementation coverage described below is complete, repository documentation and release/readiness claims must not imply that unknown or duplicate fields are universally rejected.

## Principle

A versioned AnthroSim scientific JSON object is a closed contract unless its schema explicitly declares an extension container.

For a closed object:

- every object member must be defined by the declared schema version;
- an unrecognized member is an error, even if the rest of the document is valid;
- duplicate object keys are an error;
- misspelling an optional/defaulted field must not silently select the default behavior;
- parsing and validation must fail before simulation execution, artifact acceptance, benchmark comparison, or scientific analysis continues;
- deserialize/re-serialize normalization must not erase unsupported members and thereby convert an invalid document into an apparently valid artifact.

Forward compatibility is achieved through an explicit schema-version change and migration policy, not by silently ignoring fields from a newer or misspelled contract.

## Explicit extensibility

If a contract intentionally permits user-defined or future extension keys, that flexibility must be represented as a named, typed extension field (for example an explicit map) whose key/value rules are themselves documented and validated.

No parser may treat Serde's default unknown-field tolerance, Python dictionary permissiveness, or last-value-wins duplicate-key behavior as an extension mechanism.

## Rust/Serde implementation rule

Closed Rust structures deserialized from scientific JSON should use `#[serde(deny_unknown_fields)]` or an equivalent parser boundary that proves the same property.

Hardening should cover, at minimum:

1. experiment configuration and all nested subsystem configurations;
2. world/population/resource/migration state and checkpoints;
3. M8 landscape, evidence, spatial mechanism and binding contracts;
4. M9 focal-region, travel, schedule and temporary-mobility contracts;
5. manifests, events, metrics and resume/provenance state;
6. ensemble/sweep definitions and statuses;
7. derived observability artifacts consumed as versioned scientific data.

Tagged or flattened enums must be reviewed individually before applying strictness so intentional variant representation is preserved while unsupported fields still fail closed.

## Python implementation rule

Python tooling that accepts a versioned scientific definition, benchmark/reference artifact, provenance record, or integrity-sensitive JSON document must use a loader that detects duplicate keys and must validate the exact allowed key set for closed objects.

Plain `json.load`/`json.loads` followed only by selected-key access is insufficient for a closed scientific contract because unknown keys are retained without semantic validation and duplicate keys normally use last-value-wins behavior.

## Required negative fixtures

Each hardened contract family should include tests for the failure modes that matter scientifically, rather than only generic junk fields. At minimum the suite should prove rejection of:

- a misspelled optional M9 field such as `temporaryMobliity`;
- a misspelled or unsupported evidence field that would otherwise default/vanish;
- an unknown parameter beside otherwise valid configuration;
- an unexpected field in a persisted/versioned artifact;
- duplicate keys for a scientifically consequential value;
- an artifact that would otherwise deserialize and re-serialize while silently dropping an unsupported field.

Failures must occur at the earliest responsible input/artifact boundary, before execution or downstream interpretation.

## Compatibility and schema changes

Making a currently permissive parser strict is intended to reject documents that were outside the declared contract even if older code happened to tolerate them. It should not change the meaning or serialized identity of documents containing only supported fields.

If implementation reveals that a previously tolerated field was intentionally part of the public contract, the correct response is to declare and version that field explicitly rather than preserve accidental unknown-field acceptance.

No M8/M9 scientific reference result should be rebaselined merely because malformed documents are newly rejected. A reference change is appropriate only if valid executable semantics change for a separate, reviewed reason.

## Research-readiness gate

AnthroSim must not claim universal strict scientific JSON validation until the implementation audit shows that every relevant input/artifact boundary either:

- enforces this closed-object rule; or
- explicitly documents a typed extension mechanism and its validation.

Issue #174 remains open until that coverage and the required adversarial tests are complete.
