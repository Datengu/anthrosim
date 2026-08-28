# Evidence-role firewall v1

**Status:** normative research-governance contract for GitHub issue #206.  
**Scope:** study evidence use and independence claims only; this does not change AnthroSim model dynamics.

AnthroSim distinguishes evidence provenance from evidence **use**. A source can be perfectly documented and still be used circularly if the same archaeological pattern is first used to tune a model and later described as an independent prediction.

Study protocol v1 (#230) already preserves the five TRACE evidence-use roles inside the frozen `StudyProtocol`. This contract adds the missing machine-auditable firewall around those declarations.

## The five TRACE roles

Every `evidenceRoles` assignment uses one of:

- `model_construction` — evidence informed the conceptual mechanism or model structure;
- `parameterisation` — evidence informed a parameter value, range, prior or external input;
- `calibration` — an observed/model-output target was used to fit, tune or select parameters/model variants;
- `model_output_verification` — model output was checked against evidence that is **not claimed to be independent** of model development/calibration;
- `independent_corroboration` — evidence was deliberately reserved for a later test of a declared observable.

These roles are study-specific. The same source may legitimately have different roles in different studies because every assignment is frozen inside that study's protocol identity.

## Target semantics

`evidenceRoles[*].target` is part of the scientific declaration, not a free comment.

For confirmatory protocols:

- `calibration`, `model_output_verification`, and `independent_corroboration` must target the exact `id` of a declared study observable;
- `model_construction` and `parameterisation` may target explicit non-observable identifiers such as `mechanism:m9_trigger` or `parameter:resources.baseProductivity`;
- calibration must name the **observable/pattern used as the fitting criterion**, rather than only naming the parameter that happened to be changed.

That last rule matters because circularity is about reuse of the observed target. If archaeological pattern X was used to tune a parameter, the calibration target is X's declared observable ID.

Qualitative/contextual evidence is therefore not forced into a numeric parameter link. It can be assigned to a conceptual mechanism, applicability-domain decision, structural choice or other explicit target with explanatory `notes`.

## Independent corroboration is a stronger claim

For a confirmatory study, every `independent_corroboration` assignment must have an exact matching entry in `heldOutCorroboration` with the same:

- `evidenceId`; and
- observable target (`target` = `observableId`).

The relationship is bidirectional: a held-out target without the matching independent role also fails.

For the same `(evidenceId, observableId)` pair, `independent_corroboration` may not coexist with any other evidence-use role. In particular, this rejects:

```text
evidence X -> calibration of observable Y
          -> independent corroboration of observable Y
```

It also rejects describing evidence already used for `model_output_verification` of Y as later held-out corroboration of Y.

If evidence was reused in model development or an in-sample fit, the scientifically honest downgrade is to declare `model_output_verification` (or the appropriate construction/parameterisation/calibration role) and omit the independent/held-out claim.

## Explicit reuse across different targets

One source can have multiple declared assignments. Repeated assignments are the machine-readable disclosure that reuse occurred.

Reuse across genuinely different targets is permitted. For example, one publication might parameterise `parameter:resourceProductivity` while a different measurement from that source is reserved to corroborate `occupancy_pattern`. The derived assessment reports every evidence ID with multiple roles/targets so reviewers can judge whether the asserted separation is substantively defensible.

The firewall enforces target-level non-circularity; it does not assert that target separation automatically makes two measurements statistically or conceptually independent. That remains a scientific judgement and should be explained in assignment `notes`.

## Exploratory studies

Exploratory work remains permissive. The audit validates role vocabulary, IDs and basic structure, but does not reject cross-role reuse as though the study had made a held-out confirmatory claim.

The output is explicitly labelled:

```text
assessmentStatus = exploratory_permissive
firewallEnforced = false
```

An exploratory result cannot acquire an independent-confirmatory status merely because its role table happens to look like a confirmatory one.

## Pre-execution validation

Before freezing a confirmatory study protocol:

```text
python scripts/research-evidence-role-audit.py validate protocol.json
```

A successful check prints the exact `study-protocol-v1-*` content identity computed with the same canonical FNV-1a64 identity rule used by `StudyProtocol`.

Then freeze the protocol normally:

```text
anthrosim-study prepare \
  --protocol protocol.json \
  --definition research-definition.json \
  --study-dir study/example
```

The role assignments and held-out declarations are now part of the immutable protocol identity and study execution identity. Changing them requires a new protocol/revision rather than silently relabelling evidence after seeing results.

## Finalized-study assessment

After `anthrosim-study finalize` has produced `study-result-binding.json`:

```text
python scripts/research-evidence-role-audit.py derive study/example
```

The tool verifies that:

1. `study-plan.json` and `study-manifest.json` agree;
2. the standalone `study-protocol.json` exactly equals the frozen protocol embedded in the plan;
3. the recomputed protocol content identity equals `protocolIdentity`;
4. `study-result-binding.json` belongs to that exact study/protocol/definition;
5. a confirmatory independent-corroboration claim is still marked as bound before execution and eligible for a pre-result confirmatory claim;
6. the evidence-role firewall passes for the exact frozen protocol.

It then writes:

```text
study/example/analysis/evidence-role-assessment.json
```

The assessment preserves:

- protocol identity;
- study execution and result identities;
- definition and research identities;
- pre-result binding/eligibility state;
- every evidence-role assignment;
- every held-out corroboration target;
- role counts;
- explicit multi-role/multi-target source reuse;
- one stable `evidence-role-assessment-v1-sha256-*` identity.

The write is immutable/idempotent: an existing different assessment is not overwritten. A changed scientific declaration belongs in a new study revision.

Verify later with:

```text
python scripts/research-evidence-role-audit.py verify study/example
```

Verification re-derives the assessment from the frozen study and requires exact equality, so changing either the assessment or its upstream study binding invalidates verification.

## Relationship to analysis provenance and archive integrity

The #206 assessment is a research-governance artifact. A canonical downstream analysis under #232 can include `analysis/evidence-role-assessment.json` as a declared input so the inferential output is explicitly linked to the evidence-independence assessment used for that claim.

The final study/deposit can then be sealed with the existing SHA-256 `research-integrity.py` layer.

These layers answer different questions:

```text
EvidenceCatalog
  -> what is this source and where did it come from?

StudyProtocol evidenceRoles (#230)
  -> how did this study declare that source would be used?

Evidence-role firewall (#206)
  -> does a claimed held-out target conflict with declared prior use?

Analysis provenance (#232)
  -> which executable analysis produced the reported result?

Research integrity
  -> did the preserved files change after archiving?
```

## Relationship to observation models

#209 remains responsible for defining how simulated state maps to archaeological observations. An observation model may itself use evidence during construction or calibration; those uses must be assigned the appropriate role rather than allowing its output to inherit an automatic independence claim.

A held-out archaeological target is independent only with respect to the declared prior uses in the frozen study. The firewall cannot prove causal/statistical independence from metadata alone.

## Model-semantics boundary

This contract changes no `ExperimentConfig`, M2/M3/M4/M8/M9 transition rule, RNG stream, checkpoint state, run identity, protected scientific reference, or `MODEL_SEMANTICS_ID`.

It changes only what research claims are permitted from a frozen evidence-use declaration.

## Acceptance meaning

For a finalized research study, a reviewer can recover one frozen table stating which evidence informed construction, parameterisation, calibration, ordinary output verification, and independent corroboration. A source used to fit an observable cannot be relabelled as independent corroboration of that same observable without failing the audit.

A successful fit remains scientifically useful; it is simply labelled as an in-sample/calibration or model-output-verification result rather than an independent prediction.
