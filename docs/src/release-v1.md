# Toward v1.0 (release plan)

We are **not** tagging v1.0 today. The reason is technical, not administrative:
the `1.0` label is a semver stability promise, and that promise is only credible
after production hours. What ships now is the **complete v0.1.0 foundation**
(every subsystem implemented and tested), with an explicit path to v1.0:

1. **Phase 0.1–0.3 (now–stabilization)**: adopt in 1 internal app; collect
   `RenderSample` data and mismatch logs; freeze the `data-rwh*` format.
2. **Phase 0.4–0.6**: fuzz diff/patch (target: 1M iterations), axe a11y audit,
   real-browser benchmarks (Chrome/Firefox/WebKit), `cargo semver-checks` in CI.
3. **1.0-rc**: remove all expired `#[deprecated]` items, finalize `MIGRATION.md`,
   request public API review.
4. **1.0**: tag plus staged release (crates in dependency order).

Exit criteria live in `VERSIONING.md`. Every step is measured by the existing CI
(`ci.yml`: matrix plus browser, benchmark, and bundle jobs).
