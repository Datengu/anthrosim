# AV6-003 v40 remediation revalidation

**Status:** living-remediation evidence for Audit-v6 finding AV6-003 / issue #699. This note does not alter the immutable `v0.3.6` / `anthrosim-model-semantics-v35` discovery evidence.

## Repair identity

- Production repair PR: #772
- Protected-base SHA: `9d9c89495f66e50f30d434255069544cd8dfa866`
- Repair semantics: `anthrosim-model-semantics-v40`
- Package version: `0.3.6`
- Focused repair verification run: `34613578743`

The repair preserves living external direct-parent context in dependency-aware household fission by refining otherwise equivalent external-parent signatures with the parent's persistent residence cell. This is the same persistent cross-household kin-location context consumed downstream by M4. `PersonId`, `HouseholdId`, packed-record order and the global stochastic-coupling rank are not used as replacement social tie-breaks.

The original Audit-v6 Area-C external-kin relabelling construction from evidence PR #698 is promoted as a permanent regression. The existing #399 in-source relationship-invariance regression and the M4 cross-household kin-location record-order invariance regression also pass.

## Household-lifecycle structural sensitivity (#207)

The frozen #207 design was rerun rather than assumed compatible because it directly exercises `deterministic_dependency_fission_v2`.

- First v40 run: `34614039366`
- First-run head: `599fc83201066997e2662603a7a5516126b32972`
- Artifact: `10269147530`
- Artifact ZIP SHA-256: `1d1d400ba269e8356a55f5e98f52eb4280105367a62a38dfe1010cf74331bc38`
- Raw result SHA-256: `e242f2ed153ca65f4aca4b91d79fdc63c82a6255ef099e505a2a4dc170997d75`
- Independent reproduction run: `34614627601`
- Independent reproduction head: `9a1fda5f6facfc2e948c4f46ba0a33f13cb31c83`
- Execution: 16/16 declared arm-runs completed; zero extinctions.

The independent reproduction matched the first raw v40 JSON byte-for-byte. The previous checked machine reference was genuine v21 evidence and is retained unchanged as `research/household-lifecycle-sensitivity-v2/reference-result-v21.json`; the living `reference-result.json` is bound to the reviewed v40 evidence.

The long-range v21-to-v40 numerical difference is cumulative across many authoritative semantics changes and is not attributed solely to AV6-003. The qualitative #207 conclusion survives: household lifecycle remains a material structural-uncertainty dimension, while aggregate visitor person-days remain essentially invariant in this ensemble.

## General demographic confirmation (#304)

The exact-head v40 confirmation run `34614906438` completed all 780 predeclared runs successfully. All three frozen Monte Carlo precision diagnostics returned `sufficient_stop`; the scientific recommendation remains `no_universal_demographic_baseline`.

The run initially failed only because the checked canonical result was still bound to v39. The generated evidence was retained as:

- Workflow run: `34614906438`
- Artifact: `10269924097`
- Artifact ZIP SHA-256: `8402aa776fb3bb8fb453141692d5b147854340c55eee0f57d9c3e9aa45f0ee37`
- Generated `expected-result.json` SHA-256: `c7a131a88176299637483cf63ccd92eb4cb3fb51bfbe45f7e4633972cf8ad438`
- Generated model semantics: `anthrosim-model-semantics-v40`
- Run count: 780
- Recommendation: `no_universal_demographic_baseline`

One-shot evidence-binding run `34616008998` downloaded that exact artifact, verified the expected-result hash and scientific disposition, copied the generated result to `research/general-demography-baseline-v1/confirmatory-result.json`, passed the current-semantics guard plus `general_demography_reference_semantics`, and removed its transient helper workflow before committing `d297a2c4f73fc74513e54ec3f93907029d4224a7`.

## M7.6 canonical reference revalidation

The first complete v40 protected-matrix pass reached the canonical M7.6 experiment only after all preceding workspace, benchmark, release-build, performance/memory, M5/M6 integration and 1000-run soak jobs were green. M7.6 completed all **144/144** scientifically eligible runs, but the canonical-reference assertion stopped at the expected semantics-identity mismatch because the checked reference still identified v39.

The generated v40 M7.6 evidence was retained as:

- Artifact: `10271418177`
- Artifact ZIP SHA-256: `04615ae21dd2bd78c4c38c24f2fb78984012fe2da1e9de921a9a401ba30319c4`
- Execution: 144/144 declared runs completed.

A fail-closed one-shot verifier reconstructed and compared every canonical point summary before allowing any reference update. All **18/18 point summaries matched the v39 reference exactly**, so the v40 update is a provenance/semantics rebind only: no canonical numerical result changed. Verification workflow run `34618007706` passed and removed its transient helper after binding the reference; the resulting reference commit is `6b6a2795dceb58ba95a5958f386f4130b410a59c`.

This result is consistent with the scope of AV6-003: the repair changes household-fission choice only when otherwise-equivalent source candidates differ in represented living external-parent residence context. The canonical M7.6 design did not produce a numerical change under v40.

## Other downstream scientific gates

Applicable scientific/security run `34614906592` passed both current v40 scientific references without a numerical rebaseline:

- M8.6 terrain null-model benchmark: passed canonical reference and tamper rejection.
- M9.7 aggregation benchmark: passed deterministic replay, active checkpoint/resume, canonical reference and tamper rejection.

A final complete protected/scientific matrix is still required on the final user-authored PR head after this evidence note is synchronized. AV6-003 is P2, so after that exact-head matrix is green it may be merged and closed without a separate mandatory post-merge evidence-only derivative.
