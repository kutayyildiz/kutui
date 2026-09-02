# Parenting System

The parenting system owns parent-child relationships in the ECS.

`Parent` is authoritative. `children_by_parent` is a derived reverse index
maintained by `ParentingSystem`.

Other systems do not modify hierarchy state directly. They request changes
through parenting events.

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

`PendingDestroy` is a hard constraint. `ParentingPriority` only decides
conflicts between normal parenting set and clear intent.
