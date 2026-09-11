# Toward v1.0 (release plan)

We are **not** tagging v1.0 today. The reason is technical, not administrative:
the `1.0` label is a semver stability promise, and that promise is only credible
after production hours. What ships now is the **complete v0.2.0 foundation**
(every subsystem implemented, fuzz-tested, and CI-green), with an explicit path
to v1.0:

## Progress since 0.1.0

- Deterministic fuzz harness over diff + applier: 100k–1M iterations,
  apply-success rate and SSR-match rate ≥ 99.9% at default seed
  (`tests/fuzz_diff.rs`, threshold asserted at ≥ 95%).
- Golden SSR tests freeze the `data-rwh` / `data-rwe-*` / `data-rwh-checksum` /
  `<!---->` / `<!--rwc:-->` contract (`tests/ssr_golden.rs`).
- Router now recurses into nested index children and lazy children
  (`nested_index_recurses_into_grand_children`, `index_without_children_returns_own_view`).
- End-to-end runtime loop exercised in `examples/todo-app`: create → view →
  `Msg::Add`/`Toggle` → update → diff → apply → SSR → `verify_hydration`.
- CI matrix (fmt, clippy, test on 3 OSes × stable/nightly, wasm32, chrome +
  firefox, bundle, docs, semver) is green.
- Release, audit, and Pages workflows are in place.

## Remaining exit criteria for v1.0

1. **Phase 0.3–0.5**: adopt in one internal production app; collect
   `RenderSample` and hydration mismatch logs; freeze the `data-rwh*` format
   across releases.
2. **Phase 0.6–0.8**: raise fuzz budget to 10M iterations and require ≥ 99.99%
   apply success; add axe-core a11y audit to the browser job; measure real
   browser benchmarks (Chrome / Firefox / WebKit) against budgets set in
   `crates/core/src/perf.rs`.
3. **1.0-rc**: remove all expired `#[deprecated]` items; finalise
   `MIGRATION.md`; request public API review from users.
4. **1.0**: tag plus staged release in dependency order
   (core → macro → dom/router/ssr/testing → cli).

Exit criteria live in `VERSIONING.md`. Every step is measured by the existing
CI (`ci.yml`: matrix, browser, bundle, docs, semver).
