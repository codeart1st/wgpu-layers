use geo_types::Geometry::*;
use log::info;
use wgpu::util::DeviceExt;

use crate::view::View;

pub mod feature;

const DIMENSIONS: usize = 2;

#[derive(Debug)]
pub struct Bucket<'a, F> {
  /// wgpu device
  device: &'a wgpu::Device,

  /// wgpu pipeline
  pipeline: wgpu::RenderPipeline,

  /// map features
  features: Vec<F>,

  /// wgpu bind group
  bind_group: wgpu::BindGroup,

  /// vertex buffer
  vertex_wgpu_buffer: Option<wgpu::Buffer>,

  vertex_buffer: Vec<f32>,

  /// index buffer
  index_wgpu_buffer: Option<wgpu::Buffer>,

  index_buffer: Vec<u16>,

  /// world buffer
  world_buffer: wgpu::Buffer,
}

impl<'a, F> Bucket<'a, F> {
  pub fn new(device: &'a wgpu::Device, texture_format: &wgpu::TextureFormat, view: &View) -> Self {
    let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        "shader/bucket.wgsl"
      ))),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
          ty: wgpu::BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });

    let world_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&view.view_matrix),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &bind_group_layout,
      entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: world_buffer.as_entire_binding(),
      }],
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
        targets: &[wgpu::ColorTargetState {
          format: *texture_format,
          blend: None,
          write_mask: wgpu::ColorWrites::ALL,
        }],
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
      bind_group,
      vertex_wgpu_buffer: None,
      vertex_buffer: Vec::with_capacity(0),
      index_wgpu_buffer: None,
      index_buffer: Vec::with_capacity(0),
      world_buffer,
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

        queue.write_buffer(
          &self.world_buffer,
          0,
          bytemuck::cast_slice(&view.view_matrix),
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

  pub fn add_features(&mut self, mut features: Vec<F>)
  where
    F: feature::WithGeometry<geo_types::Geometry<f32>>,
  {
    for feature in features.iter() {
      let geometry = feature.get_geometry();
      match geometry {
        Polygon(polygon) => {
          let exterior = polygon.exterior();
          let interior = polygon.interiors();
          let mut vertex_count = exterior.0.len();
          let mut rings = Vec::with_capacity(1 + interior.len());
          rings.push(exterior);
          interior.iter().for_each(|r| {
            rings.push(r);
            vertex_count += r.0.len();
          });
          let mut vertices = Vec::with_capacity(vertex_count * DIMENSIONS);
          let mut hole_indices = Vec::new();
          for (i, ring) in rings.iter().enumerate() {
            for coord in ring.coords() {
              vertices.push(coord.x);
              vertices.push(coord.y);
            }
            if i < rings.len() - 1 {
              hole_indices.push(vertices.len())
            }
          }
          let indices = earcutr::earcut(&vertices, &hole_indices, DIMENSIONS);
          self.vertex_buffer.append(&mut vertices);
          self
            .index_buffer
            .append(&mut indices.iter().map(|i| *i as u16).collect());
        }
        _ => {
          info!("Geometry type currently not supported");
        }
      }
    }
    self.features.append(&mut features);

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
}
