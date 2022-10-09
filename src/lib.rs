use std::{
  cell::{Cell, RefCell},
  sync::Mutex,
};

#[macro_use]
extern crate lazy_static;

use geo_types::GeometryCollection;

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

pub struct Mapped {
  mapped: bool,
}

lazy_static! {
  pub static ref INSTANCE: Mutex<Instance> = Mutex::new(Instance::default());
  pub static ref MAPPED: Mutex<Mapped> = Mutex::new(Mapped { mapped: false });
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  impl super::renderer::ToSurface for web_sys::OffscreenCanvas {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface_from_offscreen_canvas(self)
    }
  }

  impl super::renderer::ToSurface for web_sys::HtmlCanvasElement {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface_from_canvas(self)
    }
  }

  #[wasm_bindgen(js_name = startWithOffscreenCanvas)]
  pub async fn start_with_offscreencanvas(canvas: web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    super::init(&canvas, (canvas.width(), canvas.height())).await;
  }

  #[wasm_bindgen(js_name = startWithCanvas)]
  pub async fn start_with_canvas(canvas: web_sys::HtmlCanvasElement) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    super::init(&canvas, (canvas.width(), canvas.height())).await;
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render(view_matrix: Vec<f32>, new_size: Vec<u32>) {
  if let Ok(instance) = INSTANCE.try_lock() {
    match &instance.renderer {
      Some(renderer) => {
        let mut borrowed_renderer = renderer.borrow_mut();
        let mut view_matrix_array = [[0.0; 4]; 4];
        for i in 0..4 {
          for j in 0..4 {
            view_matrix_array[i][j] = *view_matrix.get(i * 4 + j).expect("view matrix is wrong");
          }
        }
        borrowed_renderer.view.view_matrix = view_matrix_array;

        let current_size = instance.current_size.get();
        if current_size.0 != new_size[0] || current_size.1 != new_size[1] {
          instance.current_size.set((new_size[0], new_size[1]));
          borrowed_renderer.set_size(instance.current_size.get());
        }

        borrowed_renderer.render(&instance.buckets.borrow());
      }
      None => todo!(),
    }
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = addPbfTileData))]
pub async fn add_pbf_tile_data(pbf: Vec<u8>, tile_coord: Vec<u32>, extent: Vec<f32>) {
  let parser = parser::Parser::new(pbf).expect("parse error");

  // TODO: map tile_coord (key) to bucket (value)
  // TODO: later pass z index together with grid extent (min tile and max tile) to find out which buckets needs to be drawn

  let layer_index_option = parser
    .get_layer_names()
    .iter()
    .position(|layer_name| layer_name == "land");

  if let Ok(instance) = INSTANCE.try_lock() {
    match layer_index_option {
      Some(layer_index) => match parser.get_features(layer_index) {
        Some(mut parsed_features) => {
          if parsed_features.is_empty() {
            return;
          }
          match &instance.renderer {
            Some(renderer) => {
              let mut borrowed_renderer = renderer.borrow_mut();
              let mut bucket = borrowed_renderer.create_bucket();
              bucket.add_features(&mut parsed_features);
              bucket.set_extent(extent);

              if let Ok(mut mapped) = MAPPED.lock() {
                if !mapped.mapped {
                  mapped.mapped = true;
                  borrowed_renderer.compute().await;
                }
              }

              instance.buckets.borrow_mut().push(bucket);
            }
            None => (),
          }
        }
        None => (),
      },
      None => (),
    }
  }
}

pub async fn init<W: renderer::ToSurface>(window: &W, size: (u32, u32)) {
  let renderer = RefCell::new(renderer::Renderer::new(window, size).await);

  if let Ok(mut instance) = INSTANCE.lock() {
    instance.renderer = Some(renderer);
    instance.current_size = Cell::new(size);
  }
}
