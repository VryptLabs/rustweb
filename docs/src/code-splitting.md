# Code splitting & bundles

1. **Features**: `rustweb-core` (`ssr/hydration/a11y`), `rustweb-dom` (`ssr`).
   Disable what you do not use via `--no-default-features --features …`.
2. **Release profile**: `opt-level="z"`, `lto`, `panic="abort"` (see the root `Cargo.toml`).
3. **wasm-opt**: `rustweb build --optimize -Oz` runs `wasm-opt` when installed
   (requires `binaryen`). In CI, the `bundle` job compares `dist/` sizes.
4. **Per-route splitting**: mark boundaries with `LazyRoute`. On `wasm32`, the factory
   loads the chunk via a dynamic `import()` emitted by `wasm-bindgen --split`;
   on native, the factory is synchronous so the split point stays explicit and testable.
5. **Budgets**: `rustweb build --budget 200kb` fails when `dist/*.wasm+js` exceeds the limit.
   Set per-route budgets in CI (`BENCH_MULT` for slow runners).
