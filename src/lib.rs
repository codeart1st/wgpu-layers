use std::cell::{Cell, RefCell};

use geo_types::GeometryCollection;
use log::info;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod bucket;
mod parser;
pub mod renderer;
mod view;

type FeatureVec = Vec<bucket::feature::Feature<GeometryCollection<f32>>>;

#[derive(Default)]
pub struct Instance {
  renderer: Option<RefCell<renderer::Renderer>>,
  features: RefCell<FeatureVec>,
  extent: Cell<[f32; 4]>,
  current_size: Cell<(u32, u32)>,
}

thread_local! {
  static INSTANCE: RefCell<Instance> = RefCell::new(Instance::default());
}

#[cfg(target_arch = "wasm32")]
mod wasm {
  use geo_types::{polygon, GeometryCollection};
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  impl super::renderer::ToSurface for web_sys::OffscreenCanvas {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface_from_offscreen_canvas(self)
    }
  }

  #[wasm_bindgen]
  pub async fn start(canvas: web_sys::OffscreenCanvas, vector_tile: Vec<u8>) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    let parser = super::parser::Parser::new(vector_tile).expect("parse error");

    log::info!("{:?}", parser.get_layer_names());

    let test_geometry: geo_types::Geometry<f32> = (polygon!(
      exterior: [
        (x: -3862117.0, y: 9809176.0),
        (x: 8579526.0, y: 9915819.0),
        (x: 901254.0, y: 1597691.0),
        (x: -3862117.0, y: 9809176.0)
      ],
      interiors: []
    ))
    .try_into()
    .expect("Can't convert polygon to geometry");

    let mut features = vec![super::bucket::feature::Feature {
      geometry: GeometryCollection(vec![test_geometry]),
      properties: None,
    }];

    match parser.get_features(2) {
      Some(mut parsed_features) => {
        features.append(&mut parsed_features);
      }
      None => (),
    }

    super::init(&canvas, (canvas.width(), canvas.height()), features).await;
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render(view_matrix: Vec<f32>, new_size: Vec<u32>) {
  INSTANCE.with(|instance| {
    let borrowed_instance = instance.borrow();
    match &borrowed_instance.renderer {
      Some(renderer) => {
        let mut borrowed_renderer = renderer.borrow_mut();
        borrowed_renderer.view.view_matrix = view_matrix.try_into().expect("View matrix is wrong");

        let current_size = borrowed_instance.current_size.get();
        if current_size.0 != new_size[0] || current_size.1 != new_size[1] {
          borrowed_instance
            .current_size
            .set((new_size[0], new_size[1]));
          borrowed_renderer.set_size(borrowed_instance.current_size.get());
        }

        let mut bucket = borrowed_renderer.create_bucket();
        let features = borrowed_instance.features.borrow();
        bucket.add_features(&features);
        bucket.set_extent(borrowed_instance.extent.get());

        borrowed_renderer.render(vec![bucket]);
      }
      None => (),
    }
  });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = addPbfTileData))]
pub fn add_pbf_tile_data(pbf: Vec<u8>, tile_coord: Vec<u32>, extent: Vec<f32>) {
  INSTANCE.with(|instance| {
    let borrowed_instance = instance.borrow();

    borrowed_instance
      .extent
      .set([extent[0], extent[1], extent[2], extent[3]]);

    if borrowed_instance.features.borrow().len() > 1 {
      return;
    }

    let parser = parser::Parser::new(pbf).expect("parse error");

    log::info!("{:?} {:?}", tile_coord, parser.get_layer_names());

    match parser.get_features(2) {
      Some(mut parsed_features) => {
        borrowed_instance
          .features
          .borrow_mut()
          .append(&mut parsed_features);
      }
      None => (),
    }
  });
}

pub async fn init<W: renderer::ToSurface>(
  window: &W,
  size: (u32, u32),
  features: Vec<bucket::feature::Feature<GeometryCollection<f32>>>,
) {
  let renderer = RefCell::new(renderer::Renderer::new(window, size).await);

  info!("renderer initialized");

  INSTANCE.with(|instance| {
    instance.borrow_mut().renderer = Some(renderer);
    instance.borrow_mut().features = RefCell::new(features);
    instance.borrow_mut().current_size = Cell::new(size);
  })
}
