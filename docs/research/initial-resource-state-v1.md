# Initial resource state contract v1

AnthroSim treats day-zero M3 food stock as a scientific initial-condition assumption, not as evidence inferred from storage capacity.

## Authoritative configuration

`ResourceConfig.initialStockUnitsPerProductivity` defines the requested day-zero stock for each cell before the ordinary `productivityScalePermille` multiplier. For a cell with base productivity `P`, the configured starting stock is:

`P × initialStockUnitsPerProductivity × productivityScalePermille / 1000`

The physical cell storage capacity is still enforced as an upper bound. Therefore reducing capacity below the requested stock can cap the initial state, but increasing capacity does not create additional historical stock.

The synthetic-validation preset declares `initialStockUnitsPerProductivity = 10`. This reproduces the historical synthetic baseline in which generated world stock and ten years of default storage capacity coincided. The value is retained as a regression/null-model assumption only; it is not archaeological or palaeoecological evidence.

`World.foodStock` remains part of the frozen synthetic world representation for compatibility, but M3 day-zero dynamic stock is no longer implicitly inherited from it. The authoritative research-facing assumption is the resource configuration above.

## Reproducibility and provenance

The initial-stock parameter is inside `ExperimentConfig.resources`. It is therefore serialized into run manifests, checkpoints, research definitions, exact research identities, and sensitivity coordinates through the same full-configuration machinery used for other M3 parameters.

Evidence can bind the assumption directly at `resources.initialStockUnitsPerProductivity`. For an empirical or evidence-informed resource claim, support for capacity, regeneration, or productivity does not substitute for explicit support for the historical starting-stock assumption.

A study interpreting early scarcity, persistence, migration pressure, or temporary aggregation should either vary plausible starting stocks or justify a particular starting state with evidence. Changing `cellStockCapacityYears` is not a substitute for declaring starting stock.

## Burn-in and analysis windows

AnthroSim does not silently run an unrecorded resource-only equilibration phase. If a study intends to discard initialization transients, the burn-in is an analysis decision and must be declared in the frozen `StudyProtocol.analysisWindows` contract.

A burn-in exclusion must therefore be visible as a nonzero `analysisStartDay`. If the cutoff is selected by a convergence diagnostic, the analysis window should use the `convergence_diagnostic` selection rule and the diagnostic and criterion must be predeclared in the study rationale or analysis plan rather than chosen after inspecting the target result.

A study may claim practical insensitivity to initial stock only after comparing plausible starting-stock alternatives under otherwise identical regeneration and demand rules and showing convergence for the predeclared outcome/window. A long elapsed time by itself is not evidence that convergence occurred.

If the scientific question concerns day-zero or early-run scarcity timing, burn-in cannot be used to erase the relevant transient; the initial resource state is part of the estimand's causal setup and must be reported directly.

## Semantics version

Adding an authoritative initial-resource-state degree of freedom changes causal initialization and checkpoint compatibility. `MODEL_SEMANTICS_ID` therefore advances from v15 to v16. The default synthetic numerical baseline is intentionally preserved where the historical default assumptions are unchanged.
