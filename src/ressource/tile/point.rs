use geo_types::Geometry::*;
use log::info;

use crate::{
  feature::WithGeometry,
  ressource::{material::MaterialType, BindGroupScope, RessourceManager},
};

use super::{Bucket, BucketType, Tile, TileUniform};

const DIMENSIONS: usize = 2;

const RECT_VERTEX_BUFFER: [f32; 8] = [-0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5];
const RECT_INDICES_BUFFER: [u32; 6] = [0, 1, 2, 2, 3, 0];

impl<F> Bucket<F, { BucketType::Point }> for Tile {
  fn new(ressource_manager: &RessourceManager, extent: [f32; 4]) -> Self {
    let tile_uniform = TileUniform::default();
    let tile_uniform_buffer =
      ressource_manager.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[tile_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });
    let bind_group = ressource_manager.create_bind_group(
      &BindGroupScope::Model,
      &[wgpu::BindGroupEntry {
        binding: 0,
        resource: tile_uniform_buffer.as_entire_binding(),
      }],
    );

    let vertex_wgpu_buffer = Some(ressource_manager.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&RECT_VERTEX_BUFFER),
        usage: wgpu::BufferUsages::VERTEX,
      },
    ));

    let index_wgpu_buffer = Some(ressource_manager.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&RECT_INDICES_BUFFER),
        usage: wgpu::BufferUsages::INDEX,
      },
    ));

    Self {
      material: ressource_manager.get_material(MaterialType::Point),
      bind_group,
      tile_uniform_buffer,
      vertex_wgpu_buffer,
      vertex_buffer: Vec::with_capacity(0),
      index_wgpu_buffer,
      index_buffer: Vec::with_capacity(0),
      instance_wgpu_buffer: None,
      extent,
      bucket_type: BucketType::Point,
    }
  }

  fn add_features(&mut self, features: &mut Vec<F>, ressource_manager: &RessourceManager)
  where
    F: WithGeometry<geo_types::GeometryCollection<f32>>,
  {
    for feature in features.iter() {
      let geometry_collection = feature.get_geometry();
      for geometry in geometry_collection.iter() {
        match geometry {
          Polygon(polygon) => {
            let exterior = polygon.exterior();
            let interior = polygon.interiors();
            let mut vertex_count = exterior.0.len() - 1;
            let mut rings = Vec::with_capacity(1 + interior.len());
            rings.push(exterior);
            interior.iter().for_each(|r| {
              rings.push(r);
              // ignore last coordinate (closed ring)
              vertex_count += r.0.len() - 1;
            });
            let mut vertices = Vec::with_capacity(vertex_count * DIMENSIONS);
            for ring in rings.iter() {
              // ignore last coordinate (closed ring)
              let end = ring.0.len() - 1;
              let coordinate_slice = &ring.0[..end];
              for coord in coordinate_slice.iter() {
                vertices.push(coord.x);
                vertices.push(coord.y);
              }
            }
            self.vertex_buffer.append(&mut vertices);
          }
          _ => {
            info!("Geometry type currently not supported");
          }
        }
      }
    }

    self.instance_wgpu_buffer = Some(ressource_manager.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&self.vertex_buffer),
        usage: wgpu::BufferUsages::VERTEX,
      },
    ));
  }
}
