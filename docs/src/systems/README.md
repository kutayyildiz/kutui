# Systems

The core uses small systems with clear ownership and explicit dependencies.

## Current Systems

* [**ParentingSystem**](parenting.md) — Maintains parent-child relationships and
  the hierarchy.
* [**OrderingSystem**](ordering.md) — Maintains dense ordering between siblings.
* [**FocusingSystem**](focusing.md) — Maintains remembered local focus within
  each parent.

## Order

The canonical execution order is:

```text
DestroySystem::prepare
        ↓
ParentingSystem
        ↓
OrderingSystem
        ↓
FocusingSystem
        ↓
DestroySystem::finalize
        ↓
transient cleanup
```

Each system may rely on the invariants established by the systems before it.

In debug builds, each system is validated immediately after it runs.

## Dependency Chain

```text
Parenting → Ordering → Focusing
```

## Design

Systems:

* own the state they modify;
* receive changes through events;
* expose transient change state only when required by downstream systems;
* remain generic and independent of backend-specific concepts.

Detailed behavior and invariants are documented in the individual system
documents above.

## Requests

Changes are submitted by spawning temporary entities containing request
components. A system consumes the requests it owns and despawns their request
entities after processing.

Request components therefore belong on dedicated disposable entities.
