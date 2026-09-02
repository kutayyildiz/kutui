# Data Flow

## Internal State

```rust
struct ParentingEvents {
    set_parent: HashMap<Entity, Entity>,
    clear_parent: HashSet<Entity>,
    clear_children: HashSet<Entity>,
    entities: Vec<Entity>,
}

pub struct ParentingSystem {
    children_by_parent: Option<Hierarchy>,
}

const PARENTING_PRIORITY: ParentingPriority = ParentingPriority::Clear;
```

The pipeline also uses temporary sets:

```text
pending_destroy

destroy_clear_parent
```

`destroy_clear_parent` is kept separate from normal `ClearParent` intent until
destruction conflicts are resolved.

## Pipeline

| Stage | Reads | Modifies | Result |
|---|---|---|---|
| **1. Query** | `SetParent`, `ClearParent`, `ClearChildren`, `PendingDestroy` | temporary data | Parenting requests and destruction targets are collected |
| **2. Normalize SetParent** | set requests, entity existence, `PendingDestroy` | `set_parent` | Invalid sets and sets involving destroyed entities are discarded; one surviving set remains per child |
| **3. Build destroy clears** | `PendingDestroy`, `Parent`, hierarchy | `destroy_clear_parent` | Existing relationships that must be detached are represented as clear intent |
| **4. Resolve destroy conflicts** | `set_parent`, `destroy_clear_parent` | both | A valid reparenting set may replace a destroy-generated clear |
| **5. Lower ClearChildren** | hierarchy, `set_parent`, `clear_children` | `clear_parent`, `clear_children` | Bulk clears become normal per-child clear intent |
| **6. Merge clears** | `destroy_clear_parent`, `clear_parent` | `clear_parent` | All surviving clear operations share one set |
| **7. Resolve parenting priority** | `set_parent`, `clear_parent` | both | Normal Set/Clear conflicts are resolved by `PARENTING_PRIORITY` |
| **8. Remove no-ops** | current `Parent` state, resolved events | both | Remaining operations require a state change |
| **9. Apply** | resolved sets and clears | `Parent`, hierarchy | Resolved relationship changes are committed |
| **10. Cleanup** | hierarchy | hierarchy | Empty hierarchy entries are removed |
| **11. Consume** | parenting event entity IDs | world | All processed parenting events are despawned |

## Queries

The system performs four logical queries:

```text
SetParent
ClearParent
ClearChildren
PendingDestroy
```

The three parenting event queries contribute to `ParentingEvents`.

`PendingDestroy` is collected separately and is never consumed by
`ParentingSystem`.

No hierarchy mutation occurs during query or conflict resolution.

## SetParent Normalization

A set is discarded if:

```text
its child does not exist
its parent does not exist
its child is PendingDestroy
its parent is PendingDestroy
```

Among remaining `SetParent` requests for the same child, the first encountered
request survives. HECS query order is intentionally accepted as arbitrary.

## Destroy Clears

For every `PendingDestroy(D)`:

```text
if D currently has a parent:
    destroy_clear_parent += D

for every current child C of D:
    destroy_clear_parent += C
```

A relationless destroyed entity generates no clear, but it still invalidates any
`SetParent` that references it.

## Destroy Conflict Resolution

After invalid sets are removed, a surviving `SetParent(C, Q)` is safe because
neither endpoint is being destroyed.

Therefore:

```text
SetParent(C, Q)
vs
destroy_clear_parent(C)

-> SetParent(C, Q)
```

This lets a surviving child move away from a parent that is being destroyed.

## ClearChildren Lowering

`ClearChildren(P)` becomes normal `ClearParent` intent for:

```text
current children of P
+
surviving prospective children from SetParent(_, P)
```

Then:

```text
clear_children has been fully consumed and is ignored by later phases
```

## Normal Priority

After destroy conflicts are resolved and all clears are merged, normal parenting
priority applies only to:

```text
SetParent(child, parent)
vs
ClearParent(child)
```

After this stage:

```text
set_parent.keys() ∩ clear_parent = ∅
```

## Phase Guarantees

Before apply:

```text
all surviving SetParent endpoints exist
no surviving SetParent references PendingDestroy
clear_children is no longer used after lowering
set_parent and clear_parent are disjoint
```

After apply and cleanup:

```text
Parent and children_by_parent match
children_by_parent contains no empty child sets
```
