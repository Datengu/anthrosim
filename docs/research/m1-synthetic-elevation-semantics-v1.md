# M1 synthetic elevation semantics v1

**Status:** normative contract for the synthetic M1 baseline

## Scope

M1's `elevation` field is a signed synthetic coordinate in `-500..=500`. It is not an empirical altitude, height above sea level, or direct lowland/highland measurement. The zero point is an arbitrary synthetic midpoint created by centring a deterministic `0..=1000` coherent field.

The historical implementation has always used the **distance from that synthetic midpoint** for its causal elevation effects. This contract makes that meaning explicit rather than describing the same equation as ordinary lowland favourability or directional relative elevation.

## Executable relation

For signed synthetic elevation `E`:

```text
extremeness(E) = abs(E)
midpoint_favourability(E) = 1000 - 2 * abs(E)
```

Within the valid `-500..=500` range, midpoint favourability is therefore `1000` at `E = 0`, `500` at `E = ±250`, and `0` at `E = ±500`.

Holding the independent wetness, fertility and ruggedness fields fixed, M1 derives:

```text
water_access
  = floor((3 * wetness + midpoint_favourability) / 4)

base_productivity
  = floor((5 * water_access
         + 3 * fertility
         + 2 * midpoint_favourability) / 10)

movement_cost
  = BASE_MOVEMENT_COST + 2 * ruggedness + extremeness
```

Thus sign reflection is intentionally symmetric:

```text
f(-E) = f(E)
```

for the elevation contribution to water accessibility, productivity and traversal cost when all non-elevation inputs are held constant.

## Scientific interpretation

This is a **synthetic midpoint-optimum null model**, not a claim that real landscapes have an ecological or mobility optimum at some corresponding physical elevation. In particular:

- negative synthetic elevation must not be interpreted automatically as lower physical terrain;
- positive synthetic elevation must not be interpreted automatically as higher physical terrain;
- `E = 0` is the centre of the generated synthetic field, not sea level;
- the symmetric penalty/favourability relationship is a mechanism-testing source of heterogeneous spatial structure only.

A research study that needs directional lowland/highland semantics must not infer them from this M1 baseline. It should instead supply an evidence-grounded M8 transformation or introduce a separately versioned directional synthetic mechanism with an explicit scientific justification.

## Compatibility and provenance

This contract is a semantic clarification of the already-executing M1 equations. It deliberately does **not** change generated cell values, world digests, resource stocks, movement costs, M8 fallback fields, or downstream authoritative simulation state. Consequently it does not advance `MODEL_SEMANTICS_ID`: model semantics v19 already produces these exact numerical relations.

Controlled unit tests pin the five audit points `-500, -250, 0, +250, +500` with wetness, fertility and ruggedness held fixed and separately enforce exact sign-reflection symmetry.
