#[cfg(target_arch = "wasm32")]
mod tests {
  use wasm_bindgen::JsCast;
  use wasm_bindgen_futures::JsFuture;
  use wasm_bindgen_rayon::init_thread_pool;
  use wasm_bindgen_test::*;

  wasm_bindgen_test_configure!(run_in_browser);

  #[wasm_bindgen_test]
  fn fail() {
    assert_eq!(1, 2);
  }

  #[wasm_bindgen_test]
  async fn test() {
    // FIXME: testing multithreaded wasm is not possible for now, see: https://github.com/rustwasm/wasm-bindgen/issues/2892
    /*let _ = JsFuture::from(init_thread_pool(
      web_sys::window()
        .unwrap()
        .navigator()
        .hardware_concurrency() as usize,
    ))
    .await;*/

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.create_element("canvas").unwrap();
    let canvas = canvas
      .dyn_into::<web_sys::HtmlElement>()
      .map_err(|_| ())
      .unwrap();

    let body = document.body().unwrap();
    let style = canvas.style();

    body.append_child(&canvas).unwrap();
    style.set_property("position", "absolute").unwrap();
    style.set_property("top", "1em").unwrap();
    style.set_property("right", "1em").unwrap();

    let canvas: web_sys::HtmlCanvasElement = canvas
      .dyn_into::<web_sys::HtmlCanvasElement>()
      .map_err(|_| ())
      .unwrap();

    let size = vec![canvas.width(), canvas.height()];

    wgpu_layers::wasm::start_with_canvas(canvas).await;

    #[rustfmt::skip]
    let view_matrix = vec![
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    ];
    wgpu_layers::render(view_matrix, size);
    assert_eq!(1, 55);
  }
}
