#![allow(incomplete_features)]
#![feature(adt_const_params)]

use feature::Feature;
use geo_types::GeometryCollection;
use log::{error, info};
use ressource::tile::{Bucket, BucketType, Tile};
use std::cell::{Cell, RefCell};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod feature;
mod parser;
pub mod renderer;
mod ressource;
mod tessellation;

#[derive(Default)]
pub struct Instance {
  renderer: RefCell<Option<renderer::Renderer>>,
  tiles: RefCell<Vec<Tile>>,
  current_size: Cell<(u32, u32)>,
}

struct Message {
  parsed_features: Option<Vec<Feature<GeometryCollection<f32>>>>,
  extent: Vec<f32>,
}

thread_local! {
  static TILE_PARSER_QUEUE: (Sender<Message>, Receiver<Message>) = channel();
  static INSTANCE: Instance = Instance::default();
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
    match console_log::init_with_level(log::Level::Info) {
      Ok(()) => (),
      Err(err) => log::error!("{}", err),
    }

    super::init(&canvas, (canvas.width(), canvas.height())).await;
  }

  #[wasm_bindgen(js_name = startWithCanvas)]
  pub async fn start_with_canvas(canvas: &web_sys::HtmlCanvasElement) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    match console_log::init_with_level(log::Level::Info) {
      Ok(()) => (),
      Err(err) => log::error!("{}", err),
    }

    super::init(canvas, (canvas.width(), canvas.height())).await;
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn render(view_matrix: Vec<f32>, new_size: Vec<u32>) {
  process_tile_parser_queue();

  INSTANCE.with(|instance| {
    let mut borrowed_renderer = instance.renderer.borrow_mut();
    let renderer = borrowed_renderer.as_mut().unwrap();
    let mut view_matrix_array = [[0.0; 4]; 4];

    #[allow(clippy::needless_range_loop)]
    for i in 0..4 {
      for j in 0..4 {
        view_matrix_array[i][j] = *view_matrix.get(i * 4 + j).expect("view matrix is wrong");
      }
    }
    renderer.view.set_view_matrix(view_matrix_array);

    let current_size = instance.current_size.get();
    if current_size.0 != new_size[0] || current_size.1 != new_size[1] {
      instance.current_size.set((new_size[0], new_size[1]));
      renderer.set_size(instance.current_size.get());
    }

    renderer.render(&instance.tiles.borrow());
  });
}

fn process_tile_parser_queue() {
  TILE_PARSER_QUEUE.with(|(_, receiver)| loop {
    match receiver.try_recv() {
      Ok(msg) => {
        if let Some(mut parsed_features) = msg.parsed_features {
          if parsed_features.is_empty() {
            return;
          }

          INSTANCE.with(|instance| {
            /*{
              let renderer = instance.renderer.as_ref().unwrap().clone();
              wasm_bindgen_futures::spawn_local(async move {
                if let Ok(mut renderer) = renderer.try_lock() {
                  renderer.compute().await;
                };
              });
            }*/
            let borrowed_renderer = instance.renderer.borrow();
            let renderer = borrowed_renderer.as_ref().unwrap();
            let mut tile = renderer.create_tile::<Feature<geo_types::GeometryCollection<f32>>>(
              BucketType::Fill,
              msg.extent.try_into().unwrap(),
            );

            match tile.get_bucket_type() {
              BucketType::Fill => {
                <Tile as Bucket<
                  Feature<geo_types::GeometryCollection<f32>>,
                  { BucketType::Fill },
                >>::add_features(
                  &mut tile, &mut parsed_features, &renderer.ressource_manager
                );
              }
              BucketType::Line => {
                <Tile as Bucket<
                  Feature<geo_types::GeometryCollection<f32>>,
                  { BucketType::Line },
                >>::add_features(
                  &mut tile, &mut parsed_features, &renderer.ressource_manager
                );
              }
            }

            instance.tiles.borrow_mut().push(tile);
          });
        }
      }
      Err(err) => match err {
        Disconnected => {
          error!("{}", err.to_string());
          break;
        }
        Empty => {
          info!("no more tiles left");
          break;
        }
      },
    }
  });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = addPbfTileData))]
pub async fn add_pbf_tile_data(pbf: Vec<u8>, _tile_coord: Vec<u32>, extent: Vec<f32>) {
  TILE_PARSER_QUEUE.with(|(sender, _)| {
    let sender = sender.clone();
    rayon::spawn(move || {
      let parser = parser::Parser::new(pbf).expect("parse error");

      let layer_index_option = parser
        .get_layer_names()
        .iter()
        .position(|layer_name| layer_name == "land");

      if let Some(layer_index) = layer_index_option {
        sender
          .send(Message {
            parsed_features: parser.get_features(layer_index),
            extent,
          })
          .unwrap();
        info!("parsed");
      }
    });
  });
}

pub async fn init<W: renderer::ToSurface>(window: &W, size: (u32, u32)) {
  let renderer = renderer::Renderer::new(window, size).await;

  INSTANCE.with(|instance| {
    instance.renderer.replace(Some(renderer));
    instance.tiles.replace(Vec::new());
    instance.current_size.replace(size);
  });
}
