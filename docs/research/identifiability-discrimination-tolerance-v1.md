# Held-out discrimination tolerance contract v1

This note defines the machine-enforced scientific domain of `corroborationDiscriminationTolerance` used by `scripts/research-identifiability.py` when reporting held-out predictions between still-compatible structural hypotheses.

The tolerance is a finite, **non-negative** separation threshold. `0` is valid and means that the conservative structural simulation-interval envelopes must have strictly positive separation before the prediction can be labelled discriminating. Positive values require separation strictly greater than the declared tolerance. Negative values are scientifically invalid and the authoritative analyzer fails closed before emitting a discrimination result.

This domain restriction matters because structural interval envelopes that overlap have minimum separation `0`. Allowing a negative tolerance would make `0 > tolerance` true and could therefore mislabel overlapping structural predictions as discriminating. The non-negative contract prevents that failure while preserving the existing conservative-envelope rule: each structure spans the minimum bound `intervalLower` through maximum bound `intervalUpper` over all compatible points, and adequate bound Monte Carlo precision remains required for stochastic outputs.

The tolerance does not alter the calibration/held-out evidence-role firewall. Held-out corroboration is still diagnostic only; it cannot make the calibration/identifiability gate pass. Changing a held-out observation into calibration evidence requires a new declared analysis and provenance record.

Permanent regression coverage is provided by `scripts/test-research-identifiability-discrimination-tolerance.py`, which checks the negative rejection boundary together with zero- and positive-tolerance controls.
