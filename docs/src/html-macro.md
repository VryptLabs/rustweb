# html! macro

```rust,ignore
html! {
  <main class="app" aria-label="App">
    <h1>{ &title }</h1>
    <button onclick={ on_add } aria-pressed={ pressed }>{ "Add" }</button>
    { if empty { html!{ <p>"empty"</p> } } else { html!{ <List items={ items } /> } } }
    { items.iter().map(|i| html!{ <li key={ i.id }>{ &i.name }</li> }).collect::<Vec<_>>() }
  </main>
}
```

- `aria-*`/`role` attributes pass through; typos such as `aria-lable` are **compile errors**
  with a suggestion (`Did you mean aria-label?`).
- `on*`/`on:*` are event bindings (delegated at the root), not attributes.
- `{expr}` uses the `HtmlChild` trait: `VNode`, strings, numbers, `Option`, `Vec`,
  `bool` (`false` renders nothing). Conditionals and loops use plain Rust.
- `<Button label="Save" />` builds `ButtonProps { label: "Save".into(), … }`,
  so missing or mistyped props are ordinary Rust errors at the call site.
  Components that accept children must declare a `pub children: Vec<VNode>` field.
