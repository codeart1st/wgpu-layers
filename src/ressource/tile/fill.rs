use geo_types::Geometry::{MultiPolygon, Polygon};
use log::{error, info};
use mvt_reader::feature::Feature;

use crate::ressource::{BindGroupScope, RessourceManager, material::MaterialType};

use super::{Bucket, BucketType, Tile, TileUniform};

const DIMENSIONS: usize = 2;

impl Tile {
  fn add_polygon(&mut self, polygon: &geo_types::Polygon<f32>) {
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
    let mut hole_indices = Vec::new();
    for (i, ring) in rings.iter().enumerate() {
      // ignore last coordinate (closed ring)
      let end = ring.0.len() - 1;
      let coordinate_slice = &ring.0[..end];
      for coord in coordinate_slice.iter() {
        vertices.push(coord.x);
        vertices.push(coord.y);
      }
      if i < rings.len() - 1 {
        hole_indices.push(vertices.len() / DIMENSIONS)
      }
    }

    let earcut_result = earcutr::earcut(&vertices, &hole_indices, DIMENSIONS);
    match earcut_result {
      Ok(indices) => {
        let offset = (self.vertex_buffer.len() / DIMENSIONS) as u32;
        self.vertex_buffer.append(&mut vertices);
        self
          .index_buffer
          .append(&mut indices.iter().map(|i| (*i as u32) + offset).collect());
      }
      Err(_) => {
        error!("earcut parsing error");
      }
    }
  }
}

impl<F> Bucket<F, { BucketType::Fill }> for Tile {
  fn new(ressource_manager: &RessourceManager, extent: [f32; 4]) -> Self {
    let tile_uniform = TileUniform::default();
    let tile_uniform_buffer =
      ressource_manager.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[tile_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });
    let bind_group =
      ressource_manager.create_bind_group(&BindGroupScope::Model, &[wgpu::BindGroupEntry {
        binding: 0,
        resource: tile_uniform_buffer.as_entire_binding(),
      }]);

    Self {
      material: ressource_manager.get_material(MaterialType::Fill),
      bind_group,
      tile_uniform_buffer,
      vertex_wgpu_buffer: None,
      vertex_buffer: Vec::with_capacity(0),
      index_wgpu_buffer: None,
      index_buffer: Vec::with_capacity(0),
      instance_wgpu_buffer: None,
      extent,
      bucket_type: BucketType::Fill,
    }
  }

  fn add_features(&mut self, features: &mut Vec<Feature>, ressource_manager: &RessourceManager) {
    for feature in features.iter() {
      match feature.get_geometry() {
        Polygon(polygon) => {
          self.add_polygon(polygon);
        }
        MultiPolygon(multi_polygon) => {
          for polygon in multi_polygon.iter() {
            self.add_polygon(polygon);
          }
        }
        _ => {
          info!("Geometry type currently not supported");
        }
      }
    }

    self.vertex_wgpu_buffer = Some(ressource_manager.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&self.vertex_buffer),
        usage: wgpu::BufferUsages::VERTEX,
      },
    ));

    self.index_wgpu_buffer = Some(ressource_manager.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&self.index_buffer),
        usage: wgpu::BufferUsages::INDEX,
      },
    ));
  }
}
