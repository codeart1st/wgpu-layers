use std::{mem::size_of, num::NonZeroU64, sync::Arc};

use log::info;
use mvt_reader::feature::Feature;

use super::{material::Material, view::View, BindGroupScope, RessourceManager};

mod fill;
mod line;
mod point;

const DIMENSIONS: usize = 2;

const TILE_SIZE: f32 = 4096.0;

#[derive(PartialEq, Eq, Clone)]
pub enum BucketType {
  Fill,
  Line,
  Point,
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct TileUniform {
  model_view_matrix: glam::Mat4,
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

  instance_wgpu_buffer: Option<wgpu::Buffer>,

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

        match self.get_bucket_type() {
          BucketType::Point => {
            render_pass.set_vertex_buffer(1, self.instance_wgpu_buffer.as_ref().unwrap().slice(..));
            let instance_end = (self.vertex_buffer.len() / DIMENSIONS) as _;
            render_pass.draw_indexed(0..end, 0, 0..instance_end);
          }
          _ => {
            render_pass.draw_indexed(0..end, 0, 0..1);
          }
        }
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

/// tile_transform * flip_tile_transform because of Y-axis swap
#[rustfmt::skip]
fn get_model_matrix(extent: [f32; 4], tile_size: f32) -> glam::Mat4 {
  let tile_transform = glam::Mat4::from_cols_array(&[
    (extent[2] - extent[0]) / tile_size, 0.0, 0.0, 0.0, // a11 a21 a31 a41
    0.0, (extent[2] - extent[0]) / tile_size, 0.0, 0.0, // a12 a22 a32 a42
    0.0, 0.0, 1.0, 0.0,                                 // a13 a23 a33 a43
    extent[0], extent[1], 0.0, 1.0,                     // a14 a24 a34 a44
  ]);
  let flip_tile_transform = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, tile_size, 0.0, 1.0,
  ]);
  tile_transform.mul_mat4(&flip_tile_transform)
}

fn get_transforms(
  view_matrix: glam::Mat4,
  extent: [f32; 4],
  half_width: f32,
  half_height: f32,
) -> TileUniform {
  let model_matrix = get_model_matrix(extent, TILE_SIZE);
  let model_view_matrix = view_matrix.mul_mat4(&model_matrix);

  let min_extent = model_view_matrix.mul_vec4(glam::Vec4::W);
  let max_extent =
    model_view_matrix.mul_vec4(glam::Vec4::from_slice(&[TILE_SIZE, TILE_SIZE, 0.0, 1.0]));

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

  fn add_features(&mut self, features: &mut Vec<Feature>, ressource_manager: &RessourceManager);
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
