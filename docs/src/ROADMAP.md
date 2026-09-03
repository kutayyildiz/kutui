# ROADMAP

## Core structure

- [x] **Entity roles** — Define the core entity types and their
      responsibilities.
- [x] **Parenting** — Manage generic parent-child relationships between
      entities.
- [x] **Ordering** — Manage the order of entities within a parent.
- [x] **Focus** — Manage focused entities and remembered local focus.
- [x] **Lifecycle** — Manage entity destruction and other lifecycle transitions
      without violating structural invariants.
- [x] **Orchestration** — Define the canonical system execution order, debug
      validation boundaries, and transient cleanup.

## Interaction

- [ ] **Controller** — Provide the semantic interface used to request changes
      from the core.

## Spatial model

- [ ] **Geometry** — Define generic spatial concepts such as position, size, and
      bounds.
- [ ] **Constraints** — Represent generic sizing and layout constraints.
- [ ] **Layout contract** — Define how layouts receive state and produce spatial
      results.
- [ ] **Basic layouts** — Implement the initial generic layout types.
- [ ] **Nested layouts** — Support layouts containing other layout-managed
      containers.

## Backend boundary

- [ ] **Desired presentation state** — Define the generic presentation state
      produced by the core for external backends.
- [ ] **Backend observations** — Define the generic state and events reported by
      external backends into the core.
- [ ] **Backend contract** — Define the interface between the core and backend
      adapters.
- [ ] **Backend adapter** — Implement an initial concrete backend adapter.

## Later

- [ ] **Optimizations** — Add performance improvements only when measurements
      show they are necessary.
