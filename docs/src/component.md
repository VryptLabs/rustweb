# Components & lifecycle

Order: `validate props → create → view → mounted → (msg → update → should_render? → before_update → view → updated)* → before_unmount`.

Every stage returns `Result` and executes behind `catch_unwind`
(`core::runtime::run_guarded`). Error-boundary example: override `on_error`
to render a fallback for a failed child subtree.

Context API: a `ContextMap` is cloned down the tree. A provider inserts
`T: Clone + 'static`; descendants read it via `ctx.get_context::<T>()` with no prop drilling.
