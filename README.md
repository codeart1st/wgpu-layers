# Browser support

Chrome Development with enable-unsafe-webgpu working.
Firefox dom.webgpu.enabled and gfx.offscreencanvas.enabled. Missing compositingAlphaMode support.

# Build the project

```sh
wasm-pack build --target web
```

# Run a native example

```sh
cargo run --example window --target `rustc -vV | sed -n 's|host: ||p'`
```