use std::cell::{Cell, RefCell};

use geo_types::GeometryCollection;
use log::info;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod bucket;
mod parser;
pub mod renderer;
mod view;

type GeometryFeature = bucket::feature::Feature<GeometryCollection<f32>>;
type BucketVec = Vec<bucket::Bucket<GeometryFeature>>;

#[derive(Default)]
pub struct Instance {
  renderer: Option<RefCell<renderer::Renderer>>,
  buckets: RefCell<BucketVec>,
  current_size: Cell<(u32, u32)>,
}

thread_local! {
  static INSTANCE: RefCell<Instance> = RefCell::new(Instance::default());
}

#[cfg(target_arch = "wasm32")]
mod wasm {
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  impl super::renderer::ToSurface for web_sys::OffscreenCanvas {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface_from_offscreen_canvas(self)
    }
  }

  #[wasm_bindgen]
  pub async fn start(canvas: web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    super::init(&canvas, (canvas.width(), canvas.height())).await;
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render(view_matrix: Vec<f32>, new_size: Vec<u32>) {
  INSTANCE.with(|instance| {
    let borrowed_instance = instance.borrow();
    match &borrowed_instance.renderer {
      Some(renderer) => {
        let mut borrowed_renderer = renderer.borrow_mut();
        let mut view_matrix_array = [[0.0; 4]; 4];
        for i in 0..4 {
          for j in 0..4 {
            view_matrix_array[i][j] = *view_matrix.get(i * 4 + j).expect("view matrix is wrong");
          }
        }
        borrowed_renderer.view.view_matrix = view_matrix_array;

        let current_size = borrowed_instance.current_size.get();
        if current_size.0 != new_size[0] || current_size.1 != new_size[1] {
          borrowed_instance
            .current_size
            .set((new_size[0], new_size[1]));
          borrowed_renderer.set_size(borrowed_instance.current_size.get());
        }

        borrowed_renderer.render(&borrowed_instance.buckets.borrow());
      }
      None => (),
    }
  });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = addPbfTileData))]
pub fn add_pbf_tile_data(pbf: Vec<u8>, tile_coord: Vec<u32>, extent: Vec<f32>) {
  let parser = parser::Parser::new(pbf).expect("parse error");

  let layer_index_option = parser
    .get_layer_names()
    .iter()
    .position(|layer_name| layer_name == "land");

  match layer_index_option {
    Some(layer_index) => match parser.get_features(layer_index) {
      Some(mut parsed_features) => {
        if parsed_features.is_empty() {
          return;
        }
        INSTANCE.with(|instance| {
          let borrowed_instance = instance.borrow();

          match &borrowed_instance.renderer {
            Some(renderer) => {
              let borrowed_renderer = renderer.borrow();
              let mut bucket = borrowed_renderer.create_bucket();
              bucket.add_features(&mut parsed_features);
              bucket.set_extent(extent);

              borrowed_instance.buckets.borrow_mut().push(bucket);
            }
            None => (),
          }
        });
      }
      None => (),
    },
    None => (),
  }
}

pub async fn init<W: renderer::ToSurface>(window: &W, size: (u32, u32)) {
  let renderer = RefCell::new(renderer::Renderer::new(window, size).await);

  INSTANCE.with(|instance| {
    instance.borrow_mut().renderer = Some(renderer);
    instance.borrow_mut().current_size = Cell::new(size);
  })
}
