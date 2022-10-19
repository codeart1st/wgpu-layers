use crate::ressource::{RessourceManager, RessourceScope};

use super::{CreatePipeline, Material, MaterialType, Style};

impl CreatePipeline<{ MaterialType::Fill }> for Material {
  fn new(ressource_manager: &RessourceManager, shader_module: &wgpu::ShaderModule) -> Self {
    let vertex_state = wgpu::VertexState {
      module: shader_module,
      entry_point: "vs_fill",
      buffers: &[wgpu::VertexBufferLayout {
        array_stride: 8,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
          format: wgpu::VertexFormat::Float32x2,
          offset: 0,
          shader_location: 0,
        }],
      }],
    };
    let fragment_state = wgpu::FragmentState {
      module: shader_module,
      entry_point: "fs_fill",
      targets: &[Some(wgpu::ColorTargetState {
        format: ressource_manager.texture_format,
        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::default(),
      })],
    };
    let pipeline = ressource_manager.create_render_pipeline(vertex_state, fragment_state);

    let style = Style {
      fill_color: [0.506, 0.694, 0.31, 1.0],
      stroke_color: [0.0, 0.0, 0.0, 1.0],
      stroke_width: 10.0,
    };
    let style_buffer = ressource_manager.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: None,
      contents: bytemuck::cast_slice(&[style]),
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group = ressource_manager.create_bind_group(&RessourceScope::Material,
      &[wgpu::BindGroupEntry {
        binding: 1,
        resource: style_buffer.as_entire_binding(),
      }],
    );

    Self {
      pipeline,
      bind_group,
      material_type: MaterialType::Fill,
    }
  }
}
