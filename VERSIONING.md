# Versioning, API stability, deprecation & migration

## Semver commitment

- `0.x` (current, `0.2.0`): **minor = potentially breaking**, patch = fixes only.
  Every breaking change in a minor release must include a `MIGRATION.md` delta plus a `#[deprecated]` shim wherever feasible.
- `1.x` (target): **major = breaking**, minor = additive and backward-compatible,
  patch = fixes. The public API covers all `pub` items across the 7 crates, the `html!`
  semantics, the SSR format (`data-rwh*`), and router behavior. Any SSR format change
  that breaks hydration of previously rendered output counts as breaking (major).

## What counts as public API

1. `pub` signatures in `rustweb-core/dom/router/ssr/testing/macro/cli`.
2. Documented `html!` expansion (syntax plus error-message contract: error messages
   must be actionable, naming the tag/attribute, with a hint and a spec link where relevant).
3. Hydration traversal order (`data-rwh` pre-order) and marker attributes
   (`data-rwh-checksum`, `data-rwe-*`, `<!--rwc:…-->`).
4. `Patch` semantics (paths are index paths over the resolved tree) and `GuardResult`.
5. CLI flags, exit codes, and size-report format.

## Deprecation policy

1. Mark with `#[deprecated(since = "0.N.0", note = "use X instead; see MIGRATION.md#…")]`.
2. Keep for at least **1 minor release** (or 3 months, whichever is longer)
   before removal. Removal happens only in a minor (`0.x`) or major (`1.x`) release.
3. CI runs a `semver-checks` job (`cargo semver-checks` when available, otherwise a
   human-reviewed `cargo public-api` diff) that fails when the public API
   changes without the appropriate release label.

## Migration guide template (per breaking change)

```md
# Migrating 0.N → 0.N+1
## Summary (before/after table per item)
## Mechanical steps (find/replace, codemod if any)
## Real-app diff example (todo-app)
## Rollback plan
```

Initial delta: `MIGRATION.md` (v0.1 → v0.2 was non-breaking; no migration steps yet from a prior major).

## v1.0 exit criteria (see `docs/src/release-v1.md`)

- At least 1 production app, at least 3 browsers x stable/nightly green for 30 days.
- 1M fuzz iterations over diff/patch with no applier divergence.
- Clean axe a11y audit for the sample app.
- Real-browser benchmarks within budget; no unexpired `#[deprecated]` items remaining.
