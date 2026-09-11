# rustweb — production-grade Rust frontend library (WebAssembly)

Closest reference: Yew. Key difference: production contracts are enforced in code from day one —
no panics across component boundaries, deterministic hydration, root-level event delegation, and CI-enforced performance budgets.

## Workspace

| Crate | Responsibility |
|---|---|
| `rustweb-core` | `VNode`, `Component` trait + lifecycle, `Msg→update→view`, Context API, `Scheduler`, errors, profiling hooks |
| `rustweb-macro` | `html!` proc-macro: ARIA validation, event binding, conditionals/loops via `{expr}`, typed props |
| `rustweb-dom` | Keyed diff (LIS-minimized moves) + patch applier + `EventDelegator` (1 listener per event at the root) |
| `rustweb-router` | Nested routes, guards (`Allow/Redirect/Deny`), lazy routes (`LazyRoute`, code-split boundary) |
| `rustweb-ssr` | Deterministic `render_to_string` + `data-rwh` hydration ids + `verify_hydration` |
| `rustweb-testing` | Unit harness, snapshots (`assert_snapshot`), headless browser helpers |
| `rustweb-cli` | `new/build/serve`, `wasm-bindgen`/`wasm-opt` integration, size budgets |
| `tests/` | Integration suite: snapshots, hydration round-trips, diff/patch properties, `html!` smoke tests, benchmark regression, headless (ignored) |

## Quick start

```bash
cargo test --workspace              # unit + integration (no browser required)
HEADLESS=1 cargo test -- --ignored  # + browser tests (requires WebDriver at $WEBDRIVER_URL)
cargo run -p rustweb-cli -- new my-app
cargo run -p rustweb-cli -- build --budget 200kb
```

## Production contracts (summary)

- **No-panic policy**: every lifecycle stage returns `Result`; the runtime wraps calls with `catch_unwind`, converting panics into `ComponentError::Panicked` isolated at the error boundary. See `crates/core/src/runtime.rs`.
- **Mismatch-free hydration**: SSR and the client walk the tree in the same pre-order with deterministic hydration ids (`data-rwh`). Verify with `verify_hydration`. See `crates/ssr/src/lib.rs`.
- **Event delegation**: never attach `addEventListener` per node. `EventDelegator` keeps one listener per event type at the root; `binding_count` may grow while `root_listener_count` stays small. See `crates/dom/src/events.rs`.
- **Bundles**: `opt-level="z" + lto`, `wasm-opt -Oz`, feature flags (`default/ssr/hydration/a11y`), per-route code splitting via `LazyRoute` + dynamic wasm imports (guide: `docs/src/code-splitting.md`).

## Versioning

See `VERSIONING.md`. Commitments: strict semver, deprecation of at least 1 minor release with `#[deprecated]` plus a migration note, and a migration guide for every breaking change.

## Release status

**v1.0.0-rc.1 (current)** — the public API is frozen for 1.0.0. Every codeable
exit criterion is met and CI-enforced: 10M-iteration fuzz (apply-success
≥ 99.99%), offline a11y audit gating the shipped example, deterministic
hydration, root event delegation, typed props, full CI matrix, and crates
published to crates.io with docs.rs building.

The only remaining gates before the final `v1.0.0` tag are non-codeable:
production burn-in (soak + hydration-mismatch logs) and real-browser
benchmarks (Chrome/Firefox/WebKit) against the budgets in `crates/core/src/perf.rs`.
See `docs/src/release-v1.md`.

Rationale for the rc rather than a direct `1.0.0`: tagging `1.0.0` without
production hours and browser benchmark data would undermine the semver
stability promise itself. The rc freezes the API so integrators can validate
against it during the soak window.
