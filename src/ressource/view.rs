use std::mem;

use super::{BindGroupScope, RessourceManager};

#[rustfmt::skip]
const NORMALIZED_MATRIX: [[f32; 4]; 4] = [
  [1.0, 0.0, 0.0, 0.0],
  [0.0, 1.0, 0.0, 0.0],
  [0.0, 0.0, 1.0, 0.0],
  [0.0, 0.0, 0.0, 1.0],
];

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct ViewBuffer {
  /// transformation matrix world-space to view-space
  view_matrix: [[f32; 4]; 4],

  width: u32,

  height: u32,

  _pad: [u32; 2],
}

pub struct View {
  bind_group: wgpu::BindGroup,

  /// width of surface
  width: u32,

  /// height of surface
  height: u32,

  /// half width of surface
  half_width: f32,

  /// half height of surface
  half_height: f32,

  view_buffer: ViewBuffer,

  view_matrix_buffer: wgpu::Buffer,
}

impl View {
  pub fn new((width, height): (u32, u32), ressource_manager: &mut RessourceManager) -> Self {
    let view_matrix = ViewBuffer {
      view_matrix: NORMALIZED_MATRIX,
      width,
      height,
      _pad: [0, 0],
    };
    let view_matrix_buffer =
      ressource_manager.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[view_matrix]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      });

    ressource_manager.register_bind_group_layout(
      BindGroupScope::Global,
      &wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(mem::size_of::<ViewBuffer>() as _),
          },
          count: None,
        }],
      },
    );

    let bind_group = ressource_manager.create_bind_group(
      &BindGroupScope::Global,
      &[wgpu::BindGroupEntry {
        binding: 0,
        resource: view_matrix_buffer.as_entire_binding(),
      }],
    );

    Self {
      bind_group,
      width,
      height,
      half_width: width as f32 * 0.5,
      half_height: height as f32 * 0.5,
      view_buffer: view_matrix,
      view_matrix_buffer,
    }
  }

  pub fn set<'frame>(
    &'frame self,
    render_pass: &mut wgpu::RenderPass<'frame>,
    queue: &wgpu::Queue,
  ) {
    render_pass.set_bind_group(BindGroupScope::Global as u32, &self.bind_group, &[]);

    queue.write_buffer(
      &self.view_matrix_buffer,
      0,
      bytemuck::cast_slice(&[self.view_buffer]),
    );
  }

  pub fn set_size(&mut self, (width, height): (u32, u32)) {
    self.width = width;
    self.height = height;
    self.half_width = width as f32 * 0.5;
    self.half_height = height as f32 * 0.5;
    self.view_buffer.width = width;
    self.view_buffer.height = height;
  }

  pub fn set_view_matrix(&mut self, view_matrix: [[f32; 4]; 4]) {
    self.view_buffer.view_matrix = view_matrix;
  }

  pub fn get_view_matrix(&self) -> [[f32; 4]; 4] {
    self.view_buffer.view_matrix
  }

  pub fn get_size(&self) -> (u32, u32) {
    (self.width, self.height)
  }

  pub fn get_half_size(&self) -> (f32, f32) {
    (self.half_width, self.half_height)
  }
}
