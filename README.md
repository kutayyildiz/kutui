# Project

A small, data-oriented library for organizing presentation state using an Entity Component System.

The project models how content is organized, related, ordered, focused, and positioned without assuming what that content represents or how it is rendered.

Rendering and platform-specific behavior are intentionally outside the scope of the core. Other systems can consume the resulting state and realize it however they need.

## Architecture

The current source structure follows the main ECS categories:

```text
src/
├── entities/
├── components/
└── systems/
```

This structure is intentionally simple and may be reorganized into higher-level domains as the project grows.
