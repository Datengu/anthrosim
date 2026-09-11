# AV6-002 male-parent age time-reference repair

Status: living post-Audit-v6 remediation contract for issue #694.

Scientific Audit v6 showed that one public male-parent age window had two temporal meanings: dynamic annual M2 tested male age at the start of `[t-365,t)`, while declared founder genealogy tested the same male-parent relationship at the child's birth day. The repair makes the parameter mean one thing everywhere.

## Authoritative rule

For a birth recorded at annual M2 boundary `t`, `maleParentMinAgeYears` and `maleParentMaxAgeYearsExclusive` constrain the male parent's completed age **at the child's birth boundary `t`**.

This does not move every M2 age lookup to the boundary. Female fertility schedule selection and elapsed background-mortality schedule selection remain based on age at `t-365`, because those parameters describe the annual exposure interval. Parentage locality also remains the persistent-residence exposure snapshot immediately before a same-day M4 move. The male-parent age window instead constrains the authoritative parent-at-child chronology created at `t`.

Declared founder genealogy already uses the child's declared birth day for the same configured window, so generated and declared authoritative genealogy now share one reproductive-age contract.

## Boundary behaviour

With `[18,70)` male-parent support:

- age `17y200d` at interval start and `18y200d` at child birth is eligible;
- age `69y200d` at interval start and `70y200d` at child birth is ineligible.

Permanent regression coverage preserves both directions and compares dynamic M2 directly with the equivalent all-pre-epoch declared-founder relationship.

## Compatibility

This repair can change whether births occur, and which male may be selected, when a candidate crosses a configured male-parent age threshold during the elapsed model year. That can change authoritative genealogy and downstream household/kin/movement state. It therefore advances the living `MODEL_SEMANTICS_ID` from `anthrosim-model-semantics-v38` to `anthrosim-model-semantics-v39`.

A v38 checkpoint must not silently continue under v39 because future parentage at an age-threshold crossing can differ even with identical persisted state and RNG positions. No wire-schema change is required; the incompatibility is scientific semantics rather than serialization shape.
