# Orchestrator

## Purpose

`Orchestrator` provides the canonical entry point for running the core systems.

It owns the system instances and controls when transient state is cleaned up.

## State

`Orchestrator` contains:

```text
DestroySystem
ParentingSystem
OrderingSystem
FocusingSystem
```

It retains these system instances across calls to `run`.

## Run

The core is advanced through:

```rust
orchestrator.run(&mut world);
```

A run processes queued requests and updates core state.

In debug builds, system-owned invariants are validated during the run.

## Transient Cleanup

Some state exists only to communicate changes during a single run.

Currently:

```text
ParentChanged
OrderInvalidated
```

These components are removed before `run` returns.

They therefore must not be treated as persistent state.

## Hierarchy

The current hierarchy can be accessed through:

```rust
orchestrator.hierarchy()
```

The returned hierarchy is the hierarchy maintained internally by the core.

## Lifetime

An `Orchestrator` is stateful and should normally remain associated with the
`World` it manages.

Typical usage is:

```rust
let mut world = World::new();
let mut orchestrator = Orchestrator::new();

orchestrator.run(&mut world);
```

The same orchestrator should then be reused for subsequent runs.

## Invariants

After `run` returns:

```text
processed request entities have been consumed
PendingDestroy entities have been finalized
ParentChanged has been removed
OrderInvalidated has been removed
```

## Out of Scope

`Orchestrator` does not define:

```text
application frame timing
render scheduling
layout execution
backend event loops
input routing
asynchronous scheduling
```
