# Destroy System

## Purpose

`DestroySystem` manages deferred entity destruction.

A destruction request does not immediately remove the target. The target is
first marked with `PendingDestroy`, then removed during finalization.

## State and Ownership

`DestroySystem` owns:

```text
PendingDestroy
```

and consumes:

```text
DestroyRequest
```

## Prepare

`prepare` processes every `DestroyRequest`.

For a valid target:

```text
DestroyRequest(target)
    ↓
target += PendingDestroy
```

A request targeting a missing entity is ignored.

A target already carrying `PendingDestroy` is left unchanged.

Every observed request entity is consumed after processing.

## Finalize

`finalize` despawns every entity carrying:

```text
PendingDestroy
```

After finalization, no entity marked for destruction remains in the world.

## Request Lifetime

`DestroyRequest` is expected to exist on a temporary request entity.

The request entity is consumed during `prepare`.

The target itself is not the request entity.

## Invariants

After `prepare`:

```text
valid destruction targets have PendingDestroy
observed DestroyRequest entities have been consumed
```

After `finalize`:

```text
no entity with PendingDestroy remains
```

## Out of Scope

`DestroySystem` does not define:

```text
relationship repair
ordering repair
focus repair
recursive destruction
domain-specific lifecycle policies
external resource cleanup
```
