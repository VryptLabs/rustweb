# Error handling & panic safety

Rule: **no panic crosses a component boundary in production**.

- Every lifecycle stage returns `Result<_, ComponentError|RenderError|PropsError>`.
- The runtime wraps calls with `catch_unwind` (`core::runtime::run_guarded`):
  a panic becomes `ComponentError::Panicked { component, stage, location, payload }`.
- A panicking event handler is isolated in `EventDelegator::dispatch` (returns `false` plus a log line).
- The patch applier never panics on a bad path: it returns a recoverable `Err`.
- Error boundaries: override `Component::on_error` to render fallback UI for a failed child.
- Logs: every error carries the component name, the stage, and (for panics) the location,
  ready to forward to the profiler via `RenderProfiler::on_error`.
