#![feature(thread_local)]
#![allow(clippy::await_holding_refcell_ref)]
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
  timeout(500).await; // wait for compute shader
  wgpu_layers::render(get_view_matrix(), vec![CANVAS_SIZE.0, CANVAS_SIZE.1]);
  timeout(500).await; // wait to render

  // assert
  let image_data = get_canvas_image_data(canvas).await;
  let image = pdqhash::image::load_from_memory(&image_data[..]).unwrap();
  let (hash, _) = pdqhash::generate_pdq_full_size(&image);

  assert_eq!(
    [
      110, 147, 181, 199, 72, 220, 37, 190, 238, 220, 47, 28, 184, 128, 194, 108, 153, 177, 194,
      237, 155, 33, 200, 131, 34, 47, 24, 147, 102, 238, 119, 176
    ],
    hash
  );
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
  timeout(500).await; // wait to render

  // assert
  let image_data = get_canvas_image_data(canvas).await;
  let image = pdqhash::image::load_from_memory(&image_data[..]).unwrap();
  let (hash, _) = pdqhash::generate_pdq_full_size(&image);

  assert_eq!(
    [
      171, 170, 84, 117, 171, 170, 84, 81, 171, 138, 84, 84, 171, 170, 171, 138, 171, 138, 84, 81,
      85, 255, 84, 113, 171, 174, 84, 84, 171, 170, 84, 85
    ],
    hash
  );
}
