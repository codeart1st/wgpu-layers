mod renderer;
mod offscreen_window;

#[cfg(target_arch = "wasm32")]
mod wasm {
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::JsCast;
  use log::info;

  // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
  // allocator.
  #[cfg(feature = "wee_alloc")]
  #[global_allocator]
  static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

  #[wasm_bindgen]
  pub async fn start(canvas: web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    let context = canvas
      .get_context("webgpu")
      .unwrap()
      .unwrap()
      .dyn_into::<web_sys::GpuCanvasContext>()
      .unwrap();

    info!("{:?}", context);

    let window = super::offscreen_window::OffscreenWindow::new();
    let renderer = super::renderer::Renderer::new(&window, (canvas.width(), canvas.height())).await;
  }
}
