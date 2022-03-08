# Browser support

Chrome Development with enable-unsafe-webgpu working.

Firefox dom.webgpu.enabled and gfx.offscreencanvas.enabled but still missing interop:
* https://bugzilla.mozilla.org/show_bug.cgi?id=1753302
* https://bugzilla.mozilla.org/show_bug.cgi?id=1698612

# Build the project

```sh
wasm-pack build --target web
```