## Summary

<!-- One paragraph: what changes and why. -->

## Checklist

- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace --all-targets` passes
- [ ] Public API changes labeled (`breaking` / `deprecated`) and `MIGRATION.md` updated if needed
- [ ] Docs updated (`docs/src/` and rustdoc where applicable)
- [ ] No panics introduced across component boundaries (all fallible paths return `Result`)

## Test plan

<!-- Commands run and their results. -->
