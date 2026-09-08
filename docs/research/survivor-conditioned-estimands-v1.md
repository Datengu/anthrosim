# Survivor-conditioned condition estimands v1

## Scope

`meanLivingConditionPermille` remains a valid descriptive statistic: it is the arithmetic mean condition among people who are alive at the observation boundary. It is not, by itself, an unconditional population-level treatment effect when the compared mechanism can change mortality.

This contract is downstream research governance. It does not change simulation state, mortality, condition dynamics, RNG streams, scheduler ordering, checkpoints, or `MODEL_SEMANTICS_ID`.

## Why the distinction is required

Survival is post-treatment when an experimental mechanism can affect mortality. Consequently the people contributing to a terminal survivor mean can differ between arms.

A higher survivor mean can therefore accompany worse survival. For example:

- control: 10 survivors, mean condition 740 permille;
- treatment: 9 survivors, mean condition 800 permille because the low-condition person died.

The treatment has the higher survivor-conditioned descriptive mean and the lower surviving population. Reporting only `800 > 740` as a condition improvement would hide the selection-by-death mechanism.

The reverse can also occur: an intervention that keeps frail people alive may improve survival while lowering the mean condition among survivors.

## Frozen StudyProtocol declaration

The v1 StudyProtocol schema already provides a required free-text `interpretation` field for every observable and rejects unknown JSON fields. To avoid an unrelated protocol-schema migration, this contract uses machine-readable tokens inside that existing frozen field.

Any observable whose `source` references `meanLivingConditionPermille` and is used for between-arm research interpretation must declare all three tokens:

```text
estimand=survivor_condition_at_boundary
conditioning=survival
death_handling=no_post_death_imputation
```

The declaration must be frozen with the StudyProtocol before confirmatory result inspection under the normal protocol rules.

The validator is:

```text
python3 scripts/research-survivor-conditioning.py <study-protocol.json>
```

A missing token is a research-gate failure.

## Joint survival requirement

Every StudyProtocol comparison that includes a survivor-conditioned condition observable must also include at least one survival/population observable in the same comparison **and in the same `analysisWindowId` as that survivor-conditioned observable**. A terminal survivor-conditioned condition endpoint therefore cannot be safeguarded by an early-window population or mortality measurement: the joint observable must expose survival at the boundary that defines the survivor set being summarized.

The machine gate recognizes only exact, producer-defined source identities. Current recognized identities are:

```text
metrics.population.finalLivingPopulation
metrics.json.finalLivingPopulation
metrics.population.livingPopulation
metrics.population.deathsSinceStart
metrics.population.conditionMortalityDeaths
metrics.population.resourceScarcityDeaths
metrics.population.populationExtinct
```

These names are semantic identifiers, not search tokens. A free-form label such as `derived.not_a_real_mortality_observable`, `derived.fake_survival`, or `derived.finalLivingPopulation` is not evidence of a population outcome and must fail closed even though it contains a mortality/survival-related word. New source identities must be added deliberately when their produced semantics are established; substring matching is not permitted.

If a comparison contains multiple survivor-conditioned condition observables at different analysis boundaries, each one independently requires a recognized survival/population observable at its own boundary. Merely including one survival observable somewhere in the comparison is insufficient.

This does not combine condition and survival into an invented scalar. It requires them to be reported jointly at the same analysis boundary so survivor composition remains visible.

## Death handling

AnthroSim does not automatically assign a condition value after death. `meanLivingConditionPermille = null` after extinction remains undefined, consistent with the empty-set semantics introduced under issue #222.

A study that needs a baseline-cohort, composite, multistate, quality-adjusted, or other post-death estimand must define and justify that estimand explicitly. It must not obtain one by silently replacing death with condition zero or any other in-domain sentinel.

## Repeated-state and person-time analyses

Issue #215 now preserves compact condition distributions at M3 resource-period boundaries. Those histories can support future predeclared repeated-state or person-time summaries. Such summaries answer a different question from the terminal survivor mean and must be named accordingly.

The present v1 gate deliberately does not pretend that a repeated survivor distribution identifies an unconditional causal effect in the presence of death. It simply provides a less terminal-only view when scientifically appropriate.

## Other post-treatment conditioning

The same warning applies conceptually to metrics such as means among movers when the treatment changes who moves. This contract does not automatically reclassify every conditional metric, but research reports must state the conditioning event when it is itself an experimental outcome.

## Synthetic reversal regression

`scripts/test-research-survivor-conditioning.py` fixes the audit example as an executable regression:

- control: `finalLivingPopulation = 10`, `meanLivingConditionPermille = 740`;
- treatment: `finalLivingPopulation = 9`, `meanLivingConditionPermille = 800`.

The regression also proves that the recognized exact population/death/extinction identities pass while fabricated labels containing mortality/survival terms fail closed. It additionally fixes the Audit-v5 AV5-008 boundary condition: a recognized survival/population observable in an earlier analysis window cannot satisfy a terminal survivor-conditioned condition comparison, while the same recognized observable at the terminal boundary does satisfy it.

The derived assessment must report:

- survivor mean direction: higher;
- living-population direction: lower;
- discordant directions: true;
- survivor condition is population treatment effect: false.

The regression also verifies that extinction leaves survivor condition undefined and that no automatic post-death imputation is introduced.

## Interpretation rule

A research conclusion may say that an arm has a higher or lower **mean condition among survivors**. It may not turn that statement into a population-level condition improvement or deterioration without an explicitly justified estimand that handles differential mortality.
