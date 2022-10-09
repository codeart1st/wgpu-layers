# Preview

![image](https://user-images.githubusercontent.com/581407/190889486-81f80bce-3ee1-40e5-9aa2-285987e3beeb.png)

# Build the project

```sh
wasm-pack build --target web
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
wasm-pack test --chrome --test '*'
```