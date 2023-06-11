use mvt_reader::feature::Feature;

use crate::ressource::{material::MaterialType, BindGroupScope, RessourceManager};

use super::{Bucket, BucketType, Tile, TileUniform};

impl<F> Bucket<F, { BucketType::Line }> for Tile {
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

    Self {
      material: ressource_manager.get_material(MaterialType::Line),
      bind_group,
      tile_uniform_buffer,
      vertex_wgpu_buffer: None,
      vertex_buffer: Vec::with_capacity(0),
      index_wgpu_buffer: None,
      index_buffer: Vec::with_capacity(0),
      instance_wgpu_buffer: None,
      extent,
      bucket_type: BucketType::Line,
    }
  }

  fn add_features(&mut self, _: &mut Vec<Feature>, _: &RessourceManager) {}
}
