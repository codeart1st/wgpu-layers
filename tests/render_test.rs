#[cfg(target_arch = "wasm32")]
mod tests {
  use log::info;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen::JsCast;
  use wasm_bindgen_futures::JsFuture;
  use wasm_bindgen_rayon::init_thread_pool;
  use wasm_bindgen_test::*;

  wasm_bindgen_test_configure!(run_in_browser);

  const BASE64_PREFIX: &str = "data:image/png;base64,";

  #[wasm_bindgen_test]
  async fn empty() {
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

    canvas.set_width(512);
    canvas.set_height(512);

    let size = vec![canvas.width(), canvas.height()];

    wgpu_layers::wasm::start_with_canvas(&canvas).await;

    #[rustfmt::skip]
    let view_matrix = vec![
      1.0, 0.0, 0.0, 0.0,
      0.0, 1.0, 0.0, 0.0,
      0.0, 0.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    ];
    wgpu_layers::render(view_matrix, size);

    let promise = js_sys::Promise::new(&mut move |resolve, _| {
      let cb = Closure::wrap(Box::new(move |blob| {
        let args = js_sys::Array::new();
        args.set(0, JsValue::from(blob));
        resolve.apply(&JsValue::NULL, &args);
      }) as Box<dyn Fn(web_sys::Blob)>);
      canvas.to_blob(cb.as_ref().unchecked_ref());
      cb.forget(); // leaking
    });

    let blob = JsFuture::from(promise).await.unwrap();
    let blob = web_sys::Blob::from(blob);

    let promise = js_sys::Promise::new(&mut move |resolve, _| {
      let file_reader = std::rc::Rc::new(web_sys::FileReader::new().unwrap());
      let file_reader_cb = file_reader.clone();

      let cb = Closure::wrap(Box::new(move |event| {
        let args = js_sys::Array::new();
        info!("{:?}", event);
        let data_url = file_reader_cb.result().unwrap();
        args.set(0, JsValue::from(data_url));
        resolve.apply(&JsValue::NULL, &args);
      }) as Box<dyn Fn(web_sys::Blob)>);

      file_reader.set_onload(Some(&cb.as_ref().unchecked_ref()));
      file_reader.read_as_data_url(&blob);

      cb.forget(); // leaking
    });

    let data_url = JsFuture::from(promise).await.unwrap();

    let expect: String = [
      BASE64_PREFIX,
      include_base64::include_base64!("tests/snapshots/render_test_empty.png"),
    ]
    .join("")
    .replace("_", "/")
    .replace("-", "+");

    assert_eq!(data_url.as_string().unwrap(), expect);
  }
}
