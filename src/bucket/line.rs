use std::sync::Arc;

use geo_types::Geometry::*;
use log::info;
use wgpu::util::DeviceExt;

use crate::view::View;

use super::{feature::WithGeometry, get_transforms, AcceptFeatures, Bucket, BucketType, Style};

const DIMENSIONS: usize = 2;

impl<F> AcceptFeatures<F, { BucketType::Line }> for Bucket<F> {
  fn new(device: Arc<wgpu::Device>, texture_format: &wgpu::TextureFormat, view: &View) -> Self {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: None,
      source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        "shader/styling.wgsl"
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
      stroke_color: [0.0, 0.0, 0.0, 1.0],
      stroke_width: 10.0,
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
        entry_point: "vs_stroke",
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: 8,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &[
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 0,
            },
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 0,
              shader_location: 1,
            },
          ],
        }],
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader_module,
        entry_point: "fs_stroke",
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
      bucket_type: BucketType::Line,
    }
  }

  fn add_features(&mut self, features: &mut Vec<F>)
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
}
