#![feature(thread_local)]
#![cfg(target_arch = "wasm32")]

mod utils;

use wasm_bindgen_test::*;

use utils::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn osm_pbf() {
  initialize();

  // arrange
  let canvas_ref = CANVAS.borrow();
  let canvas = canvas_ref.as_ref().unwrap();
  wgpu_layers::wasm::start_with_canvas(canvas).await;
  wgpu_layers::add_pbf_tile_data(
    include_bytes!("pbf/osm_4_8_5.pbf").to_vec(),
    vec![4, 8, 5],
    vec![0.0, 5009377.085697312, 2_504_688.5, 7_514_065.5],
  )
  .await;

  // act
  wgpu_layers::render(get_view_matrix(), vec![CANVAS_SIZE.0, CANVAS_SIZE.1]);

  // assert
  let result = canvas_as_data_url(canvas).await;
  let expect = get_snapshot(include_base64::include_base64!(
    "tests/snapshots/render_test_osm_pbf.png"
  ));
  assert_eq!(result.as_string().unwrap(), expect);
}

#[wasm_bindgen_test]
async fn empty() {
  initialize();

  // arrange
  let canvas_ref = CANVAS.borrow();
  let canvas = canvas_ref.as_ref().unwrap();
  wgpu_layers::wasm::start_with_canvas(canvas).await;

  // act
  wgpu_layers::render(get_view_matrix(), vec![CANVAS_SIZE.0, CANVAS_SIZE.1]);

  // assert
  let result = canvas_as_data_url(canvas).await;
  let expect = get_snapshot(include_base64::include_base64!(
    "tests/snapshots/render_test_empty.png"
  ));
  assert_eq!(result.as_string().unwrap(), expect);
}
