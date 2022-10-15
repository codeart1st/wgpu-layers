use std::sync::Arc;

use log::info;

use crate::view::View;

pub mod feature;
pub mod fill;
pub mod line;
pub mod line_tessellation;

const TILE_SIZE: f32 = 4096.0;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct Transforms {
  /// inverse view matrix
  view_matrix: [[f32; 4]; 4],

  /// model matrix
  model_matrix: [[f32; 4]; 4],

  /// inverse view matrix multiplied by model matrix
  model_view_matrix: [[f32; 4]; 4],

  /// tile clipping rect
  clipping_rect: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct Style {
  /// fill color
  fill_color: [f32; 4],

  /// stroke color
  stroke_color: [f32; 4],

  /// stroke width
  stroke_width: f32,
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

fn get_transforms(
  view_matrix: [[f32; 4]; 4],
  extent: [f32; 4],
  half_width: f32,
  half_height: f32,
) -> [Transforms; 1] {
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

  [Transforms {
    view_matrix,
    model_matrix,
    model_view_matrix,
    clipping_rect,
  }]
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BucketType {
  Fill,
  Line,
}

#[derive(Debug)]
pub struct Bucket<F> {
  /// wgpu device
  device: Arc<wgpu::Device>,

  /// wgpu pipeline
  pipeline: wgpu::RenderPipeline,

  /// map features
  features: Vec<F>,

  /// tile extent
  extent: [f32; 4],

  /// wgpu bind group
  bind_group: wgpu::BindGroup,

  /// vertex buffer
  vertex_wgpu_buffer: Option<wgpu::Buffer>,

  vertex_buffer: Vec<f32>,

  /// index buffer
  index_wgpu_buffer: Option<wgpu::Buffer>,

  index_buffer: Vec<u16>,

  /// transforms buffer
  transforms_buffer: wgpu::Buffer,

  bucket_type: BucketType,
}

impl<F> Bucket<F> {
  pub fn render<'frame>(
    &'frame self,
    pass: &mut wgpu::RenderPass<'frame>,
    queue: &wgpu::Queue,
    view: &View,
  ) {
    match (
      self.vertex_wgpu_buffer.as_ref(),
      self.index_wgpu_buffer.as_ref(),
    ) {
      (Some(vertex_buffer), Some(index_buffer)) => {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let (half_width, half_height) = view.get_half_size();
        queue.write_buffer(
          &self.transforms_buffer,
          0,
          bytemuck::cast_slice(&get_transforms(
            view.view_matrix,
            self.extent,
            half_width,
            half_height,
          )),
        );

        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_buffer.len() as u32, 0, 0..1);
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

pub trait AcceptFeatures<F, const T: BucketType>
where
  Self: Sized,
{
  fn new(device: Arc<wgpu::Device>, texture_format: &wgpu::TextureFormat, view: &View) -> Self;

  fn add_features(&mut self, features: &mut Vec<F>)
  where
    F: feature::WithGeometry<geo_types::GeometryCollection<f32>>;
}

pub trait AcceptExtent {
  fn set_extent(&mut self, extent: Vec<f32>);
}

impl<F> AcceptExtent for Bucket<F> {
  fn set_extent(&mut self, extent: Vec<f32>) {
    self.extent = extent.try_into().expect("extent wrong format");
  }
}
