# Routing

Nested routes, guards, lazy loading:

```rust,ignore
Router::new().route(
  Route::new("users", "Users")
    .guard(|ctx| if ctx.principal.is_some() { Allow } else { Redirect("/login".into()) })
    .child(Route::new(":id", "UserDetail")
      .child(Route::new("edit", "UserEdit").guard(can_edit)))
    .child(Route::new("*", "FileView")), // wildcard → params["wildcard"]
)
```

Lazy loading (code-split boundary):

```rust,ignore
Route::new("settings", "Settings").lazy_children(LazyRoute::new(|| {
  // on wasm: dynamically import the split chunk; on native: construct directly
  Ok(vec![Route::new("profile", "Profile")])
}))
```

The factory runs exactly once (`OnceCell`); the `lazy_loads_once` test asserts this.
Redirects are followed for up to 8 hops, beyond which a recoverable `RedirectLoop` is returned.
