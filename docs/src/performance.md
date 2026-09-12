# Performance

## Profiling in an app

- Install a profiler: `set_global_profiler(UnnecessaryRenderDetector::default())`
  once at startup; read `unnecessary()/total()` in tests and development.
- Avoid re-renders: narrow `should_render` (the default already compares `old != new` for props),
  use stable `key` values on lists, split large components.
- Events: keep `root_listener_count()` small (1 per type) even as `binding_count()` grows.
- In a real browser compare `view_time` against `patch_time` via `RenderSample`.

## Benchmark harnesses (CI-enforced)

- **Budget gate** — `tests/bench_regression.rs` fails CI when diff/apply/SSR of
  1000 rows exceed fixed time budgets (tune with `BENCH_MULT` on slow runners).
- **Native statistical benchmarks** — `cargo bench -p rustweb-tests --bench render`
  (Criterion) over 10/100/1000/5000-row keyed lists for `diff`, `apply`, and
  `render_to_string`, plus a small hot-path case. The `bench` CI job saves a
  `ci` baseline and uploads the HTML report as an artifact.
- **Real-browser benchmarks** — `crates/dom/src/bench_wasm.rs` runs the same
  hot paths compiled to `wasm32` inside headless Chrome via `wasm-bindgen-test`,
  timing them with `performance.now()` and asserting per-op budgets. The
  `wasm-bench` CI job runs `wasm-pack test crates/dom --headless --chrome --release`.

