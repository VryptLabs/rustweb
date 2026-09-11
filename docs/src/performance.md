# Performance

- Install a profiler: `set_global_profiler(UnnecessaryRenderDetector::default())`
  once at startup; read `unnecessary()/total()` in tests and development.
- Avoid re-renders: narrow `should_render` (the default already compares `old != new` for props),
  use stable `key` values on lists, split large components.
- Events: keep `root_listener_count()` small (1 per type) even as `binding_count()` grows.
- Measure with `tests/bench_regression.rs`; in a real browser compare `view_time` against
  `patch_time` via `RenderSample`.
