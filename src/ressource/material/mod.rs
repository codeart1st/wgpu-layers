use std::{collections::HashMap, sync::Arc};

use super::{RessourceManager, RessourceScope, ShaderModuleScope};

mod fill;
mod line;

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

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum MaterialType {
  Fill,
  Line,
}

#[derive(Debug)]
pub struct Material {
  /// wgpu pipeline
  pipeline: wgpu::RenderPipeline,

  /// wgpu bind group
  bind_group: wgpu::BindGroup,

  material_type: MaterialType,
}

pub trait CreatePipeline<const T: MaterialType>
where
  Self: Sized,
{
  fn new(ressource_manager: &RessourceManager, shader_module: &wgpu::ShaderModule) -> Self;
}

pub struct MaterialManager {
  shader_module: Arc<wgpu::ShaderModule>,

  materials: HashMap<MaterialType, Arc<Material>>,
}

impl MaterialManager {
  pub fn new(ressource_manager: &mut RessourceManager) -> Self {
    let shader_module = ressource_manager.create_shader_module(
      ShaderModuleScope::Common,
      std::borrow::Cow::Borrowed(include_str!("shader/common.wgsl")),
    );

    ressource_manager.register_bind_group_layout(
      RessourceScope::Material,
      &wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
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
    let material =
      self
        .materials
        .entry(material_type.clone())
        .or_insert(match material_type {
          MaterialType::Fill => Arc::new(
            <Material as CreatePipeline<{ MaterialType::Line }>>::new(ressource_manager, &self.shader_module),
          ),
          MaterialType::Line => Arc::new(
            <Material as CreatePipeline<{ MaterialType::Line }>>::new(ressource_manager, &self.shader_module),
          ),
        });
    material.clone()
  }
}
