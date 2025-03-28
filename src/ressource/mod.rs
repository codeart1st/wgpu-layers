use std::{borrow::Cow, cell::RefCell, collections::HashMap, sync::Arc};

use material::{Material, MaterialManager, MaterialType};
use wgpu::util::DeviceExt;

use self::tile::{Bucket, BucketType, Tile, TileManager};

mod material;
pub mod tile;
pub mod view;

#[derive(Eq, Hash, PartialEq)]
enum BindGroupScope {
  Global = 0,
  Material = 1,
  Model = 2,
}

impl BindGroupScope {
  pub fn value(&self) -> usize {
    match self {
      BindGroupScope::Global => BindGroupScope::Global as usize,
      BindGroupScope::Material => BindGroupScope::Material as usize,
      BindGroupScope::Model => BindGroupScope::Model as usize,
    }
  }
}

#[derive(Eq, Hash, PartialEq)]
enum ShaderModuleScope {
  Common,
}

// help to manage wgpu ressources
pub struct RessourceManager {
  device: wgpu::Device,

  texture_format: wgpu::TextureFormat,

  material_manager: Option<RefCell<MaterialManager>>,

  tile_manager: Option<RefCell<TileManager>>,

  bind_group_layouts: [wgpu::BindGroupLayout; 3],

  shader_modules: HashMap<ShaderModuleScope, wgpu::ShaderModule>,
}

impl RessourceManager {
  pub fn new(device: wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
    let empty_desc = &wgpu::BindGroupLayoutDescriptor {
      label: None,
      entries: &[],
    };
    let bind_group_layouts = [
      device.create_bind_group_layout(empty_desc),
      device.create_bind_group_layout(empty_desc),
      device.create_bind_group_layout(empty_desc),
    ];
    let mut manager = Self {
      device,
      texture_format,
      material_manager: None,
      tile_manager: None,
      bind_group_layouts,
      shader_modules: HashMap::new(),
    };
    manager.material_manager = Some(RefCell::new(MaterialManager::new(&mut manager)));
    manager.tile_manager = Some(RefCell::new(TileManager::new(&mut manager)));
    manager
  }

  pub(self) fn create_shader_module(
    &mut self,
    scope: ShaderModuleScope,
    code: Cow<str>,
  ) -> wgpu::ShaderModule {
    #[allow(clippy::arc_with_non_send_sync)]
    let shader_module = self
      .device
      .create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(code),
      });
    let result = shader_module.clone();
    self.shader_modules.insert(scope, shader_module);
    result
  }

  pub(self) fn create_bind_group(
    &self,
    scope: &BindGroupScope,
    entries: &[wgpu::BindGroupEntry],
  ) -> wgpu::BindGroup {
    self.device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: None,
      layout: &self.bind_group_layouts[scope.value()],
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
    let pipeline_layout = self
      .device
      .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[
          &self.bind_group_layouts[0],
          &self.bind_group_layouts[1],
          &self.bind_group_layouts[2],
        ],
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
        cache: None,
      })
  }

  pub(self) fn register_bind_group_layout(
    &mut self,
    scope: BindGroupScope,
    desc: &wgpu::BindGroupLayoutDescriptor,
  ) {
    self.bind_group_layouts[scope.value()] = self.device.create_bind_group_layout(desc);
  }

  pub fn create_tile<F>(&self, bucket_type: BucketType, extent: [f32; 4]) -> Tile {
    match bucket_type {
      BucketType::Fill => Bucket::<F, { BucketType::Fill }>::new(self, extent),
      BucketType::Line => Bucket::<F, { BucketType::Line }>::new(self, extent),
      BucketType::Point => Bucket::<F, { BucketType::Point }>::new(self, extent),
    }
  }

  fn get_material(&self, material_type: MaterialType) -> Arc<Material> {
    self
      .material_manager
      .as_ref()
      .unwrap()
      .borrow_mut()
      .get(self, material_type)
  }
}
