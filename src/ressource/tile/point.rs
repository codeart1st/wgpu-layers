use geo_types::Geometry::{MultiPoint, Point};
use log::info;
use mvt_reader::feature::Feature;

use crate::ressource::{material::MaterialType, BindGroupScope, RessourceManager};

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

  fn add_features(&mut self, features: &mut Vec<Feature>, ressource_manager: &RessourceManager) {
    for feature in features.iter() {
      match feature.get_geometry() {
        Point(point) => {
          let mut vertices = Vec::with_capacity(DIMENSIONS);
          vertices.push(point.x());
          vertices.push(point.y());
          self.vertex_buffer.append(&mut vertices);
        }
        MultiPoint(multi_point) => {
          let mut vertices = Vec::with_capacity(multi_point.0.len() * DIMENSIONS);
          for point in multi_point.iter() {
            vertices.push(point.x());
            vertices.push(point.y());
          }
          self.vertex_buffer.append(&mut vertices);
        }
        _ => {
          info!("Geometry type currently not supported");
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
