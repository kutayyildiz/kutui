# Hierarchy Cache

## Authority

`Parent` is the authoritative relationship state.

```text
child -> Parent(parent)
```

`children_by_parent` is a derived reverse index:

```text
parent -> { children }
```

It exists for efficient parent-to-children lookup.

## Initialization

`children_by_parent` may be initialized lazily from existing `Parent`
components.

After initialization, `ParentingSystem` maintains it incrementally.

Under correct usage it does not become stale because no other system may modify
hierarchy-related state.

## Representation Rule

Only entities with at least one child appear as hierarchy keys.

Valid:

```text
P -> { A, B }
```

Invalid after cleanup:

```text
P -> {}
```

A leaf with a `Parent` component does not need its own hierarchy entry unless it
also has children.

## Clear

For:

```text
C -> P
```

clearing `C` performs:

```text
remove Parent from C
remove C from children_by_parent[P]
```

## Set / Reparent

For:

```text
C -> P
```

becoming:

```text
C -> Q
```

apply:

```text
remove C from children_by_parent[P]
set Parent(Q) on C
insert C into children_by_parent[Q]
```

`Parent` and `children_by_parent` are changed as one logical operation.

## PendingDestroy

`PendingDestroy` does not mutate the hierarchy during resolution.

Instead, current relationships involving destroyed entities are represented as
temporary destroy clears.

After all conflicts are resolved, the resulting clear/set plan is applied
normally.

This keeps hierarchy mutation in one place and allows surviving children to be
reparented safely in the same run.

## Cleanup Pass

After all resolved parenting changes are applied:

```text
remove hierarchy entries whose child set is empty
```

The cleanup pass guarantees the representation rule for the next run.

It is not a repair mechanism for arbitrary cache corruption.

## Cache Mismatch

If ECS `Parent` state and `children_by_parent` disagree under correct usage, the
hierarchy invariant has been violated.

The system should not silently choose one representation and repair the other
during normal operation.

### TODO

Consider a debug-only verification facility that checks:

```text
every Parent edge exists in children_by_parent
every children_by_parent edge matches Parent
no hierarchy entry is empty
all referenced entities exist
```
