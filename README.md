# Preview

![image](https://user-images.githubusercontent.com/581407/205440021-d99e2a8e-b83f-4032-a237-2b8d0fca2c6d.png)

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
wasm-pack build --release --all-features --target web
```

# Next goals

- [x] Fill
  - [x] Initial support
- [ ] Line
  - [x] Initial support
  - [x] Anti aliasing
  - [ ] Line joins
  - [ ] Line caps
- [x] Points
  - [x] Initial support
  - [ ] Shapes
- [ ] Move polygon triangulation to worker threads
- [ ] Architecture overhaul
  - [ ] Combine tiles in buckets with same material
  - [ ] Split code in smaller chunks
- [ ] CI
  - [x] Initial
  - [ ] Deployment
  - [ ] Publishing
- [ ] OpenLayers integration
  - [ ] Smooth frame sync

# Run a native example

```sh
cargo run --example window --target `rustc -vV | sed -n 's|host: ||p'`
```

# Run tests

Native unit tests
```sh
LIBGL_ALWAYS_SOFTWARE=true cargo test --target `rustc -vV | sed -n 's|host: ||p'` -- --nocapture
```

WASM Browser Integration tests
```sh
wasm-pack test --chrome --release --features console_log,console_error_panic_hook --test '*'
```

# Useful environment variables

| Name                  | Example |
| --------------------- | ------- |
| WGPU_BACKEND          | gl      |
| LIBGL_ALWAYS_SOFTWARE | true    |
| WINIT_UNIX_BACKEND    | x11     |
| VK_ICD_FILENAMES      | /usr/share/vulkan/icd.d/lvp_icd.x86_64.json |
