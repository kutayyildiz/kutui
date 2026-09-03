# Parenting System

## Purpose

`ParentingSystem` maintains generic parent-child relationships between entities.

`Parent` is the authoritative relationship state:

```text
child -> parent
```

The system also maintains a derived reverse index:

```text
parent -> { children }
```

The reverse index exists for efficient parent-to-children lookup.

## State and Ownership

`ParentingSystem` owns:

```text
Parent
Hierarchy
```

where:

```rust
pub type Hierarchy = HashMap<Entity, HashSet<Entity>>;
```

Only `ParentingSystem` may add, change, or remove `Parent`, and only
`ParentingSystem` may modify `Hierarchy`.

Other systems may read these values but must request relationship changes
through `ParentRequest`.

Relationship changes are exposed to downstream systems through the transient
`ParentChanged` component.

`ParentingSystem` also reads `PendingDestroy` as a lifecycle constraint, but
does not own or consume it.

## Invariants

After `ParentingSystem` runs:

* Every `Parent` edge has a matching edge in `Hierarchy`.
* Every child in `Hierarchy` has the matching `Parent`.
* Every referenced parent exists.
* `Hierarchy` contains no empty child groups.
* No final parent relationship contains an entity that is `PendingDestroy`.
* Every `ParentChanged` represents an actual relationship change.
* Only `ParentingSystem` modifies parenting state.

A mismatch between `Parent` and `Hierarchy` is an invariant violation. Normal
system operation does not attempt to repair arbitrary hierarchy corruption.

## Structural Changes

Whenever an entity's parent relationship changes, the resulting change is
exposed through `ParentChanged`.

`ParentChanged::previous` contains the previous parent when one existed.

Conceptually:

```text
unparented -> P
    previous = None

P -> Q
    previous = Some(P)

P -> unparented
    previous = Some(P)
```

A no-op relationship change such as:

```text
P -> P
```

does not produce `ParentChanged`.

`ParentChanged` is not consumed by `ParentingSystem`. It remains available to
downstream systems until transient cleanup at the end of the orchestrator run.

### Destruction

`PendingDestroy` is a hard constraint.

An entity marked `PendingDestroy` may not remain an endpoint of a final
parenting relationship.

For:

```text
P -> D -> C
PendingDestroy(D)
```

the relationships involving `D` must disappear.

A surviving child may still be reparented away from the destroyed parent in the
same run:

```text
D -> C
PendingDestroy(D)
ParentRequest::Set { target: C, parent: Q }

-> C -> Q
```

A parentable entity must remain alive until `ParentingSystem` has processed its
`PendingDestroy` state.

Actual entity destruction belongs to lifecycle handling, not to
`ParentingSystem`.

## Events

`ParentingSystem` consumes `ParentRequest`.

The available requests are:

```text
ParentRequest::Set
ParentRequest::Clear
ParentRequest::ClearChildren
```

Every observed `ParentRequest` event entity is consumed after resolution,
including requests that are invalid, duplicated, out-prioritized, no-ops, or
successfully applied.

### Set

```text
ParentRequest::Set { target, parent }
```

A set request is discarded when:

* `target` does not exist.
* `parent` does not exist.
* `target` is `PendingDestroy`.
* `parent` is `PendingDestroy`.

When several valid `Set` requests target the same entity, the first one
encountered survives.

HECS query order is intentionally accepted as arbitrary.

### Clear

```text
ParentRequest::Clear { target }
```

Removes the target's current parent relationship when one exists.

A clear targeting an already parentless entity becomes a no-op.

### ClearChildren

```text
ParentRequest::ClearChildren { target }
```

Clears both:

```text
current children
+
surviving prospective children from ParentRequest::Set { parent: target, .. }
```

`ClearChildren` is lowered to per-child clear intent before normal conflict
resolution.

### Conflict Resolution

Destruction constraints are resolved before normal parenting priority:

```text
destruction constraints
        ↓
normal Set/Clear priority
```

A valid reparenting operation may move a surviving child away from a destroyed
parent.

After destruction conflicts are resolved, normal `Set` versus `Clear` conflicts
use `PARENTING_PRIORITY`.

The current policy is:

```text
Clear
```

so:

```text
ParentRequest::Set { target: A, parent: P }
ParentRequest::Clear { target: A }

-> A becomes parentless
```

No-op elimination happens only after conflict resolution so that no-op requests
still retain their conflict intent while priority is being decided.

## System Order

`ParentingSystem` is the first structural system.

The canonical sequence begins:

```text
ParentingSystem
    ↓
parenting validation
    ↓
OrderingSystem
```

Downstream systems may trust the validated parenting postconditions rather than
revalidating hierarchy state themselves.

## Validation

In debug builds, parenting validation runs immediately after
`ParentingSystem`.

It verifies:

```text
Parent -> Hierarchy
Hierarchy -> Parent
referenced parents exist
Hierarchy contains no empty groups
ParentChanged represents an actual relationship change
```

Validation is read-only.

It detects invariant violations but never repairs state.

## Out of Scope

`ParentingSystem` does not currently enforce:

```text
cycle detection
self-parenting rejection
hierarchy depth limits
domain-specific parent/child kinds
actual entity destruction
```

These concerns may be layered on separately without changing the basic
parenting model.
