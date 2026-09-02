# Invariants

These rules are part of the parenting system contract.

## Hierarchy Ownership

Hierarchy-related state is owned by `ParentingSystem`.

```text
Parent components may only be added, changed, or removed by ParentingSystem.
children_by_parent may only be modified by ParentingSystem.
```

Other systems request hierarchy changes through parenting events.

## Parenting Event Ownership

Parenting event entities are consumed only by `ParentingSystem`.

```text
SetParent
ClearParent
ClearChildren
```

Other systems may create these events, but may not destroy them after
submission.

## Event Entity Invariant

An event entity is a transient container for one event.

```text
an event entity contains exactly one event component
an event entity contains no non-event components
an event entity contains no second event component
```

Its entity ID has no domain meaning.

Therefore:

```text
event entities must never be referenced by persistent world state
event entities must never be parenting targets
event entities must never be parents
```

Nothing should point to an event entity.

## Destruction Invariant

A parentable entity must pass through `PendingDestroy` before it is despawned.

Once marked, it must remain alive until `ParentingSystem` has processed that
`PendingDestroy` state.

This gives `ParentingSystem` a chance to resolve every relationship involving
it.

Direct or early despawn that bypasses this ordering is an invariant violation.

## Hierarchy Correctness

Under correct usage:

```text
Parent and children_by_parent are synchronized.
```

A mismatch is an invariant violation, not something normal parenting logic
should silently repair.

### TODO

Add an optional debug-only full hierarchy validation pass later.

## Failure Policy

Unexpected hierarchy corruption or disappearance of required entities during
apply is a programmer error.

The system should fail loudly rather than silently commit a different result.

## Deferred Structural Validation

The current parenting system does not enforce:

```text
cycle detection
self-parenting rejection
hierarchy depth limits
allowed parent/child entity kinds
```

These are intentionally outside the current scope and may be layered on later.
