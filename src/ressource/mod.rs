use std::{borrow::Cow, collections::HashMap, sync::Arc};

use material::{Material, MaterialManager, MaterialType};
use wgpu::util::DeviceExt;

mod material;

#[derive(Eq, Hash, PartialEq)]
pub(self) enum RessourceScope {
  World,
  Camera,
  Model,
  Material,
}

#[derive(Eq, Hash, PartialEq)]
pub(self) enum ShaderModuleScope {
  Common,
}

// help to manage wgpu ressources
pub struct RessourceManager {
  device: Arc<wgpu::Device>,

  texture_format: wgpu::TextureFormat,

  material_manager: Option<MaterialManager>,

  bind_group_layouts: HashMap<RessourceScope, wgpu::BindGroupLayout>,

  shader_modules: HashMap<ShaderModuleScope, Arc<wgpu::ShaderModule>>,
}

impl RessourceManager {
  pub fn new(device: Arc<wgpu::Device>, texture_format: wgpu::TextureFormat) -> Self {
    let mut manager = Self {
      device,
      texture_format,
      material_manager: None,
      bind_group_layouts: HashMap::new(),
      shader_modules: HashMap::new(),
    };
    manager.material_manager = Some(MaterialManager::new(&mut manager));
    manager
  }

  pub(self) fn create_shader_module(
    &mut self,
    scope: ShaderModuleScope,
    code: Cow<str>,
  ) -> Arc<wgpu::ShaderModule> {
    let shader_module = Arc::new(
      self
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
          label: None,
          source: wgpu::ShaderSource::Wgsl(code),
        }),
    );
    let result = shader_module.clone();
    self.shader_modules.insert(scope, shader_module);
    result
  }

  pub(self) fn create_bind_group(
    &self,
    scope: &RessourceScope,
    entries: &[wgpu::BindGroupEntry],
  ) -> wgpu::BindGroup {
    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: self.bind_group_layouts.get(scope).unwrap(),
      entries,
    })
  }

  pub(self) fn create_buffer_init(&self, desc: &wgpu::util::BufferInitDescriptor) -> wgpu::Buffer {
    self.device.create_buffer_init(desc)
  }

  pub(self) fn create_render_pipeline(
    &self,
    vertex_state: wgpu::VertexState,
    fragment_state: wgpu::FragmentState,
  ) -> wgpu::RenderPipeline {
    let bind_group_layouts: Vec<&wgpu::BindGroupLayout> =
      self.bind_group_layouts.values().collect();
    let pipeline_layout = self
      .device
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &bind_group_layouts[..],
        push_constant_ranges: &[],
      });
    self
      .device
      .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: vertex_state,
        fragment: Some(fragment_state),
        primitive: wgpu::PrimitiveState::default(),
        multisample: wgpu::MultisampleState::default(),
        depth_stencil: None,
        multiview: None,
      })
  }

  pub(self) fn register_bind_group_layout(
    &mut self,
    scope: RessourceScope,
    desc: &wgpu::BindGroupLayoutDescriptor,
  ) {
    let bind_group_layout = self.device.create_bind_group_layout(desc);
    self.bind_group_layouts.insert(scope, bind_group_layout);
  }

  fn get_material(&self, material_type: MaterialType) -> Arc<Material> {
    self
      .material_manager
      .as_ref()
      .unwrap()
      .get(self, material_type)
  }
}
