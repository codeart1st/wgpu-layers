#![allow(incomplete_features)]
#![feature(adt_const_params)]

use log::{error, info};
use mvt_reader::feature::Feature;
use ressource::tile::{Bucket, BucketType, Tile};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use std::sync::mpsc::{channel, Receiver, Sender};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use geo_types::Geometry::{LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};

pub mod renderer;
mod ressource;
mod tessellation;

#[derive(Default)]
pub struct Instance {
  renderer: Rc<RefCell<Option<renderer::Renderer>>>,
  tiles: Rc<RefCell<Vec<Tile>>>,
  current_size: Cell<(u32, u32)>,
}

struct Message {
  parsed_features: Vec<Feature>,
  extent: Vec<f32>,
}

thread_local! {
  static TILE_PARSER_QUEUE: (Sender<Message>, Receiver<Message>) = channel();
  static INSTANCE: Instance = Instance::default();
}

#[cfg(target_arch = "wasm32")]
const DIMENSIONS: usize = 2;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  impl super::renderer::ToSurface for web_sys::OffscreenCanvas {
    fn create_surface(
      &self,
      instance: &wgpu::Instance,
    ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
      instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(self.clone()))
    }
  }

  impl super::renderer::ToSurface for web_sys::HtmlCanvasElement {
    fn create_surface(
      &self,
      instance: &wgpu::Instance,
    ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
      instance.create_surface(wgpu::SurfaceTarget::Canvas(self.clone()))
    }
  }

  #[wasm_bindgen(js_name = startWithOffscreenCanvas)]
  pub async fn start_with_offscreencanvas(canvas: &web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    match console_log::init_with_level(log::Level::Info) {
      Ok(()) => (),
      Err(err) => log::error!("{}", err),
    }

    super::init(canvas, (canvas.width(), canvas.height())).await;
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
    let mut reference = instance.renderer.borrow_mut();
    let renderer = reference.as_mut().unwrap();

    renderer
      .view
      .set_view_matrix(glam::Mat4::from_cols_slice(&view_matrix[..]));

    let current_size = instance.current_size.get();
    if current_size.0 != new_size[0] || current_size.1 != new_size[1] {
      instance.current_size.set((new_size[0], new_size[1]));
      renderer.set_size(instance.current_size.get());
    }

    renderer.render(&instance.tiles.borrow());
  });
}

#[cfg(target_arch = "wasm32")]
fn get_buffers(features: &[Feature]) -> (Vec<f32>, Vec<u32>) {
  let mut all_vertices = vec![];
  let mut all_indices = vec![];

  for feature in features.iter() {
    match feature.get_geometry() {
      LineString(line) => {
        let mut vertices = Vec::with_capacity(line.0.len() * DIMENSIONS);
        let mut indices = Vec::with_capacity(line.0.len());
        let offset = (all_vertices.len() / DIMENSIONS) as u32;

        for (i, coord) in line.0.iter().enumerate() {
          vertices.push(coord.x);
          vertices.push(coord.y);
          indices.push(i as u32 + offset);
        }

        if let Some(last) = indices.last() {
          indices.push(*last); // separate linestring from the next one
        }

        all_vertices.append(&mut vertices);
        all_indices.append(&mut indices);
      }
      MultiLineString(multi_line) => {
        for line in multi_line.0.iter() {
          let mut vertices = Vec::with_capacity(line.0.len() * DIMENSIONS);
          let mut indices = Vec::with_capacity(line.0.len());
          let offset = (all_vertices.len() / DIMENSIONS) as u32;
          for (i, coord) in line.0.iter().enumerate() {
            vertices.push(coord.x);
            vertices.push(coord.y);
            indices.push(i as u32 + offset);
          }

          if let Some(last) = indices.last() {
            indices.push(*last); // separate linestring from the next one
          }

          all_vertices.append(&mut vertices);
          all_indices.append(&mut indices);
        }
      }
      _ => {
        log::info!("Geometry type currently not supported");
      }
    }
  }
  (all_vertices, all_indices)
}

fn process_tile_parser_queue() {
  TILE_PARSER_QUEUE.with(|(_, receiver)| loop {
    match receiver.try_recv() {
      Ok(msg) => {
        let mut parsed_features = msg.parsed_features;
        if parsed_features.is_empty() {
          return;
        }

        INSTANCE.with(|instance| {
          let extent: [f32; 4] = msg.extent.try_into().unwrap();
          let mut reference = instance.renderer.try_borrow_mut().unwrap();
          let renderer = reference.as_mut().unwrap();

          if let Some(feature) = parsed_features.get(0) {
            match feature.get_geometry() {
              &Point(_) | &MultiPoint(_) => {
                let mut tile = renderer.create_tile::<Feature>(BucketType::Point, extent);

                if tile.get_bucket_type() == BucketType::Point {
                  <Tile as Bucket<Feature, { BucketType::Point }>>::add_features(
                    &mut tile,
                    &mut parsed_features,
                    &renderer.ressource_manager,
                  );
                }

                instance.tiles.borrow_mut().push(tile);
              }
              &LineString(_) | &MultiLineString(_) => {
                #[cfg(target_arch = "wasm32")]
                {
                  let (vertices, indices) = get_buffers(&parsed_features[..]);

                  let clone = instance.renderer.clone();
                  let tiles = instance.tiles.clone();

                  #[allow(clippy::await_holding_refcell_ref)]
                  wasm_bindgen_futures::spawn_local(async move {
                    let mut reference = clone.borrow_mut();
                    let renderer = reference.as_mut().unwrap();
                    let (vertices_buffer, indices_buffer) =
                      renderer.compute(&vertices[..], &indices[..]).await;

                    let mut tile = renderer.create_tile::<Feature>(BucketType::Line, extent);
                    tile.add_buffers(vertices_buffer, indices_buffer);
                    tiles.borrow_mut().push(tile);
                  });
                }
              }
              &Polygon(_) | &MultiPolygon(_) => {
                let mut tile = renderer.create_tile::<Feature>(BucketType::Fill, extent);

                if tile.get_bucket_type() == BucketType::Fill {
                  <Tile as Bucket<Feature, { BucketType::Fill }>>::add_features(
                    &mut tile,
                    &mut parsed_features,
                    &renderer.ressource_manager,
                  );
                }

                instance.tiles.borrow_mut().push(tile);
              }
              _ => (),
            }
          }
        });
      }
      Err(err) => match err {
        Disconnected => {
          error!("{}", err.to_string());
          break;
        }
        Empty => {
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

    let parse = move || {
      let reader = mvt_reader::Reader::new(pbf).expect("parse error");
      let layer_names = reader.get_layer_names().unwrap();

      for (i, _) in layer_names.iter().enumerate() {
        sender
          .send(Message {
            parsed_features: reader.get_features(i).unwrap(),
            extent: extent.clone(),
          })
          .unwrap();
      }
    };

    #[cfg(not(feature = "multithreaded"))]
    parse();

    #[cfg(feature = "multithreaded")]
    rayon::spawn(parse);
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
