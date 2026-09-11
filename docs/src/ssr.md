# SSR & hydration

Server: `render_to_string(&vnode)` produces HTML with pre-order `data-rwh="N"` ids,
`data-rwe-<event>` markers for listeners, `<!--rwc:Name-->` at component boundaries,
and `data-rwh-checksum` on the root element.

Client: build the same `VNode`, call `verify_hydration(&html, &vnode)` during
development and testing; in production the hydrator binds existing nodes by id
instead of recreating them. Divergence surfaces as
`HydrationError::{TagMismatch,TextMismatch,Structure,Checksum}`.

Anti-mismatch rules:

1. One `view` function for server and client (never branch on `is_server`).
2. No randomness or wall-clock time in `view` (inject via props/context).
3. Stable child order; dynamic lists must use `key`.
