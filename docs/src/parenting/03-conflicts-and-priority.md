# Parenting System — Conflicts and Priority

There are two different conflict classes.

```text
destruction constraints
        ↓
normal parenting priority
```

They must not be treated as the same kind of clear.

## 1. Destruction Constraints

`PendingDestroy` means the entity may not remain part of a final parent
relationship.

For:

```text
P -> D -> C
PendingDestroy(D)
```

resolution generates destroy clears for the existing edges:

```text
destroy_clear_parent(D)
destroy_clear_parent(C)
```

These are temporary resolution data, not `ClearParent` events.

### Sets Involving Destroyed Entities

The following are always discarded:

```text
SetParent(D, X)  // destroyed child
SetParent(X, D)  // destroyed parent
```

This rule bypasses `ParentingPriority`.

### Reparenting a Former Child

Given:

```text
D -> C
PendingDestroy(D)
SetParent(C, Q)
```

`SetParent(C, Q)` is valid because neither `C` nor `Q` is being destroyed.

It therefore wins over the destroy-generated clear for `C`:

```text
C -> Q
```

Destruction prevents relationships involving `D`; it does not force surviving
children to remain orphaned.

## 2. Normal Parenting Priority

Normal clear intent comes from:

```text
ClearParent
ClearChildren -> ClearParent
```

After destruction conflicts are resolved, surviving destroy clears are merged
into the clear set. They no longer conflict with any surviving set.

`ParentingPriority` then applies only to normal Set/Clear conflicts:

```text
SetParent(A, P)
ClearParent(A)
```

With `ParentingPriority::Clear`:

```text
-> ClearParent(A)
```

With `ParentingPriority::Set`:

```text
-> SetParent(A, P)
```

## ClearChildren

`ClearChildren(P)` is lowered before normal priority resolution.

It covers both current and prospective children.

```text
P -> A
ClearChildren(P)
SetParent(B, P)
```

becomes normal clear intent for:

```text
A
B
```

Any conflict with `SetParent` is then resolved by `PARENTING_PRIORITY`.

This also handles:

```text
D -> C
PendingDestroy(D)
ClearChildren(D)
SetParent(C, Q)
```

The destroy clear allows the valid reparenting set to survive the destruction
pass. `ClearChildren(D)` independently creates normal clear intent for `C`, so
the final result is decided by normal parenting priority:

```text
Clear priority -> C has no parent
Set priority   -> C -> Q
```

## Set-vs-Set

HECS query ordering is arbitrary.

After invalid sets are discarded, the first encountered valid `SetParent` for a
child survives and later sets for that child are discarded.

```text
SetParent(A, B)
SetParent(A, C)

-> first encountered valid request survives
```

This intentionally inherits HECS query iteration order.

## No-op Elimination

No-op elimination happens after all conflict resolution.

A no-op may still carry conflict intent before priority is resolved.

```text
A -> B
SetParent(A, B)
ClearParent(A)
```

With Set priority, the set must defeat the clear before the set may be removed
as a no-op.

A losing directive is never reconsidered after the winner becomes a no-op.
