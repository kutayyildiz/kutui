# Parenting System — Implementation Contract

This document describes the intended code shape without prescribing low-level
implementation details.

## Top-Level Flow

The implementation should visibly follow the semantic pipeline.

Conceptually:

```rust
pub fn run(&mut self, world: &mut World) {
    self.ensure_hierarchy(world);

    let pending_destroy = collect_pending_destroy(world);
    let mut events = collect_parenting_events(world, &pending_destroy);

    let mut destroy_clears = build_destroy_clears(
        world,
        &self.children_by_parent,
        &pending_destroy,
    );

    resolve_destroy_conflicts(
        &events.set_parent,
        &mut destroy_clears,
    );

    lower_clear_children(
        &self.children_by_parent,
        &mut events,
    );

    events.clear_parent.extend(destroy_clears);

    resolve_parenting_priority(
        &mut events,
        PARENTING_PRIORITY,
    );

    remove_noops(world, &mut events);

    apply_parenting_events(
        world,
        &mut self.children_by_parent,
        &events,
    );

    cleanup_hierarchy(&mut self.children_by_parent);
    consume_parenting_events(world, events.entities);
}
```

Exact helper boundaries may differ. Conflict policy should not be hidden inside
apply logic.

## Collection

Four logical queries are involved:

```text
SetParent
ClearParent
ClearChildren
PendingDestroy
```

Every parenting event entity ID is recorded so it can be consumed after
processing.

`PendingDestroy` is queried separately and is not consumed by `ParentingSystem`.
Entities marked `PendingDestroy` must remain alive until this run has processed
them.

### SetParent Collection

Invalid sets are ignored while collecting normalized set intent:

```text
missing child
missing parent
PendingDestroy child
PendingDestroy parent
```

For each child, the first encountered valid set survives:

```rust
set_parent.entry(child).or_insert(parent);
```

Later sets for that child are discarded.

## Destroy Clear Construction

For each `PendingDestroy(D)`:

```text
if D has Parent:
    destroy_clears += D

for child in children_by_parent[D]:
    destroy_clears += child
```

These clears represent relationships that destruction requires to disappear.

They are not normal parenting events.

## Destroy Conflict Resolution

All surviving sets already have non-destroyed endpoints.

Therefore, if a surviving set targets a child also present in `destroy_clears`,
the set wins and the destroy clear is removed.

This represents reparenting a surviving child away from a destroyed parent.

## ClearChildren Lowering

For every `ClearChildren(P)`, add normal clear intent for:

```text
current children_by_parent[P]
+
children with surviving SetParent(child, P)
```

After lowering, `clear_children` is considered consumed and must not be read by
later phases.

## Normal Priority

Merge surviving destroy clears into `clear_parent`.

Then resolve `set_parent` against `clear_parent` using `PARENTING_PRIORITY`.

At the end:

```text
set_parent.keys() ∩ clear_parent = ∅

After lowering, `clear_children` is considered consumed and must not be read by
later phases.
```

## No-op Elimination

No-op elimination happens only after both conflict classes are resolved.

It may remove:

```text
ClearParent for an entity that already has no Parent
SetParent(child, parent) when that relationship already exists
```

## Apply Contract

Apply contains no policy decisions.

At entry:

```text
all surviving SetParent endpoints exist
no surviving SetParent endpoint is PendingDestroy
set_parent and clear_parent are disjoint
clear_children is no longer used
```

Apply only performs the resolved `Parent` and hierarchy changes.

## Cleanup

After apply, remove hierarchy entries with empty child sets.

This restores the hierarchy representation invariant for the next run.

## Event Consumption

Every parenting event observed by the run is consumed, including events that
were:

```text
duplicated
discarded
invalid
out-prioritized
reduced to no-ops
successfully applied
```

Consumption means despawning the event entity.

## Error Handling

Broken hierarchy invariants are programmer errors.

Unexpected missing entities or impossible cache state during apply should fail
rather than be silently repaired into a different result.

## Out of Scope

The following are deliberately deferred:

```text
cycle detection
self-parenting validation
hierarchy depth validation
parent/child kind validation
full debug hierarchy verification
```

These can be added later without changing the event-resolution pipeline.
