# Model semantics v39 benchmark re-verification

Date: 2026-09-11
Audit finding: AV6-002 / #694
Production PR: #771

## Why the canonical references changed

AV6-002 gives `maleParentMinAgeYears` / `maleParentMaxAgeYearsExclusive` one authoritative temporal meaning: completed male age at the recorded child's birth boundary. That chronology correction can alter births and genealogy near configured male-parent age thresholds, so the living model semantics intentionally advance from `anthrosim-model-semantics-v38` to `anthrosim-model-semantics-v39`.

The existing M8.6 and M9.7 references were generated under earlier model semantics. Their protected verifiers therefore rejected the first v39 repair head even though both fresh benchmark executions completed and retained their predeclared scientific classifications. This refresh is a causal semantics rebaseline, not a weakening or bypass of either protected gate.

## Pre-merge evidence source

The reference refresh is bound to central CI workflow run `34563596919` for branch head `ec9829e5902c7b8a4c6a0bd62d56d2dac57dc603` (pull-request merge-ref build `8f70575b24473376f779e2d73f03e946f5250825`).

- M8.6 artifact `10185320001`, SHA-256 `3d1ebbde531945d61f698aefac6131669ed863b85679acf91c614fb6952533bb`, aggregate canonical SHA-256 `e5d3b6882f4fc35995ba89d7047c97de9347c5e32a35a6621465df6744730c6b`.
- M9.7 artifact `10185319300`, SHA-256 `086d639de983fbfdf3521715939af1d5e65124ed782a8bbea042c4f2be27f271`, aggregate canonical SHA-256 `95ab89935029a28516c21216b48eb403e943d882342cadba4b7f0a3e55441a09`.

M8.6 remains classified `fragile_spatial_structure`, with `terminalLargestCellSharePermille` retaining the benchmark's robust-metric classification and no degenerate arms. M9.7 remains `capability_distinguished`, with all eight predeclared paired seeds passing all paired criteria. The refreshed travel-burden reference is taken from the same M9.7 artifact.

## Gate policy

The benchmark definitions, criteria, verifiers, declared seed sets and protected workflow logic are unchanged. Only canonical expected outputs are rebound to the reviewed v39 execution. PR #771 must rerun the normal exact-head central and protected scientific/security workflows against these references and may merge only if those gates are green.

## Exact-head reference correction

Protected exact-merge run `34572831776` reran the same M8.6 and M9.7 designs after the references were checked in. M8.6 passed its canonical comparison. M9.7 reproduced the reviewed v39 verifier projection, but the gate exposed a single transcription error in the checked seed-9705 legacy projection: the artifact value is `visitorPersonDays=161820`, while the first checked projection said `161822`. The exact fraction and every other verifier-relevant field already corresponded to `161820`.

The M9 reference is therefore regenerated directly from reviewed artifact `10185319300`; the benchmark definition, criteria, verifier, travel-burden contract and scientific classification are unchanged. The living M8/M9 result pages are also synchronized to identify v39 as their checked current reference.
