# Ordering System

## Purpose

`OrderingSystem` maintains relative ordering between sibling entities.

Ordering is fully hierarchy-agnostic. Any entity with a `Parent` participates in
the ordered group formed by that parent's children.

## Invariants

After `OrderingSystem` runs:

* Every entity belonging to an ordered group has an `Order`.
* Only `OrderingSystem` modifies `Order`.
* `Order` is relative to siblings under the same parent.
* Every ordered group is dense: `0..n-1`.
* No ordered group contains duplicate `Order` values.
* Entities without a `Parent` do not participate in ordering.

Temporary violations such as a parented entity without an `Order` are valid
before `OrderingSystem` runs and are reconciled by the system.

## Structural Changes

`ParentingSystem` is authoritative for hierarchy membership.

When an entity's parent changes, it receives a transient `ParentChanged` marker.

`OrderingSystem` uses hierarchy state and these markers to reconcile affected
sibling groups.

* Children leaving a group cause the remaining order to become dense again.
* Children entering a group are appended to the end.
* Parented entities without an `Order` are appended to the end.
* `ParentChanged` is removed later by orchestrator-level transient cleanup.

## Order Requests

Ordering changes are requested through `OrderRequest`.

### Set

`Set(target, n)`

Moves the target to index `min(n, last_index)`.

Other siblings shift as necessary.

### Increment

`Increment(target)`

* Normally swaps with the next sibling.
* If already last, moves to first.

### Decrement

`Decrement(target)`

* Normally swaps with the previous sibling.
* If already first, moves to last.

## Event Ordering

Multiple `OrderRequest` events in the same run are processed in arbitrary HECS
query order.

This is intentional for now. Deterministic event ordering may be introduced
later if required.

## System Order

`ParentingSystem` runs before `OrderingSystem`.

This allows OrderingSystem to treat the hierarchy as authoritative and concern
itself only with ordering.
