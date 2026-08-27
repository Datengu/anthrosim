# Spatial boundary dependence v1 — finite study-area research contract

## Status

This document defines the normative research-readiness contract for AnthroSim issue #211. It governs how finite evidence-grounded spatial domains may be interpreted when M4 permanent migration or M9 temporary mobility can respond to the edge of the supplied raster.

This is a **research-method contract**, not a new behavioural mechanism. The authoritative M4 and M9 execution rules remain unchanged: the current `World` is finite and closed. M4 cannot consider destination cells outside it, and M9 cannot route outside it and later re-enter. The purpose of this contract is to stop those implementation facts from silently becoming historical claims.

## Scientific problem

A GIS crop is normally chosen by the analyst. Its outer edge is not automatically evidence that people, resources, routes or destinations ceased to exist there.

Without an explicit boundary contract, changing a 5 km study window to a 10 km or 20 km window can change outcomes even when every cell in the archaeological target area is unchanged. The edge can:

- remove M4 destination alternatives and create directional asymmetry;
- alter permanent migration frequency, distance and settlement concentration;
- prevent an M9 least-cost route from leaving a tight crop and re-entering it;
- change focal-region reachability, journeys and visitor person-days;
- change visitor-driven resource pressure and later M4↔M9 feedbacks.

Stable results across random seeds do not demonstrate freedom from this problem. Seed stability and **extent convergence** are different questions.

## Authoritative executable semantics

The current executable boundary semantics are `closed_finite_grid_v1`.

Under these semantics:

1. cells outside the supplied `World` do not exist for M4 candidate discovery;
2. M9 four-neighbour routing has no graph edges beyond the supplied `World`;
3. an M9 route cannot leave the supplied raster, cross unmodelled space and re-enter it;
4. AnthroSim does not invent external people, resources or movement corridors beyond available input data.

These rules describe what the engine executes. They do **not** by themselves justify interpreting the raster edge as a real historical barrier.

## Boundary interpretation declaration

Every evidence-grounded study that interprets spatial outcomes near a finite edge must explicitly classify that edge using `SpatialBoundaryDeclaration`.

### `unresolved_extent`

Use when the study has not yet justified the boundary scientifically. The finite execution edge remains active, but results potentially dependent on it are not research-ready.

### `analyst_defined_crop`

Use when the simulation extent is a chosen GIS/study crop rather than a historical barrier. This mode requires extent-sensitivity analysis and a predeclared convergence criterion before affected conclusions are described as boundary-insensitive.

### `declared_closed_barrier`

Use only when the closed edge is itself an explicit scientific assumption supported by identified evidence records. A barrier declaration must provide a non-empty rationale and valid supporting evidence IDs; unresolved evidence cannot support the declaration.

Passing structural validation does not make the evidence archaeologically persuasive. A real study must still justify that the cited evidence is appropriate to the period, location and kind of physical or social barrier being claimed.

**File dimensions are never barrier evidence.**

## Simulation domain versus analysis domain

The **simulation domain** is the whole raster on which AnthroSim executes. The **analysis domain** is the fixed physical region whose results the researcher intends to interpret.

`SpatialAnalysisDomain` defines the latter in the landscape CRS/reference coordinate space. Its boundaries must align exactly to landscape cells. This permits the same physical target area to remain unchanged while progressively larger simulation buffers are added around it.

For convergence work:

- the analysis-domain physical extent must remain identical across compared runs;
- cell size, orientation and spatial reference must remain identical;
- the normalized landscape values inside the analysis domain must remain identical;
- the experiment definition, process seeds, spatial mechanisms, founder assumptions, focal-region definition and schedules relevant to the comparison must remain fixed unless the study explicitly declares another factorial comparison;
- only the outer simulation extent and the newly represented external data should change.

Changing raster resolution is a different scientific intervention. Issue #203 governs resolution/physical-scale dependence; an extent-convergence experiment must not quietly change both resolution and extent at once.

## Boundary observability

`assess_spatial_boundary(...)` produces a machine-readable `SpatialBoundaryAssessment` tied to the exact landscape identity/digest, grid geometry, boundary declaration and analysis domain.

For every analysis cell it reports:

- grid position;
- distance in cells to the finite simulation edge;
- actual M4 candidate count;
- the full interior candidate count for the configured M4 radius;
- how many candidates are missing because of the finite edge;
- whether the M4 candidate set is truncated.

The assessment also reports:

- the minimum buffer around the analysis domain;
- whether any analysis cell has direct M4 candidate clipping;
- whether the configured M4 search horizon is clear of the boundary;
- that current M9 routes are confined to the simulation domain and cannot leave/re-enter it;
- whether the declared interpretation requires extent sensitivity.

A clear M4 candidate horizon is a **local geometric diagnostic only**. It does not prove that the full simulation is boundary-independent: migration, resources, kin distributions and temporary visitors can propagate effects inward from farther away. M9 likewise has no universal safe buffer because an alternative route may depend on a distant corridor.

## Normative extent-convergence protocol

For an analyst-defined crop or unresolved extent, the researcher must demonstrate convergence for the specific conclusion being made.

### 1. Predeclare the criterion

Before inspecting the enlarged-domain result sequence, define a `SpatialExtentAdequacyCriterion` containing:

- a stable criterion ID;
- the minimum buffer, if one is scientifically required;
- the number of consecutive stable enlargements required;
- every metric material to the intended claim;
- an absolute and/or relative tolerance for every metric.

AnthroSim deliberately provides no universal tolerance. A tolerance appropriate for visitor person-days may be inappropriate for population concentration or migration distance.

### 2. Preserve the inner problem

Construct a sequence of progressively larger simulation domains around the same physical analysis domain. The inner landscape and all non-extent experimental assumptions must remain unchanged.

Where the added outer area requires new empirical data, provenance for those added data must be preserved in the ordinary landscape/evidence machinery. Extent enlargement is not permission to fabricate an exterior environment.

### 3. Measure claim-relevant inner outcomes

At minimum, when the corresponding mechanism is active and relevant to the archaeological claim, evaluate:

- **M4 destination availability:** direct candidate clipping/missing candidates;
- **permanent migration:** completed moves and relevant direction/distance observables;
- **M9 accessibility:** reachable/unreachable origins, travel duration/cost where interpreted, and journeys started/not started;
- **focal use:** visitor person-days and other focal-region participation observables;
- **resource consequences:** resource stock/pressure in the analysis or focal domain, including visitor demand where M9 is active;
- **spatial structure/concentration:** if the claim concerns occupation, concentration or absence near the study edge.

A study may require additional metrics. Passing convergence for a narrow metric set does not license claims about unmeasured outputs.

### 4. Evaluate successive enlargements

`assess_spatial_extent_convergence(...)` compares adjacent observations in increasing buffer order.

For each declared metric it records:

- previous and current values;
- absolute difference;
- symmetric relative difference in permille using `max(previous, current)` as denominator;
- absolute/relative tolerance results;
- the combined pass/fail result.

A non-zero relative difference is rounded upward to whole permille so integer truncation cannot silently convert a small change to zero.

If both absolute and relative tolerances are declared, **both must pass**.

An enlargement contributes to the stable sequence only after the predeclared minimum-buffer condition is met. `adequate = true` only when the required number of trailing eligible enlargements all satisfy every declared tolerance.

### 5. Interpret failure explicitly

If the latest enlargement exceeds a declared tolerance, `material_boundary_dependence_at_latest_extension` is true and the criterion is not adequate.

The correct research conclusion is then that the measured result remains materially dependent on the chosen extent under the declared criterion. The researcher must enlarge the domain further, weaken/narrow the archaeological claim with justification, or explicitly report boundary dependence. The result must not be described as evidence of spatial absence, constrained catchment, route inaccessibility or edge concentration without that qualification.

## Controlled verification fixtures

The #211 acceptance suite deliberately separates several causal pathways.

### M4 candidate geometry

The same physical inner cell is embedded in a tight, buffered and larger landscape. The tight crop loses valid Manhattan-radius destinations; the two sufficiently buffered domains recover the same physical candidate offsets.

### M4 migration behaviour

A controlled resource gradient places the only better destination just beyond the tight crop. The tight run cannot move; a buffered run moves east to the represented physical cell; further enlargement preserves that first migration decision.

### M9 route reachability

A hard internal wall makes the direct route impossible. In the tight crop, no route exists because the model cannot pass around the wall outside the raster. Adding an outer corridor makes the same physical origin/destination pair reachable. A further enlargement preserves physical destination, travel cost and duration.

The comparison is performed in physical coordinate space rather than using raster-local `CellId`, because row-major IDs legitimately change when rows/columns are added around a fixed physical location.

### Focal use and resources

The same M9 fixture demonstrates the downstream pathway: the tight crop produces an unreachable journey, zero visitor person-days and no visitor resource draw at the focal cell; buffered runs become reachable, produce the visit and consume focal-cell resource stock; a larger domain reproduces the buffered result.

These are synthetic causal fixtures. They verify the boundary mechanism and convergence method; they do not supply an empirical buffer distance for a future archaeological study.

## Provenance requirements

For any result used in research interpretation, preserve alongside the ordinary run artifacts:

1. the serialized `SpatialBoundaryDeclaration`;
2. the serialized `SpatialBoundaryAssessment` for each simulation extent;
3. the fixed `SpatialAnalysisDomain`;
4. the predeclared `SpatialExtentAdequacyCriterion`;
5. the ordered `SpatialExtentObservation` inputs;
6. the resulting `SpatialExtentConvergenceAssessment`;
7. the ordinary experiment, landscape, evidence, mechanism and source-revision provenance already required by AnthroSim.

The assessment carries the exact landscape identity/digest so the declared boundary meaning is not inherited from an anonymous file dimension. For a convergence series, the archive/reproduction record must additionally demonstrate that the analysis-domain content and non-extent assumptions were held fixed across enlargements.

Until these boundary artifacts are first-class fields of the run bundle itself, they are required **companion research artifacts** rather than automatically embedded execution provenance. Their absence means the boundary research gate has not been demonstrated, even if the simulation run itself is reproducible.

## Interpretation boundary

Passing an extent criterion means only that the **declared measured outputs**, under the declared tolerance and enlargement sequence, are sufficiently stable for that study's stated purpose. It does not prove that:

- the exterior landscape is historically complete;
- every possible route has been represented;
- the chosen buffer is universally adequate;
- another metric would converge at the same extent;
- a closed social/physical barrier existed historically;
- resolution dependence (#203) has been resolved.

Conversely, detecting boundary dependence is a scientifically useful result. It tells the researcher that the current crop is still causally participating in the conclusion rather than allowing that hidden assumption to pass as archaeological structure.
