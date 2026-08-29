# Downstream analysis provenance v2

Status: normative research contract for GitHub issues #232 and #340.

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

## Run, verify, and replay

`run` fingerprints the frozen study binding plus declared inputs, implementation, and environment; rejects pre-existing outputs; executes the exact `command` argv; re-fingerprints source artifacts; fingerprints outputs; and publishes `analysis-provenance.json`. `verify` recomputes definition/provenance identities and every artifact digest. `replay` reconstructs an isolated root from declared artifacts, executes the same complete `command` argv, and requires exact output bytes.

Definition and provenance identities use `analysis-definition-v2-sha256-*` and `analysis-provenance-v2-sha256-*`. Schema-v1 definitions/records are intentionally not accepted by the v2 wrapper because silently reinterpreting the old non-executed `arguments` field would recreate the ambiguity fixed by #340.

## Scientific boundary

This provenance layer establishes which frozen study, executable argv, configuration-file bytes, code, environment specification, RNG declarations, and outputs form one reproducible downstream analysis lineage. It does not establish statistical validity, sufficient Monte Carlo precision, identifiability, empirical validity, or archaeological truth; those remain separate research gates.
