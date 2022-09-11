# Preview

![image](https://user-images.githubusercontent.com/581407/189522082-0e56db05-731c-4df7-b49e-d6e7e932ba4e.png)

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
