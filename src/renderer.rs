use log::info;

use crate::{
  ressource::{
    RessourceManager,
    tile::{BucketType, Tile},
    view::View,
  },
  tessellation::LineTessellation,
};

const PREFERRED_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;
const PREFERRED_ALPHA_MODE: wgpu::CompositeAlphaMode = wgpu::CompositeAlphaMode::PreMultiplied;

pub struct Renderer {
  /// wgpu device queue pair
  pub device_queue: (wgpu::Device, wgpu::Queue),

  /// preferred texutre format of surface
  pub texture_format: wgpu::TextureFormat,

  /// used view
  pub view: View,

  /// line tessellation
  line_tessellation: LineTessellation,

  /// wgpu surface
  swapchain: wgpu::Surface<'static>,

  /// wgpu surfaceconfiguration
  swapchain_config: wgpu::SurfaceConfiguration,

  pub ressource_manager: RessourceManager,
}

pub trait ToSurface {
  fn create_surface(
    &self,
    instance: &wgpu::Instance,
  ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError>;
}

impl Renderer {
  pub async fn new<W: ToSurface>(window: &W, (width, height): (u32, u32)) -> Self {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
      backends: wgpu::Backends::from_env().unwrap_or(wgpu::Backends::all()),
      ..Default::default()
    });

    let swapchain = match window.create_surface(&instance) {
      Ok(surface) => surface,
      Err(err) => {
        panic!("{}", err.to_string())
      }
    };

    info!("surface: {:?}", &swapchain);

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::from_env()
          .unwrap_or(wgpu::PowerPreference::HighPerformance),
        force_fallback_adapter: false,
        compatible_surface: Some(&swapchain),
      })
      .await
      .expect("Adapter not created.");

    info!("adapter: {:?}", &adapter);

    let (device, queue) = adapter
      .request_device(&wgpu::DeviceDescriptor::default())
      .await
      .expect("Device can't be created.");

    info!("device: {:?}", device);

    let swapchain_capabilities = swapchain.get_capabilities(&adapter);

    info!(
      "supported surface formats: {:?}",
      swapchain_capabilities.formats
    );

    let texture_format = if swapchain_capabilities
      .formats
      .contains(&PREFERRED_TEXTURE_FORMAT)
    {
      PREFERRED_TEXTURE_FORMAT
    } else {
      swapchain_capabilities
        .formats
        .first()
        .expect("Can't get texture format for surface.")
        .to_owned()
    };

    info!(
      "supported alpha modes: {:?}",
      swapchain_capabilities.alpha_modes
    );

    let alpha_mode = if swapchain_capabilities
      .alpha_modes
      .contains(&PREFERRED_ALPHA_MODE)
    {
      PREFERRED_ALPHA_MODE
    } else {
      swapchain_capabilities
        .alpha_modes
        .first()
        .expect("Can't get present mode for surface.")
        .to_owned()
    };

    let swapchain_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: texture_format,
      width,
      height,
      desired_maximum_frame_latency: 2,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode,
      view_formats: vec![],
    };

    swapchain.configure(&device, &swapchain_config);

    let line_tessellation = LineTessellation::new((device.clone(), queue.clone()));

    let mut ressource_manager = RessourceManager::new(device.clone(), texture_format);

    Self {
      device_queue: (device, queue),
      texture_format,
      view: View::new((width, height), &mut ressource_manager),
      line_tessellation,
      swapchain,
      swapchain_config,
      ressource_manager,
    }
  }

  pub fn create_tile<F>(&self, bucket_type: BucketType, extent: [f32; 4]) -> Tile {
    self.ressource_manager.create_tile::<F>(bucket_type, extent)
  }

  pub fn set_size(&mut self, (width, height): (u32, u32)) {
    let (device, _) = &self.device_queue;

    self.swapchain_config.width = width;
    self.swapchain_config.height = height;
    self.swapchain.configure(device, &self.swapchain_config);
    self.view.set_size((width, height));
  }

  pub async fn compute(
    &mut self,
    vertices: &[f32],
    indices: &[u32],
  ) -> (wgpu::Buffer, wgpu::Buffer) {
    self.line_tessellation.tessellate((vertices, indices)).await
  }

  pub fn render(&self, tiles: &[Tile]) {
    let (device, queue) = &self.device_queue;
    let mut command_encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let surface_texture = self
      .swapchain
      .get_current_texture()
      .expect("Can't get current texture");

    let view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    {
      command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.0,
              g: 0.412,
              b: 0.58,
              a: 1.0,
            }),
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });
    } // out of scope

    {
      let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });

      self.view.set(&mut render_pass, queue);

      // FIXME: set material / shader here. group by material in bucket
      for tile in tiles.iter() {
        tile.render(&mut render_pass, queue, &self.view);
      }
    }

    queue.submit(Some(command_encoder.finish()));
    surface_texture.present();
  }
}
