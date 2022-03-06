#[cfg(target_arch = "wasm32")]
mod wasm {
  use wasm_bindgen::prelude::*;
  use log::info;

  // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
  // allocator.
  #[cfg(feature = "wee_alloc")]
  #[global_allocator]
  static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

  #[wasm_bindgen]
  extern {
    fn alert(s: &str);
  }

  #[wasm_bindgen]
  pub fn greet() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    info!("Hello, wgpu-layers!");
  }
}
