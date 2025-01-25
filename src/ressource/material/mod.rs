use std::{collections::HashMap, marker::ConstParamTy, mem, sync::Arc};

use super::{BindGroupScope, RessourceManager, ShaderModuleScope};

mod fill;
mod line;
mod point;

#[repr(C)]
#[derive(Copy, Clone, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct Style {
  /// fill color
  fill_color: [f32; 4],

  /// stroke color
  stroke_color: [f32; 4],

  /// stroke width
  stroke_width: f32,

  _pad: [u32; 3],
}

#[derive(PartialEq, Eq, Hash, Clone, ConstParamTy)]
pub enum MaterialType {
  Fill,
  Line,
  Point,
}

pub struct Material {
  /// wgpu pipeline
  pipeline: wgpu::RenderPipeline,

  /// wgpu bind group
  bind_group: wgpu::BindGroup,
}

impl Material {
  pub fn set<'frame>(&'frame self, render_pass: &mut wgpu::RenderPass<'frame>) {
    render_pass.set_pipeline(&self.pipeline);
    render_pass.set_bind_group(BindGroupScope::Material as u32, Some(&self.bind_group), &[]);
  }
}

pub trait CreatePipeline<const T: MaterialType>
where
  Self: Sized,
{
  fn new(ressource_manager: &RessourceManager, shader_module: &wgpu::ShaderModule) -> Self;
}

pub struct MaterialManager {
  shader_module: wgpu::ShaderModule,

  materials: HashMap<MaterialType, Arc<Material>>,
}

impl MaterialManager {
  pub fn new(ressource_manager: &mut RessourceManager) -> Self {
    let shader_module = ressource_manager.create_shader_module(
      ShaderModuleScope::Common,
      std::borrow::Cow::Borrowed(include_str!("shader/common.wgsl")),
    );

    ressource_manager.register_bind_group_layout(
      BindGroupScope::Material,
      &wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(mem::size_of::<Style>() as _),
          },
          count: None,
        }],
      },
    );

    Self {
      shader_module,
      materials: HashMap::new(),
    }
  }

  pub fn get(
    &mut self,
    ressource_manager: &RessourceManager,
    material_type: MaterialType,
  ) -> Arc<Material> {
    let material = self
      .materials
      .entry(material_type.clone())
      .or_insert(match material_type {
        MaterialType::Fill =>
        {
          #[allow(clippy::arc_with_non_send_sync)]
          Arc::new(<Material as CreatePipeline<{ MaterialType::Fill }>>::new(
            ressource_manager,
            &self.shader_module,
          ))
        }
        MaterialType::Line =>
        {
          #[allow(clippy::arc_with_non_send_sync)]
          Arc::new(<Material as CreatePipeline<{ MaterialType::Line }>>::new(
            ressource_manager,
            &self.shader_module,
          ))
        }
        MaterialType::Point =>
        {
          #[allow(clippy::arc_with_non_send_sync)]
          Arc::new(<Material as CreatePipeline<{ MaterialType::Point }>>::new(
            ressource_manager,
            &self.shader_module,
          ))
        }
      });
    material.clone()
  }
}
