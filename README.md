# Preview

![image](https://user-images.githubusercontent.com/581407/190889486-81f80bce-3ee1-40e5-9aa2-285987e3beeb.png)

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

# Run tests

```sh
LIBGL_ALWAYS_SOFTWARE=true WGPU_BACKEND=gl cargo test --target `rustc -vV | sed -n 's|host: ||p'` -- --nocapture
```