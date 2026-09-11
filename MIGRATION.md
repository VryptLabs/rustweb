# Migration guide

No breaking changes between 0.1.0 and 0.2.0. Public API, `html!` semantics,
SSR marker format, and router behavior are additive.

## 0.1 → 0.2

### Breaking
- None.

### Added (non-breaking)
- `Renderer::update(component, next) -> Result<usize, ComponentError>`
  — pairs diff and apply and debug-asserts `committed == next`.
- `EventDelegator::root_listener_count()` — verifies 1 root listener per event
  type regardless of binding count.
- `UnnecessaryRenderDetector` — install via
  `rustweb_core::set_global_profiler`.

### Internal changes worth knowing
- Diff `Remove` patches now use OLD-tree navigation; `Create`/`Move`/`SetAttr`
  use NEW-tree navigation. This does not change the `Patch` public API.
- `set_attr_at` removes-then-pushes, so attribute order after a patch matches
  the diff target order.

### Mechanical steps
1. `cargo update` (workspace deps already bumped to `thiserror 2`, `syn 3` —
   compatible with the previous derive usage).
2. Rebuild the docs: `mdbook build docs`.

### Rollback
- Pin to `rustweb-* = "0.1"`; SSR markers are byte-identical, so hydration of
  previously served HTML still works.

## Future breaking releases

Every future breaking release must add a section below using this format:

```md
## 0.N → 0.N+1
### Breaking
- …
### Mechanical steps
1. …
### Example diff
```diff
…
```
### Rollback
- …
```
