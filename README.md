# Preview

![image](https://user-images.githubusercontent.com/581407/190889486-81f80bce-3ee1-40e5-9aa2-285987e3beeb.png)

---
<div align="center">
  <strong>WebGPU mapping renderer for OpenLayers</strong>
</div>
<div align="center">
  Currently only a playground for rust, wgpu, openlayers web mapping combo
</div>
<br>
<div align="center">
  <a href="https://github.com/codeart1st/wgpu-layers/actions/workflows/ci.yml">
    <img src="https://github.com/codeart1st/wgpu-layers/actions/workflows/ci.yml/badge.svg" alt="Build status"/>
  </a>
  <a href="https://github.com/codeart1st/wgpu-layers/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/codeart1st/wgpu-layers" alt="MIT license"/>
  </a>
</div>

# Build the project

```sh
wasm-pack build --all-features --target web --dev
```

# Run a native example

```sh
WINIT_UNIX_BACKEND=x11 cargo run --example window --target `rustc -vV | sed -n 's|host: ||p'`
```

# Run tests

Native unit tests
```sh
LIBGL_ALWAYS_SOFTWARE=true WGPU_BACKEND=gl cargo test --target `rustc -vV | sed -n 's|host: ||p'` -- --nocapture
```

WASM Browser Integration tests
```sh
wasm-pack test --chrome --all-features --test '*'
```