# Audit v3 Area N — cross-system integration review

Audit target: `v0.3.3` / `d3b6fc0b0750933b57252c9087513d156d88f218` / `anthrosim-model-semantics-v21`.

This is a discovery-only integration pass. It does not repair any finding. It explicitly re-attacks the required coupled surfaces after the individual A–M audits, using frozen-source inspection plus fresh v3 adversarial evidence already generated against the same immutable target.

## Coupled integration matrix

| Coupling | Fresh integration attack/result | Disposition |
|---|---|---|
| demography × households | Frozen schedulers execute annual M2 before dependency-aware household fission in both hosts. AV3-001 therefore propagates immediately into household topology: the demonstrated host mismatch is **0 births vs 1 birth** on the same declared founder history before the same-day lifecycle step. A household near the fission threshold can consequently enter lifecycle processing with a different living-member count solely because of host choice. | Existing AV3-001 is the underlying defect; no duplicate issue. |
| demography × resources | M3 periods invoke condition/resource processing with background mortality as a competing-risk context before annual M2 fertility. The prior Area-A half-open collision adversary and Area-H competing-risk evidence did not demonstrate duplicate mortality or scheduler-order cause priority on this coupled boundary. | No additional defect demonstrated. |
| households × movement | `apply_household_lifecycle_at_annual_boundary` passes temporary-presence eligibility into fission and then calls `reconcile_household_topology_at_boundary`, preventing newly split households from retaining impossible stale temporary-presence topology. AV3-004 remains important because PersonId-sensitive fission can change the household graph that later M4/M9 decisions consume. | Existing AV3-004 cross-cutting; no new defect. |
| movement × resources | M4 travel acts through the shared condition state and M3 later consumes that state; Area-L review confirmed nominal and realized travel-condition burdens are separately observable. Area-D AV3-005 remains a cross-cutting condition-state defect because latent M3 deterioration can be erased by later full supply. | Existing AV3-005; no additional defect demonstrated. |
| aggregation × resources | Fresh Area-F adversary doubled simultaneous one-person seven-day visits: visitor-person-days **7→14**, peak presence **1→2**, and visitor resource need **7→14**. Destination resource pressure therefore scales with actual simultaneous physical presence in the tested limiting case rather than persistent-residence count. | Passed tested surface. |
| initialization × demography | AV3-003 shows an internally contradictory founder chronology can create **1 artificial first-boundary birth instead of 0**; AV3-001 shows spatial execution can ignore a valid declared `lastBirthDay` entirely. These are direct initialization→M2 causal failures, not merely metadata defects. | Existing AV3-001/003. |
| initialization × spatial placement | Fresh Area-G limiting-case arms initialized at condition **400** and **900** and remained exactly **400/900** for five years with erasure mechanisms disabled; year-2 resume reproduced uninterrupted output. This demonstrates that elapsed time/checkpointing alone does not erase a declared initial-state contrast. Spatial host founder-history integration remains limited by AV3-001. | Passed tested persistence surface; AV3-001 remains. |
| stochastic inference × censoring/extinction | Area-L long-run diagnostics preserve non-completed and early-terminated counts and refuse equilibrium-like support without complete runs/windows/sensitivity coverage. However, Area-H AV3-006 independently shows that the Monte Carlo precision gate can still report half-width **3.666756860283** when the same-seed covariance-aware half-width is **5.185577281736** at threshold **4.5**. | Censoring bookkeeping passed tested surface; AV3-006 remains inference-critical. |
| sensitivity × hidden configuration | `ResearchExperimentDefinition` carries the complete typed `ExperimentConfig` plus optional exact spatial config and records resulting configurations in point/run analysis. AV3-008 nevertheless shows overlapping ancestor/descendant dimensions can create **4 recorded coordinate combinations but only 2 distinct executable treatments**. | Existing AV3-008. |
| calibration × identifiability | Area-J adversaries jointly show that calibration/identification metadata can be misleading even when the simulator runs reproducibly: unvaried theta can be `identified=true`, nuisance width can remain **1.0** while top-level equifinality is false, conservative held-out envelopes can be averaged into false separation, and evidence aliases can defeat the calibration/held-out firewall. | Existing AV3-009/010/011/012/013. |
| checkpoint/resume × RNG | Fresh Area-G year-2 continuation produced the same complete `RecordedRun` as uninterrupted execution for both 400 and 900 condition arms after excluding operational resume-lineage metadata. Checkpoints also persist explicit RNG positions. | Passed tested surface. |
| observability × scientific interpretation | Area-L fresh hand cases distinguish run-weighted versus pooled-per-move estimands (**500 vs 100**) and flag survivor-condition improvement **600→800** alongside living-population decline **100→20** as discordant rather than an unconditional population treatment effect. AV3-015 and Area-M AV3-016/017 remain output/documentation drift defects. | Tested interpretation guards pass; existing AV3-015/016/017 remain. |

## Integration consequence synthesis

The final integration pass did not reveal a new smallest underlying defect beyond AV3-001 through AV3-017. It did, however, confirm that several existing findings have multi-system consequences rather than remaining local implementation defects:

- AV3-001 can alter same-day household lifecycle state and then all later household-based movement/resource outcomes.
- AV3-004 can alter later movement and aggregation because household topology itself is the mobility/resource unit.
- AV3-005 can alter later condition-mediated mortality and migration pressure because condition is shared across M3/M4/M2 mechanisms.
- AV3-006 can invalidate apparent precision for coupled treatment comparisons even when run-level outputs and censoring bookkeeping are correct.
- AV3-008 can invalidate a nominal sensitivity design before any downstream identifiability/calibration result is interpreted.
- AV3-013/014 can make otherwise well-recorded analysis/research artifacts claim independence or exact source identity that was not actually established.

## Area-N conclusion

All protocol-required coupled surfaces were explicitly revisited against the frozen v0.3.3 baseline. No additional independent defect was demonstrated on this final pass. Area N is complete with the existing audit-v3 backlog left deliberately unrepaired.

Because Areas A–N are now covered, Audit v3 discovery is complete. The convergence classification is necessarily non-clean because the frozen baseline has new P0/P1 findings. The next phase is backlog repair and independent re-verification; no repair is performed in this discovery note.
