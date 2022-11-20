use std::{mem::size_of, num::NonZeroU64, sync::Arc};

use log::info;

use crate::feature::WithGeometry;

use super::{material::Material, view::View, BindGroupScope, RessourceManager};

mod fill;
mod line;

const TILE_SIZE: f32 = 4096.0;

#[derive(PartialEq, Eq, Clone)]
pub enum BucketType {
  Fill,
  Line,
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct TileUniform {
  model_view_matrix: [[f32; 4]; 4],
  clipping_rect: [f32; 4],
}

pub struct TileManager;

pub struct Tile {
  material: Arc<Material>,

  bind_group: wgpu::BindGroup,

  tile_uniform_buffer: wgpu::Buffer,

  /// vertex buffer
  vertex_wgpu_buffer: Option<wgpu::Buffer>,

  vertex_buffer: Vec<f32>,

  /// index buffer
  index_wgpu_buffer: Option<wgpu::Buffer>,

  index_buffer: Vec<u32>,

  extent: [f32; 4],

  bucket_type: BucketType,
}

impl Tile {
  pub fn add_buffers(&mut self, vertices_buffer: wgpu::Buffer, indices_buffer: wgpu::Buffer) {
    self.vertex_wgpu_buffer = Some(vertices_buffer);
    self.index_wgpu_buffer = Some(indices_buffer);
  }

  pub fn render<'frame>(
    &'frame self,
    render_pass: &mut wgpu::RenderPass<'frame>,
    queue: &wgpu::Queue,
    view: &'frame View,
  ) {
    match (
      self.vertex_wgpu_buffer.as_ref(),
      self.index_wgpu_buffer.as_ref(),
    ) {
      (Some(vertex_buffer), Some(index_buffer)) => {
        self.material.set(render_pass);
        render_pass.set_bind_group(BindGroupScope::Model as u32, &self.bind_group, &[]);

        let (half_width, half_height) = view.get_half_size();
        let tile_uniform =
          get_transforms(view.get_view_matrix(), self.extent, half_width, half_height);

        queue.write_buffer(
          &self.tile_uniform_buffer,
          0,
          bytemuck::cast_slice(&[tile_uniform]),
        );

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        let end = index_buffer.size() as u32 / std::mem::size_of::<u32>() as u32;
        render_pass.draw_indexed(0..end, 0, 0..1);
      }
      _ => {
        info!("No features found")
      }
    }
  }

  pub fn get_bucket_type(&self) -> BucketType {
    self.bucket_type.clone()
  }
}

#[allow(clippy::needless_range_loop)]
fn mat4x4_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
  let mut c = [[0.0; 4]; 4];

  for i in 0..4 {
    for j in 0..4 {
      for k in 0..4 {
        c[i][j] += a[k][j] * b[i][k];
      }
    }
  }
  c
}

#[allow(clippy::needless_range_loop)]
fn mat4x4_mul_vec4(a: &[[f32; 4]; 4], b: &[f32; 4]) -> [f32; 4] {
  let mut result = [0.0; 4];
  for i in 0..4 {
    for j in 0..4 {
      result[i] += b[j] * a[j][i];
    }
  }
  result
}

/// tile_transform * flip_tile_transform because of Y-axis swap
#[rustfmt::skip]
fn get_model_matrix(extent: [f32; 4], tile_size: f32) -> [[f32; 4]; 4] {
  let tile_transform = [ // column-major order
    [(extent[2] - extent[0]) / tile_size, 0.0, 0.0, 0.0], // a11 a21 a31 a41
    [0.0, (extent[2] - extent[0]) / tile_size, 0.0, 0.0], // a12 a22 a32 a42
    [0.0, 0.0, 1.0, 0.0                                ], // a13 a23 a33 a43
    [extent[0], extent[1], 0.0, 1.0                    ], // a14 a24 a34 a44
  ];
  let flip_tile_transform = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, -1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, tile_size, 0.0, 1.0],
  ];
  mat4x4_mul(&tile_transform, &flip_tile_transform)
}

fn get_transforms(
  view_matrix: [[f32; 4]; 4],
  extent: [f32; 4],
  half_width: f32,
  half_height: f32,
) -> TileUniform {
  let model_matrix = get_model_matrix(extent, TILE_SIZE);
  let model_view_matrix = mat4x4_mul(&view_matrix, &model_matrix);

  let min_extent = mat4x4_mul_vec4(&model_view_matrix, &[0.0, 0.0, 0.0, 1.0]);
  let max_extent = mat4x4_mul_vec4(&model_view_matrix, &[TILE_SIZE, TILE_SIZE, 0.0, 1.0]);

  // convert from view to screen coordinates (pixels)
  let clipping_rect = [
    (min_extent[0] + 1.0) * half_width,
    (-min_extent[1] + 1.0) * half_height,
    (max_extent[0] + 1.0) * half_width,
    (-max_extent[1] + 1.0) * half_height,
  ];

  TileUniform {
    model_view_matrix,
    clipping_rect,
  }
}

pub trait Bucket<F, const T: BucketType>
where
  Self: Sized,
{
  fn new(ressource_manager: &RessourceManager, extent: [f32; 4]) -> Self;

  fn add_features(&mut self, features: &mut Vec<F>, ressource_manager: &RessourceManager)
  where
    F: WithGeometry<geo_types::GeometryCollection<f32>>;
}

impl TileManager {
  pub fn new(ressource_manager: &mut RessourceManager) -> Self {
    ressource_manager.register_bind_group_layout(
      BindGroupScope::Model,
      &wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
              NonZeroU64::new(size_of::<TileUniform>().try_into().unwrap()).unwrap(),
            ),
          },
          count: None,
        }],
      },
    );

    Self
  }
}
