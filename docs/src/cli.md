# CLI

```sh
rustweb new my-app --dir ./my-app
rustweb build --package my-app --features ssr --optimize -Oz --budget 200kb
rustweb serve --port 8080 --dir dist
```

`build` runs `cargo build --target wasm32-unknown-unknown --release`,
then `wasm-bindgen` plus `wasm-opt` when installed, then reports the
`dist/*.wasm+js` size and enforces `--budget`.
