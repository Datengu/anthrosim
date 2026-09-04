# Downstream analysis provenance v2

Status: normative research contract for GitHub issues #232 and #340, with evidence-independence integration hardened by Audit-v3 #423 / AV3-013, identifiability coordinate binding hardened by Audit-v4 #535 / AV4-011, and finalized study-result binding validation hardened by Audit-v4 #539 / AV4-012.

## Executable configuration contract

Schema v2 has exactly one authoritative representation of scripted command-line configuration: `command`, an ordered array containing the complete argv executed by the wrapper. There is no separate executable `arguments` object. The wrapper executes this array unchanged during both canonical `run` and isolated `replay`. Therefore changing a command-line scientific option changes execution itself; a provenance record cannot separately claim a different option value.

Optional `annotations` is a JSON object for descriptive, **non-executed** metadata such as an estimand label or prose-equivalent analysis role. It is definition/provenance-bound but must never be described as configuration that generated the result.

Analysis RNG seeds are subject to the same rule as every other executable option: if a seed affects the result, it must occur in the complete `command` argv or in a declared content-bound configuration artifact consumed by that command. Schema v2 deliberately has no independent `analysisRngSeeds` execution-claim field, because a separately editable seed list could contradict the seed the program actually used. A non-executed seed note may appear in `annotations`, but it is only descriptive and does not establish RNG execution provenance.

Observation-model selection follows the same rule. Schema v2 has no separate `observationModelIdentity` execution-claim field. If an observation model changes computation, its selector/identity must be present in the authoritative argv or a declared content-bound configuration artifact. A human-readable observation-model label may be retained in `annotations`, but it is descriptive rather than proof that the executable selected that model.

Configuration files are execution-bound in two independent ways: their path must occur in the authoritative `command` when the analysis consumes them, and the file must be declared as an input or implementation artifact so its exact bytes are SHA-256 bound. Isolated replay copies only declared artifacts, so undeclared configuration dependencies fail closed.

## Required definition

An `anthrosim-analysis-definition` v2 declares: `analysisId`, scientific status, execution mode, working directory, complete `command` argv, runtime/dependency description, exact reproduction criterion, input/implementation/environment/output artifacts, manual steps and optional non-executed `annotations`.

For `scripted` mode, `command` must be non-empty. For `external_or_manual`, `command` must be empty and `manualSteps` must describe the external transformation. Confirmatory analyses continue to require at least one machine-readable environment artifact.

The only v2 reproduction criterion is `exact_output_bytes`.

## Evidence-independence assessment as provenance input

When a confirmatory scientific result makes a held-out or independent-corroboration claim, the canonical `analysis/evidence-role-assessment.json` produced by `research-evidence-role-audit.py` should be declared as an analysis input. Audit-v3 #423 hardened that assessment so protocol-local evidence IDs are resolved through the EvidenceCatalog inside the exact frozen research definition and same-source aliases cannot evade the calibration/corroboration firewall.

The downstream analysis wrapper does not recreate that scientific judgement. Instead, normal v2 input fingerprinting binds the exact assessment bytes used by the analysis. This preserves a clean separation:

```text
frozen ResearchExperimentDefinition / EvidenceCatalog
  -> source-resolved evidence-role assessment (#206 / #423)
  -> downstream analysis definition and exact input digest (#232 / #340)
  -> analysis-provenance-v2-sha256-* record
```

If the frozen EvidenceCatalog source binding changes, evidence-role verification fails and the derived assessment changes. If the assessment bytes used by an analysis change, analysis provenance verification fails. Neither layer can silently inherit an independence claim from an unbound protocol-local string.

## Identifiability executed-design binding as provenance input

Audit-v4 AV4-011/#535 established that provenance for a downstream parameter table did not prove that its claimed coordinates were coordinates of the executed model design. `research-identifiability.py` now resolves real-study coordinates from the exact `anthrosim-research` root via `--research-root` and independently validates the redundant `research-manifest.json` / `research-plan.json` identities, expanded points, run configurations and run IDs before a positive identification claim is possible.

A scripted provenance definition for a real identifiability analysis must therefore:

- include `--research-root RESEARCH_ROOT` in the authoritative `command` argv;
- declare the exact identifiability plan and data table as inputs;
- declare `RESEARCH_ROOT/research-manifest.json` and `RESEARCH_ROOT/research-plan.json` as inputs, so the immutable design bytes from which the binding is re-derived are fingerprinted;
- declare `scripts/research-identifiability.py`, `scripts/research-identifiability-legacy.py` and `scripts/research-identifiability-bind-design.py` as implementation artifacts;
- fingerprint the produced identifiability result as an output.

The result itself records `executedDesignBinding.bindingIdentity`, source identity, `researchId`, `definitionIdentity`, point/execution counts and validation state. Thus the scientific chain is explicit:

```text
immutable research manifest/plan + exact run/point identities
  -> recomputed executed-design coordinate binding (#535)
  -> identifiability result containing the binding identity
  -> exact output fingerprint in analysis-provenance-v2
```

A provenance record can still prove only that this executable lineage was followed; the identifiability gate remains responsible for the scientific validity of the coordinate binding and inference decision.

## Finalized study-result binding validation (Audit-v4 AV4-012)

`study-result-binding.json` is a producer-defined self-identifying scientific artifact, not an opaque label that downstream tools may merely fingerprint. Before a downstream consumer treats it as authoritative, `scripts/research-study-result-binding.py` validates the schema-v1 producer contract and recomputes `resultIdentity` over the complete identity-covered projection: study execution and protocol identity/revision/status, pre-result binding eligibility, definition and research execution identities, exact source identity, research-relative root, completed/failed run counts, result-artifact paths/digests, and declared analysis requirements when present.

A stale edit to any identity-covered field therefore fails before a downstream result is published or accepted. For analysis provenance, where the full study root is available, validation goes further than internal self-consistency: the binding is resolved against the redundant frozen study plan/manifest and protocol/definition copies, recomputed protocol/definition/study execution identities, the exact producer-compatible schema-v1 `researchId`, redundant research manifest/plan, finalized research-state run counts, the exact bytes and FNV-1a digests of `research/analysis/points.json` and `runs.json`, and protocol-derived analysis requirements. Recomputing a new internally consistent `resultIdentity` after falsifying those authoritative artifacts therefore does not make the binding acceptable.

The schema-v1 research execution identity retains the producer's historical ordered `serde_json` encoding contract (`schemaVersion`, `definitionIdentity`, then `SourceRevisionIdentity` fields in producer order), whereas protocol, study-execution and study-result identities use their existing canonical identity contracts. The downstream verifier reproduces those producer rules exactly rather than silently defining a second identity algorithm.

Other consumers that already have their own authoritative domain checks—evidence-role assessment, Monte Carlo sufficiency and observable-support sensitivity—first require a producer-valid binding self-identity and then apply their existing protocol/seed/requirement checks.

Canonical `run`, `capture`, and `verify` require the full available frozen-root validation. `replay` first verifies that canonical root and provenance record, then copies only the already fingerprinted declared artifacts into its intentionally minimal isolated sandbox; inside that sandbox the copied binding is rechecked for producer-valid self-identity while exact input/code/environment/output replay guarantees remain unchanged.

This hardening changes research-governance validation only. It does not alter simulation trajectories, RNG semantics, model semantics, checkpoints, or scientific parameterization.

## Run, verify, and replay

`run` fingerprints the frozen study binding plus declared inputs, implementation, and environment; rejects pre-existing outputs; executes the exact `command` argv; re-fingerprints source artifacts; fingerprints outputs; and publishes `analysis-provenance.json`. `verify` recomputes definition/provenance identities and every artifact digest. `replay` reconstructs an isolated root from declared artifacts, executes the same complete `command` argv, and requires exact output bytes.

Definition and provenance identities use `analysis-definition-v2-sha256-*` and `analysis-provenance-v2-sha256-*`. Schema-v1 definitions/records are intentionally not accepted by the v2 wrapper because silently reinterpreting the old non-executed `arguments` field would recreate the ambiguity fixed by #340.

## Scientific boundary

This provenance layer establishes which frozen study, executable argv, configuration-file bytes, code, environment specification, RNG declarations, evidence-role assessment inputs, executed-design binding sources, and outputs form one reproducible downstream analysis lineage. It does not establish statistical validity, sufficient Monte Carlo precision, identifiability, empirical validity, evidence independence by itself, or archaeological truth; those remain separate research gates.

## Observable-support sensitivity analyses (Audit-v3 AV3-007)

The observable-support sensitivity gate uses this v2 wrapper as its execution proof. Primary and alternative support analyses must be scripted analyses with `executionStatus: executed_by_wrapper`; their exact plan-derived binning definition is a declared input and an argv token, and their support inference is a fingerprinted output. `research-observable-support-results.py` invokes the normal provenance verifier before accepting an `analysisIdentity`, so support-scale robustness cannot be certified by a fabricated identifier or an unexecuted declaration.
