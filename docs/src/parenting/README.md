# Parenting System

The parenting system owns parent-child relationships in the ECS.

`Parent` is authoritative. `children_by_parent` is a derived reverse index maintained by `ParentingSystem`.

Other systems do not modify hierarchy state directly. They request changes through parenting events.

## Documents

- [01-data-flow.md](01-data-flow.md) — pipeline stages and the data produced by each stage.
- [02-invariants.md](02-invariants.md) — ownership and correctness rules.
- [03-conflicts-and-priority.md](03-conflicts-and-priority.md) — destruction conflicts, normal parenting conflicts, and priority.
- [04-hierarchy-cache.md](04-hierarchy-cache.md) — cache authority, maintenance, and cleanup.
- [05-implementation-contract.md](05-implementation-contract.md) — expected implementation shape and phase boundaries.

## Core Model

```text
Parent component:
    child -> parent

Derived hierarchy:
    parent -> { children }
```

The system resolves requests as data before mutating the hierarchy:

```text
query
  ↓
normalize SetParent
  ↓
build destroy clears
  ↓
resolve destroy clears vs SetParent
  ↓
lower ClearChildren
  ↓
merge clears
  ↓
resolve normal parenting priority
  ↓
remove no-ops
  ↓
apply
  ↓
hierarchy cleanup
  ↓
consume events
```

`PendingDestroy` is a hard constraint. `ParentingPriority` only decides conflicts between normal parenting set and clear intent.
