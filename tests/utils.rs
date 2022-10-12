#![allow(unused_attributes)]
#![feature(thread_local)]
#![cfg(target_arch = "wasm32")]

use log::info;
use std::cell::RefCell;
use std::sync::Once;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;

pub const CANVAS_SIZE: (u32, u32) = (512, 512);

const BASE64_PREFIX: &str = "data:image/png;base64,";

static INIT: Once = Once::new();

#[thread_local]
pub static CANVAS: RefCell<Option<web_sys::HtmlCanvasElement>> = RefCell::new(None);

pub fn initialize() {
  INIT.call_once(|| {
    // FIXME: testing multithreaded wasm is not possible for now, see: https://github.com/rustwasm/wasm-bindgen/issues/2892
    /*let _ = JsFuture::from(wasm_bindgen_rayon::init_thread_pool(
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

    let (width, height) = CANVAS_SIZE;
    canvas.set_width(width);
    canvas.set_height(height);

    *CANVAS.borrow_mut() = Some(canvas);
  });
}

pub async fn canvas_as_data_url(canvas: &HtmlCanvasElement) -> JsValue {
  let promise = js_sys::Promise::new(&mut move |resolve, _| {
    let cb = Closure::wrap(Box::new(move |blob| {
      let args = js_sys::Array::new();
      args.set(0, JsValue::from(blob));
      resolve.apply(&JsValue::NULL, &args).unwrap();
    }) as Box<dyn Fn(web_sys::Blob)>);
    canvas.to_blob(cb.as_ref().unchecked_ref()).unwrap();
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
      args.set(0, data_url);
      resolve.apply(&JsValue::NULL, &args).unwrap();
    }) as Box<dyn Fn(web_sys::Blob)>);

    file_reader.set_onload(Some(cb.as_ref().unchecked_ref()));
    file_reader.read_as_data_url(&blob).unwrap();

    cb.forget(); // leaking
  });
  JsFuture::from(promise).await.unwrap()
}

#[rustfmt::skip]
pub fn get_view_matrix() -> Vec<f32> {
  vec![
    7.713_025_5e-7, 0.0, 0.0, 0.0,
    0.0, 7.713_025_5e-7, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    -0.975_147_84, -4.825_174, 0.0, 1.0,
  ]
}

pub fn get_snapshot(base64: &str) -> String {
  [BASE64_PREFIX, base64]
    .join("")
    .replace('_', "/")
    .replace('-', "+")
}
