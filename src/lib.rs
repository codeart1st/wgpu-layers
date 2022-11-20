#![allow(incomplete_features)]
#![feature(adt_const_params)]

use feature::Feature;
use geo_types::GeometryCollection;
use log::{error, info};
use ressource::tile::{Bucket, BucketType, Tile};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
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
  renderer: Rc<RefCell<Option<renderer::Renderer>>>,
  tiles: Rc<RefCell<Vec<Tile>>>,
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

const DIMENSIONS: usize = 2;

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
    let mut reference = instance.renderer.borrow_mut();
    let renderer = reference.as_mut().unwrap();
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

fn triangulate<F>(features: &[F]) -> (Vec<f32>, Vec<u32>)
where
  F: feature::WithGeometry<geo_types::GeometryCollection<f32>>,
{
  let mut all_vertices = vec![];
  let mut all_indices = vec![];
  let mut offset = 0;
  for feature in features.iter() {
    let geometry_collection = feature.get_geometry();
    for geometry in geometry_collection.iter() {
      match geometry {
        geo_types::Geometry::Polygon(polygon) => {
          let exterior = polygon.exterior();
          let interior: &[geo_types::LineString<f32>] = &[]; //TODO: polygon.interiors();
          let mut vertex_count = exterior.0.len() - 1;
          let mut rings = Vec::with_capacity(1 + interior.len());
          rings.push(exterior);
          interior.iter().for_each(|r| {
            rings.push(r);
            // ignore last coordinate (closed ring)
            vertex_count += r.0.len() - 1;
          });
          let mut vertices = Vec::with_capacity(vertex_count * DIMENSIONS);
          let mut hole_indices = Vec::new();
          let mut indices = Vec::new();
          for (i, ring) in rings.iter().enumerate() {
            // ignore last coordinate (closed ring)
            let end = ring.0.len() - 1;
            let coordinate_slice = &ring.0[..end];
            for (i, coord) in coordinate_slice.iter().enumerate() {
              vertices.push(coord.x);
              vertices.push(coord.y);
              indices.push(i as u32 + offset);
            }

            indices.push(offset as u32); // close ring
            indices.push(offset as u32); // separate linestring from the next one
            offset += coordinate_slice.len() as u32;

            if i < rings.len() - 1 {
              hole_indices.push(vertices.len())
            }
          }
          all_vertices.append(&mut vertices);
          all_indices.append(&mut indices);
        }
        _ => {
          info!("Geometry type currently not supported");
        }
      }
    }
  }
  (all_vertices, all_indices)
}

fn process_tile_parser_queue() {
  TILE_PARSER_QUEUE.with(|(_, receiver)| loop {
    match receiver.try_recv() {
      Ok(msg) => {
        if let Some(mut parsed_features) = msg.parsed_features {
          if parsed_features.is_empty() {
            return;
          }

          let (vertices, indices) = triangulate(&parsed_features[..]);

          INSTANCE.with(|instance| {
            let extent: [f32; 4] = msg.extent.try_into().unwrap();

            #[cfg(target_arch = "wasm32")]
            {
              let clone = instance.renderer.clone();
              let tiles = instance.tiles.clone();

              #[allow(clippy::await_holding_refcell_ref)]
              wasm_bindgen_futures::spawn_local(async move {
                let mut reference = clone.borrow_mut();
                let renderer = reference.as_mut().unwrap();
                let (vertices_buffer, indices_buffer) =
                  renderer.compute(&vertices[..], &indices[..]).await;

                let mut tile = renderer.create_tile::<Feature<geo_types::GeometryCollection<f32>>>(
                  BucketType::Line,
                  extent,
                );
                tile.add_buffers(vertices_buffer, indices_buffer);
                tiles.borrow_mut().push(tile);
              });
            }
            let mut reference = instance.renderer.try_borrow_mut().unwrap();
            let renderer = reference.as_mut().unwrap();
            let mut tile = renderer
              .create_tile::<Feature<geo_types::GeometryCollection<f32>>>(BucketType::Fill, extent);

            match tile.get_bucket_type() {
              BucketType::Fill => {
                <Tile as Bucket<
                  Feature<geo_types::GeometryCollection<f32>>,
                  { BucketType::Fill },
                >>::add_features(
                  &mut tile, &mut parsed_features, &renderer.ressource_manager
                );
              }
              BucketType::Line => {}
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
