# Ordering System

## Purpose

`OrderingSystem` maintains relative order between sibling entities.

Every entity with a `Parent` participates in the ordered group formed by that
parent's direct children.

Sibling order is represented by `Order`.

## State and Ownership

`OrderingSystem` owns:

```text
Order
OrderInvalidated
```

Only `OrderingSystem` may add, modify, or remove `Order` or produce
`OrderInvalidated`.

The system reads:

```text
Parent
Hierarchy
ParentChanged
```

and consumes:

```text
OrderRequest
```

## Invariants

After `OrderingSystem` runs:

* Every entity with a `Parent` has an `Order`.
* Every entity without a `Parent` has no `Order`.
* `Order` is relative to siblings under the same parent.
* Every sibling group is dense: `0..n-1`.
* No sibling group contains duplicate `Order` values.
* Only `OrderingSystem` modifies ordering state.

`OrderingSystem` assumes the validated postconditions of `ParentingSystem`.

It does not scan the world for unrelated parenting or hierarchy violations.

## Structural Changes

`ParentingSystem` exposes relationship changes through `ParentChanged`.

`OrderingSystem` uses those changes to reconcile only affected sibling groups.

### Leaving a Parent

When a child leaves a sibling group:

* its previous sibling group becomes dense again;
* existing siblings retain their relative order;
* if the entity becomes parentless, its `Order` is removed.

If the entity had a previous parent, its previous sibling-relative position is
preserved through:

```text
OrderInvalidated::previous
```

`OrderInvalidated` means that the entity's previous `Order` stopped describing
its current hierarchy position because its parent relationship changed.

It is not a generic notification that `Order` changed.

Ordinary `OrderRequest` operations do not produce `OrderInvalidated`.

### Entering a Parent

A child entering a sibling group is appended to the end.

Existing siblings retain their relative order.

When several children enter the same parent during one run, their relative order
is unspecified.

A missing `Order` on a newly parented entity represented by `ParentChanged` is a
valid temporary state between `ParentingSystem` and `OrderingSystem`.

A missing `Order` on an unchanged parented entity is an invariant violation, not
state that `OrderingSystem` should silently repair.

`ParentChanged` and `OrderInvalidated` remain available to downstream systems
until transient cleanup at the end of the orchestrator run.

## Events

Ordering changes are requested through `OrderRequest`.

An order request must target an entity that currently participates in an ordered
sibling group.

A target with missing required ordering state is an invariant violation rather
than an ignored request.

Every observed `OrderRequest` event entity is consumed after processing.

### Set

```text
Set(target, order)
```

Moves the target to:

```text
min(order, last_index)
```

Other siblings shift as necessary.

### Increment

```text
Increment(target)
```

Moves the target one position forward.

If the target is already last, it wraps to the first position.

### Decrement

```text
Decrement(target)
```

Moves the target one position backward.

If the target is already first, it wraps to the last position.

### Event Ordering

Multiple `OrderRequest` events in the same run are processed in arbitrary HECS
query order.

Multiple requests may target the same entity. Each request operates on the
ordering produced by the requests processed before it, so ordering invariants
remain preserved.

No request type has priority over another.

For example, `Set` is not inherently preferred over `Increment` or `Decrement`.

Deterministic event sequencing may be introduced later if required.

## System Order

`OrderingSystem` runs after parenting and before focusing:

```text
ParentingSystem
    ↓
parenting validation
    ↓
OrderingSystem
    ↓
ordering validation
    ↓
FocusingSystem
```

It assumes validated parenting state and establishes the ordering postconditions
required by downstream systems.

## Validation

In debug builds, ordering validation runs immediately after `OrderingSystem`.

It verifies:

```text
Parent -> Order
Order -> Parent
dense sibling orders
```

Dense sibling order also proves that sibling order values are unique.

Ordering validation does not revalidate `Parent`/`Hierarchy` consistency because
that belongs to parenting validation.

Validation is read-only and never repairs state.

## Out of Scope

`OrderingSystem` does not define:

```text
domain-specific ordering semantics
cross-parent movement
deterministic ordering between simultaneous requests
layout position
rendering or stacking behavior
```

Cross-parent movement is represented as a parenting change and then reconciled
by ordering.
