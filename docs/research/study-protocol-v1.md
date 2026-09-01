# Frozen study protocol v1

**Status:** normative research-governance contract for GitHub issue #230.  
**Scope:** study-level interpretation/provenance only; this does not change AnthroSim model dynamics.

AnthroSim can reproduce an exact simulation and still permit post-hoc scientific flexibility. A researcher could keep the same runs while changing which outcome is called primary, which interval is analysed, which runs count, what threshold supports a hypothesis, or which evidence is described as independent corroboration. Reproducible execution alone therefore does not make an analysis confirmatory.

Study protocol v1 adds a frozen research-governance layer above the exact experiment definition introduced by #205 and the analysis-window semantics introduced by #219.

## Two identities, deliberately separate

A study has two different kinds of identity:

1. **simulation/research-definition identity** — what AnthroSim actually simulates, including any EvidenceCatalog embedded in the base experiment;
2. **study-protocol identity** — what scientific question, hypotheses, observables, analysis rules and evidence-use declarations the researcher intends to apply.

Changing a decision threshold, primary observable or evidence role changes the study-protocol identity. It does **not** alter the underlying `ResearchExperimentDefinition`, per-run configuration or deterministic simulation state.

For evidence-bearing confirmatory work, those two identities are nevertheless checked together: protocol-local `evidenceId` references must resolve to records in the EvidenceCatalog preserved by the exact bound research definition. The protocol cannot manufacture a second evidence namespace independent of the scientific configuration.

This separation permits honest reuse of the same deterministic simulation outputs for a new exploratory analysis while preventing that new interpretation from masquerading as the original predeclared confirmatory analysis.

## Protocol fields

`StudyProtocol` schema v1 preserves:

- `studyId` and `protocolRevision`;
- scientific `status`: `exploratory` or `confirmatory`;
- research question and applicability domain;
- competing hypotheses/null models;
- named analysis windows with explicit selection rules and rationales;
- primary and secondary observables, each linked to a declared analysis window;
- predeclared comparisons, predictions and decision criteria;
- study-specific evidence-role assignments;
- parameter and structural uncertainty plans;
- seed, pairing and replication policy;
- stopping, exclusion and censoring rules;
- sensitivity and equifinality plans;
- manipulation/treatment-realization checks;
- analysis method and multiplicity policy;
- held-out corroboration targets;
- permitted and prohibited interpretations;
- explicit amendment provenance for revisions after v1.

The TRACE evidence-role vocabulary represented here is `model_construction`, `parameterisation`, `calibration`, `model_output_verification`, and `independent_corroboration`. The frozen protocol records those assignments; the dedicated #206 evidence-role firewall in `docs/research/evidence-role-firewall-v1.md`, hardened by Audit-v3 #423 / AV3-013, resolves those assignments through the bound EvidenceCatalog and rejects circular held-out/independent-corroboration claims by immutable source identity rather than by protocol-local aliases.

## Confirmatory validation

A protocol labelled `confirmatory` must contain at least:

- two competing hypotheses/models;
- one primary observable;
- one predeclared comparison;
- at least two hypotheses and one observable in every confirmatory comparison;
- every hypothesis represented in a comparison;
- every primary observable represented in a comparison.

IDs and references are validated strictly. Unknown analysis-window, hypothesis or observable references fail closed. The schema does not pretend that a syntactically complete protocol is scientifically good; it makes omissions, changes and declared decisions auditable.

Evidence-role circularity is a distinct study-level validation concern. Before freezing a confirmatory protocol that declares evidence roles or held-out corroboration, validate the protocol against the **same research definition that will be frozen**:

```text
python scripts/research-evidence-role-audit.py validate \
  protocol.json \
  --definition research-definition.json
```

A confirmatory evidence-bearing protocol without that definition fails closed. Every referenced `evidenceId` must resolve to a real record in `base.experiment.evidence`, and same-source/same-observable circularity is checked using a canonical identity derived from the immutable `EvidenceRecord.source` object. Unknown/fabricated IDs and source aliases cannot receive independent-corroboration status.

After finalization, derive and preserve the assessment against the exact frozen study/result/definition binding. This keeps the `StudyProtocol` schema stable while making the stronger #206/#423 independence rules machine-auditable.

## Analysis windows

Each observable references a named analysis window containing `analysisStartDay`, optional `analysisEndDayInclusive`, one explicit selection rule and a rationale. The allowed selection rules mirror #219: `predeclared_fixed_duration`, `convergence_diagnostic`, `externally_meaningful_historical_start`, `initial_state_in_scope`, and `other_explicit`.

The study protocol freezes the intended window. The #219 tooling remains responsible for resolving a declared interval against realized run duration and for preventing cumulative since-start metrics from being silently relabelled as post-burn-in totals.

## Immutable preparation workflow

For confirmatory work, create a fresh study directory **before executing the research experiment**. For an evidence-bearing protocol, first run the source-binding validation above, then freeze the exact validated files:

```text
anthrosim-study prepare \
  --protocol protocol.json \
  --definition research-definition.json \
  --study-dir study/example
```

Preparation validates both machine-readable objects and writes:

```text
study/example/
  study-plan.json
  study-manifest.json
  study-protocol.json
  research-definition.json
```

`study-plan.json` and `study-manifest.json` are redundant exact copies of the immutable pre-execution plan. They bind the complete protocol and `protocolIdentity`, the exact research definition and `definitionIdentity`, executable `SourceRevisionIdentity`, a `studyExecutionId` derived from protocol + definition + source, the fixed child research directory `research/`, and whether the protocol is eligible to be described as a pre-result confirmatory declaration.

The bound research definition contains the authoritative EvidenceCatalog used by the evidence-role firewall. Because that catalogue is inside `ExperimentConfig`, changing its records or source metadata changes the enclosing research-definition identity rather than silently changing an evidential claim beneath the same study execution.

The study directory must be empty when prepared. Protocol changes do not overwrite it; they require a new study root/revision.

Run the frozen definition, not an independently edited copy:

```text
anthrosim-research \
  --definition study/example/research-definition.json \
  --run-dir study/example/research
```

Existing exploratory/engineering uses of `anthrosim-research` remain valid and do not require this wrapper. An unbound research execution simply cannot claim that #230's frozen confirmatory-protocol gate was satisfied.

## Result binding

After the research execution has reached only terminal run states (`completed` or `failed`), finalize the study:

```text
anthrosim-study finalize --study-dir study/example
```

Finalization verifies:

1. `study-plan.json` and `study-manifest.json` are identical;
2. the standalone frozen protocol and research-definition files exactly match the embedded plan;
3. both content identities recompute exactly;
4. the child `research-manifest.json` and `research-plan.json` agree;
5. the executed `definitionIdentity`, exact definition and source revision equal the pre-execution study plan;
6. the child `researchId` recomputes using the **same field-order identity algorithm as `anthrosim-research`**;
7. `research-state.json` belongs to that exact research execution and contains no still-planned/running runs;
8. the standard `analysis/points.json` and `analysis/runs.json` identify that same research execution.

It then writes immutable `study-result-binding.json`, containing:

- stable `resultIdentity`;
- `studyExecutionId`;
- protocol identity, revision, study ID and scientific status;
- pre-result-binding eligibility;
- definition identity;
- executed research ID;
- exact source revision;
- completed/failed run counts;
- each bound result-artifact path **and a digest of its exact bytes**.

The result-artifact digests mean a later change to an analysis artifact can no longer silently retain the old result binding. Running `finalize` again is idempotent when nothing changed. If the frozen protocol, research identity or bound result bytes differ, finalization fails rather than rewriting provenance.

For studies that claim held-out/independent corroboration, #206/#423 then derives `analysis/evidence-role-assessment.json` from this exact finalized binding. Derivation re-resolves every confirmatory evidence ID through the EvidenceCatalog inside the frozen definition, preserves each resolved source identity and source object, and refuses same-source/same-target aliasing. Later `verify` re-derives those bindings; mutation of either the assessment or the frozen source provenance invalidates verification.

## Amendments

Protocol revision 1 must not declare an amendment. Any later `protocolRevision` must declare `previousProtocolIdentity`, amendment timing (`before_result_inspection` or `after_result_inspection`), and a rationale. Because the complete amendment record is part of protocol content, every amendment necessarily receives a new protocol identity.

A confirmatory protocol explicitly amended **after result inspection** remains reproducible, but `confirmatoryPreResultClaimEligible` becomes `false`. It cannot silently retain a preregistered/predeclared label merely because the revised protocol is now frozen.

This mechanism records the scientific timing declaration; it is not a trusted external timestamp service. For stronger public preregistration evidence, archive or commit the prepared protocol/plan before result generation.

## Relationship to remaining research-readiness issues

This contract deliberately provides stable attachment points rather than absorbing all downstream methods work:

- **#206 / evidence-role firewall v1**, hardened by **#423 / AV3-013**: validates evidence-role independence, binds protocol IDs to frozen EvidenceCatalog source identities, and detects calibration/corroboration leakage through aliases;
- **#217**: execute and diagnose identifiability/equifinality analyses;
- **#226**: derive exposure-aware rates and cumulative-outcome semantics;
- **#231**: quantify Monte Carlo precision and replicate sufficiency for stochastic claims;
- **#229**: formalize survivor-conditioned and other analysis/statistical semantics where needed.

The frozen protocol can declare those plans now. Their dedicated issues implement the corresponding analysis logic.

## Model-semantics boundary

Study protocols and evidence-role source binding do not alter `ExperimentConfig`, initialization, M2/M3/M4/M8/M9 transition rules, random-number streams or draw order, checkpoints or deterministic run identity, or `MODEL_SEMANTICS_ID`.

A study-protocol or evidence-governance change is a change to the scientific claim/analysis contract, not to what the simulator causally does.

## Acceptance meaning

After this contract is used, a confirmatory result can identify one immutable pre-execution protocol that says what was being tested, which outputs/windows count, how runs are treated and what result would support/reject the declared hypotheses. For evidence-bearing studies, the corresponding frozen research definition also identifies the authoritative provenance source behind every confirmatory evidence reference. Changing those rules or source bindings produces a new provenance-visible identity rather than silently changing the interpretation of the old result.
