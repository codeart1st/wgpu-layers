use std::{convert::TryInto, sync::Arc};

use geo_types::Geometry::*;
use log::info;
use wgpu::util::DeviceExt;

use crate::view::View;

pub mod feature;
pub mod line_tessellation;

const DIMENSIONS: usize = 2;

const TILE_SIZE: f32 = 4096.0;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct Transforms {
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
pub struct Style {
  /// fill color
  fill_color: [f32; 4],
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

impl<F> Bucket<F> {
  pub fn new(device: Arc<wgpu::Device>, texture_format: &wgpu::TextureFormat, view: &View) -> Self {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        "shader/fill.wgsl"
      ))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        },
      ],
    });

    let (half_width, half_height) = view.get_half_size();
    let transforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&get_transforms(
        view.view_matrix,
        [0.0; 4],
        half_width,
        half_height,
      )),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let style = Style {
      fill_color: [0.506, 0.694, 0.31, 1.0],
    };
    let style_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&[style]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: transforms_buffer.as_entire_binding(),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: style_buffer.as_entire_binding(),
        },
      ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: None,
      bind_group_layouts: &[&bind_group_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: None,
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader_module,
        entry_point: "vs_main",
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: 8,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &[wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
          }],
        }],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader_module,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: *texture_format,
          blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::default(),
        })],
      }),
      primitive: wgpu::PrimitiveState::default(),
      multisample: wgpu::MultisampleState::default(),
      depth_stencil: None,
      multiview: None,
    });

    Self {
      device,
      pipeline,
      features: Vec::new(),
      extent: [0.0; 4],
      bind_group,
      vertex_wgpu_buffer: None,
      vertex_buffer: Vec::with_capacity(0),
      index_wgpu_buffer: None,
      index_buffer: Vec::with_capacity(0),
      transforms_buffer,
    }
  }

  pub fn render<'b>(&'b self, pass: &mut wgpu::RenderPass<'b>, queue: &wgpu::Queue, view: &View) {
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

  pub fn add_features(&mut self, features: &mut Vec<F>)
  where
    F: feature::WithGeometry<geo_types::GeometryCollection<f32>>,
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
                hole_indices.push(vertices.len())
              }
            }
            let indices = earcutr::earcut(&vertices, &hole_indices, DIMENSIONS);
            let offset = (self.vertex_buffer.len() / DIMENSIONS) as u16;
            self.vertex_buffer.append(&mut vertices);
            self
              .index_buffer
              .append(&mut indices.iter().map(|i| (*i as u16) + offset).collect());
          }
          _ => {
            info!("Geometry type currently not supported");
          }
        }
      }
    }
    self.features.append(features);

    self.vertex_wgpu_buffer = Some(self.device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&self.vertex_buffer),
        usage: wgpu::BufferUsages::VERTEX,
      },
    ));

    self.index_wgpu_buffer = Some(self.device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&self.index_buffer),
        usage: wgpu::BufferUsages::INDEX,
      },
    ));
  }

  pub fn set_extent(&mut self, extent: Vec<f32>) {
    self.extent = extent.try_into().expect("extent wrong format");
  }
}
