use raw_window_handle::HasRawWindowHandle;

mod renderer;
mod offscreen_window;
use log::info;

#[cfg(target_arch = "wasm32")]
mod wasm {
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  #[wasm_bindgen]
  pub async fn start(canvas: web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    let window = super::offscreen_window::OffscreenWindow::new();
    super::init(&window, (canvas.width(), canvas.height())).await;
  }
}

pub async fn init<W: HasRawWindowHandle>(window: &W, size: (u32, u32)) {
  let renderer = renderer::Renderer::new(window, size).await;

  info!("renderer initialized");

  renderer.test_draw();
}