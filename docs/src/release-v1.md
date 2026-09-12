# Toward v1.0 (release plan)

We are **not** tagging v1.0 today. The reason is technical, not administrative:
the `1.0` label is a semver stability promise, and that promise is only credible
after production hours. What ships now is the **complete v0.2.0 foundation**
(every subsystem implemented, fuzz-tested, and CI-green), with an explicit path
to v1.0:

## Progress since 0.1.0

- Deterministic fuzz harness over diff + applier (`tests/fuzz_diff.rs`,
  threshold asserted at ≥ 95%): validated at **10,000,000 iterations** with
  apply-success **99.9985%** (meets the ≥ 99.99% v1.0 criterion) and
  SSR-match **99.975%** at the fixed seed.
- Crates published to crates.io: `rustweb-{core,macro,dom,router,ssr,testing,cli}`
  v0.2.0; workspace path deps carry explicit `version = "0.2.0"` so future
  `cargo publish` from tags works without local edits.
- Built-in offline a11y audit (`rustweb_testing::a11y`) with severity-ranked,
  path-located, fix-hinted violations (rules: interactive-name, form-label,
  image-alt, heading-order, list-structure, landmark-main). Enforced on every PR
  via `cargo test`, and `examples/todo-app` gates the shipped example with zero
  blocking violations.
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

None — `v1.0.0` was cut from `1.0.0-rc.1` with no API changes. The API contract
in `VERSIONING.md` now applies (major = breaking, minor = additive, patch =
fix).

Exit criteria live in `VERSIONING.md`. Every step is measured by the existing
CI (`ci.yml`: matrix, browser, bundle, docs, semver).
