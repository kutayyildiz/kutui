# KutUI

KutUI is a data-oriented UI library built around an Entity Component System.

The core crate models generic presentation state such as hierarchy, ordering,
focus, and lifecycle without depending on a specific renderer or platform.

Rendering and platform integration live outside the core crate, allowing
adapters and renderers to build on the same core model.

## Structure

```text
crates/
└── core/
    └── src/
        ├── components/
        └── systems/
```

The project is still experimental and the architecture may change as the library
grows.
