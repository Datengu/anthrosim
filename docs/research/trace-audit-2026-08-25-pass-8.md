# TRACE adversarial scientific audit — pass 8

**Date:** 2026-08-25  
**Scope:** static repository/source/document audit of `main`  
**Lens:** cross-mechanism writer→reader causal-graph audit of shared state  
**Overall result:** **CLEAN WITH RESPECT TO NEW P1 DISCOVERY**

## Purpose

This pass asked a different question from the earlier equation, scale and inference audits:

> For every scientifically consequential shared state variable, which mechanism writes it, which later mechanisms read it, and can an unintended causal pathway emerge from that reuse?

The lens was motivated by #200, where a state variable (`condition`) carried more causal meaning than one subsystem assumed. The pass therefore traced shared state rather than reviewing each module in isolation.

## Shared-state pathways reviewed

### Condition

Writers/readers include:

- M3 resource-supply condition response;
- M4 permanent-travel condition loss;
- M3 scarcity-mortality probability;
- M4 relocation pressure;
- newborn initialization.

The unintended scarcity-cause pathway is already captured by #200, while newborn initialization is #201 and per-move nominal/realized observability is #225. No additional writer→reader pathway was identified.

### Persistent residence / person location

Residence is read by:

- M2 parent-locality/eligibility;
- M3 home resource attribution;
- M4 origin/candidate/kin logic;
- M8 residence-based observability;
- M9 origin/travel scheduling and catchment.

Known cross-mechanism risks already cover the material findings:

- #193 — same-instant M4 relocation changes M2 parentage locality;
- #197 — target-arrival journey timing can become stale after residence changes;
- #218 — residence-attributed death location must not be read as physical presence while away.

No new residence pathway was found beyond those contracts/issues.

### Dynamic resource stock

M3 owns the dynamic per-cell resource stock. M4 reads it as a local resource-support/candidate signal, while M9 duration-weighted presence changes where demand is attributed.

The major integration mismatch is already #196. M8's incorrect initial-stock reporting is #224. No new hidden resource-stock consumer was identified.

### Temporary physical presence

M9 writes temporary presence. It is consumed by:

- M3 duration-aware resource attribution;
- M4 eligibility (away households defer permanent migration);
- M9 observability.

M2 deliberately remains persistent-residence based rather than visitor-presence based, as specified by the M9 scientific contract. The M4 deferral behavior is likewise explicitly documented. No accidental new M9→M2/M4 causal pathway was found.

### Genealogy / household relations

M2 writes parentage and household membership; M4 reads a bounded direct-parent-location proxy.

The known semantic risks are already captured by:

- #188 — reproductive-sex/record-order asymmetry in the kin proxy;
- #207 — fixed household lifecycle / structural sensitivity.

No additional genealogy reader with undeclared causal effect was found.

### World/environment fields

M1/M8 world fields feed M3/M4/M9. The important residual synthetic-environment pathway in evidence-grounded spatial ensembles is already #212: M8 does not replace every causal environmental field, so seed-generated season amplitude/phase can remain active.

Physical scale and boundary effects remain #203/#211.

## Important non-findings

- M9 temporary presence does not silently rewrite persistent residence.
- M4 permanent movement remains separate from M9 temporary travel events.
- M2 visitor co-presence does not silently alter reproductive eligibility.
- M3 dynamic resource stock is not written by M4 utility evaluation.
- Derived M8/M9 observability does not feed back into authoritative simulation execution.
- The major shared-state causal risks identified by this pass all map to already-open issues rather than revealing a new hidden pathway.

## TRACE interpretation

This pass contributes mainly to:

- **4 — conceptual model evaluation:** shared state has explicit scientific meaning across subsystem boundaries;
- **5 — implementation verification:** writers/readers match the declared causal graph rather than gaining accidental cross-module semantics.

## Pass result

**No new P1 scientific-behaviour issue was identified.**

The pass materially challenged the cross-mechanism causal graph and mapped every consequential finding to an existing issue (#188, #193, #196, #197, #200, #201, #207, #212, #218, #224, #225 or associated scale issues).

It is therefore another **clean P1-discovery pass**.

The known P1 backlog still blocks final verification; this result only supports convergence of the current discovery phase.