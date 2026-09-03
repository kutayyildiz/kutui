# Parenting System

## Purpose

`ParentingSystem` maintains parent-child relationships between entities.

`Parent` is the authoritative relationship state:

```text
child -> parent
```

The system also maintains the derived reverse index:

```text
parent -> { children }
```

represented by:

```rust
pub type Hierarchy = HashMap<Entity, HashSet<Entity>>;
```

`Hierarchy` exists for efficient parent-to-children lookup.

## State and Ownership

`ParentingSystem` owns:

```text
Parent
Hierarchy
ParentChanged
```

Only `ParentingSystem` may add, modify, or remove `Parent`, modify `Hierarchy`,
or produce `ParentChanged`.

The system reads:

```text
PendingDestroy
```

and consumes:

```text
ParentRequest
```

`PendingDestroy` is treated as a lifecycle constraint but is neither owned nor
consumed by `ParentingSystem`.

## Invariants

After `ParentingSystem` runs:

* `Parent` and `Hierarchy` describe the same relationships.
* Every referenced parent exists.
* `Hierarchy` contains no empty child groups.
* The parent graph contains no cycles.
* No final parent relationship contains an entity with `PendingDestroy`.
* Every `ParentChanged` represents an actual relationship change.
* Only `ParentingSystem` modifies parenting state.

`ParentingSystem` assumes parenting state is modified only through this system.

It does not attempt to repair arbitrary external corruption.

## Structural Changes

Every actual parent relationship change produces `ParentChanged`.

`ParentChanged::previous` preserves the previous parent:

```text
unparented -> P
    previous = None

P -> Q
    previous = Some(P)

P -> unparented
    previous = Some(P)
```

A no-op relationship change does not produce `ParentChanged`.

`ParentChanged` remains available to downstream systems until transient cleanup
at the end of the orchestrator run.

### Destruction

`PendingDestroy` is a hard parenting constraint.

An entity marked `PendingDestroy` may not remain either a child or parent in the
final hierarchy.

For:

```text
P -> D -> C
PendingDestroy(D)
```

both relationships involving `D` are removed.

A surviving child may still be reparented away from a destroyed parent in the
same run:

```text
D -> C
PendingDestroy(D)
Set(C -> Q)

-> C -> Q
```

If that reparenting is rejected during later resolution, the child becomes
parentless rather than remaining attached to the destroyed parent.

Actual entity destruction belongs to lifecycle handling.

## Events

Parenting changes are requested through `ParentRequest`.

Every observed parenting request event entity is consumed after resolution.

### Set

```text
Set(target, parent)
```

A set request is ignored when:

* `target` does not exist;
* `parent` does not exist;
* `target` has `PendingDestroy`;
* `parent` has `PendingDestroy`.

If several valid `Set` requests target the same entity, the first encountered
survives.

HECS query order is intentionally accepted as arbitrary.

A surviving set is discarded if the resulting parent graph would be cyclic.

Self-parenting is therefore rejected as a cycle.

### Clear

```text
Clear(target)
```

Removes the target's current parent when one exists.

Clearing an already parentless entity is a no-op.

### ClearChildren

```text
ClearChildren(target)
```

Clears both the target's current children and surviving prospective children
from `Set` requests targeting that parent.

`ClearChildren` is lowered to ordinary per-child clear intent before normal
conflict resolution.

### Conflict Resolution

Destruction constraints are resolved before normal parenting priority.

A valid `Set` may reparent a surviving child away from an entity with
`PendingDestroy`.

Normal `Set` versus `Clear` conflicts use `PARENTING_PRIORITY`.

The current policy is:

```text
Clear
```

so:

```text
Set(A -> P)
Clear(A)

-> A becomes parentless
```

No-op requests are removed only after conflict resolution so they retain their
conflict intent while priority is being decided.

After ordinary conflicts and no-ops are resolved, the complete planned parent
graph is checked for cycles.

Cyclic `Set` requests are discarded until the resulting graph is acyclic.

## System Order

`ParentingSystem` runs before ordering and focusing:

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

It establishes the hierarchy postconditions required by downstream structural
systems.

## Validation

In debug builds, parenting validation runs immediately after
`ParentingSystem`.

It verifies:

```text
Parent -> Hierarchy
Hierarchy -> Parent
referenced parents exist
no empty hierarchy groups
no parent cycles
valid ParentChanged state
```

Validation is read-only and never repairs state.

## Out of Scope

`ParentingSystem` does not define:

```text
hierarchy depth limits
domain-specific parent/child kinds
actual entity destruction
```
