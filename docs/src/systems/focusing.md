# Focusing System

## Purpose

`FocusingSystem` maintains remembered local focus within the generic entity
hierarchy.

Focus is defined relative to a parent:

> Every parent has exactly one focused direct child.

`Focused` therefore means:

> This entity is the selected child of its current parent.

This is local hierarchical focus. It does not by itself mean that the entity is
the compositor's globally keyboard-focused window.

## State and Ownership

`FocusingSystem` owns:

```text
Focused
```

Only `FocusingSystem` may add or remove `Focused`.

The system reads:

```text
Parent
Hierarchy
Order
ParentChanged
OrderInvalidated
```

and consumes:

```text
FocusRequest
```

No `FocusChanged` transient is currently required.

## Invariants

After `FocusingSystem` runs:

* Every parent with children has exactly one focused direct child.
* Every entity with `Focused` has a `Parent`.
* Entities without a parent do not have `Focused`.
* Only `FocusingSystem` modifies focus state.

`FocusingSystem` assumes the validated postconditions of `ParentingSystem` and
`OrderingSystem`.

It does not independently revalidate hierarchy or ordering state.

## Structural Changes

Focus is repaired after parenting and ordering changes and before explicit focus
requests are processed.

### Focused Child Leaving a Parent

When a focused child leaves its previous parent, its `Focused` marker belongs to
the old relationship and must be removed.

The system uses:

```text
ParentChanged::previous
OrderInvalidated::previous
```

to preserve the old parent and old sibling position.

All stale `Focused` markers from departing focused children are removed before
previous parents are repaired.

This prevents a child that becomes newly focused during repair from having that
new focus removed later in the same pass.

If the previous parent still has children:

```text
replacement_order =
    min(previous_order, child_count - 1)
```

The child currently occupying `replacement_order` becomes focused.

For example:

```text
before:

A(0) B*(1) C(2) D(3)

B leaves
```

After ordering:

```text
A(0) C(1) D(2)
```

With:

```text
B.OrderInvalidated.previous = 1
```

focus becomes:

```text
A(0) C*(1) D(2)
```

If the departed child occupied the last position:

```text
A(0) B(1) C*(2)

C leaves
```

the replacement position is clamped:

```text
A(0) B*(1)
```

If the previous parent no longer has children, no repair is necessary.

### Child Entering a Parent

After previous parents are repaired, each changed child that currently has a
parent and is not focused is considered within its new sibling group.

If the new parent already has a focused child, nothing changes.

Otherwise the entering child becomes focused.

If several changed children enter the same parent while that parent has no
focused child, the first one encountered becomes focused.

HECS query order is intentionally accepted as arbitrary.

## Events

Focus changes are requested through `FocusRequest`.

Every observed focus request event entity is consumed after processing.

### Set

```text
Set(target)
```

For a valid unfocused target:

```text
parent = target.Parent
siblings = Hierarchy[parent]

remove Focused from the currently focused sibling
add Focused to target
```

The target becomes the only focused child of its parent.

A request targeting an entity without a parent is ignored.

A request targeting an already focused entity is a no-op.

If a target has a `Parent`, `FocusingSystem` assumes parenting validation has
already established the corresponding hierarchy relationship.

### Event Ordering

Multiple focus requests in the same run are processed in arbitrary HECS query
order.

Later observed requests may therefore replace the result of earlier requests
within the same sibling group.

## System Order

`FocusingSystem` runs after parenting and ordering:

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
    ↓
focusing validation
```

This lets focusing rely on authoritative hierarchy membership and dense sibling
order.

Transient structural change information remains available until every consumer
has run, then is removed by orchestrator-level transient cleanup.

## Validation

In debug builds, focusing validation runs immediately after `FocusingSystem`.

It verifies the focus-owned postconditions:

```text
every parent has exactly one focused direct child
Focused -> Parent
```

It does not revalidate hierarchy correctness or sibling ordering because those
belong to earlier validation stages.

Validation is read-only and never repairs state.

## Out of Scope

`FocusingSystem` does not define:

```text
global keyboard focus
backend activation
focused-window decoration
rendering
layout
domain-specific entity roles
focus-change notifications
```

A backend or later presentation layer may derive the globally active entity by
following the hierarchy's focused path.
