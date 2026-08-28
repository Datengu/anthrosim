# Authoritative history and intermediate metric invariant boundary v1

AnthroSim validates retained history at two complementary levels. The continuation digest binds the exact serialized checkpoint bytes needed for deterministic continuation and output integrity; structural invariants independently reject retained histories that are internally impossible even when an attacker or repair tool has recomputed that digest.

For authoritative demographic history, every post-founder person record must have exactly one birth event and every dead person record must have exactly one death event. Duplicate entity events are rejected, so an omitted person cannot be hidden by substituting a second otherwise-valid event. For M4, a household cannot have two completed migration events on the same decision day, and every retained decision trace must reconcile with the shared fields of its authoritative move event.

Intermediate metric snapshots are checked against the authoritative event prefix through that snapshot day. Birth, death, migration-move, moved-person and movement-distance totals must reconcile; population records and living population must satisfy the founder-plus-births-minus-deaths accounting identity; resource stock must satisfy initial plus regenerated equals harvested plus remaining stock; condition-mediated death counts and basic bounded/cumulative counters must remain plausible; and cumulative counters may not move backwards.

Historical population/resource/migration digests and `stateDigest64` are deliberately not reconstructed for intermediate snapshots because the full historical present state needed to reproduce those digests is not retained. Their exact bytes are protected by continuation integrity, while the independent checks here are accounting/plausibility invariants. This distinction avoids presenting a hash as independent scientific validation.

These checks are validation hardening only. They do not change simulation trajectories, RNG consumption, demographic/migration/resource equations, model semantics identity, or scientific reference values. Existing M8/M9 reference outputs must remain unchanged.
