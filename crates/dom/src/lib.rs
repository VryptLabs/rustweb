pub mod diff;
pub mod events;
pub mod patch;
pub mod renderer;

pub use diff::diff;
pub use events::{DelegatedEvent, EventDelegator};
pub use patch::{Patch, PatchPath};
pub use renderer::Renderer;

#[cfg(all(test, target_arch = "wasm32"))]
mod bench_wasm;
