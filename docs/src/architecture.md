# Architecture

```text
Component::view → VNode ─┬─→ dom::diff → Patch[] → Renderer (web-sys) → DOM
                         └─→ ssr::render_to_string → HTML + data-rwh
Event: root listener → EventDelegator → Link::send(Msg) → Scheduler → update → view
```

- `rustweb-core`: types and traits, no `web-sys` dependency (unit-testable on the host).
- `rustweb-dom`: the only crate that touches `web-sys` (on `wasm32`).
- `rustweb-ssr`: pure string rendering, deterministic.
- `rustweb-router`: view-agnostic (resolves to a view name plus params).
