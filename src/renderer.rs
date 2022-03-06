use log::info;

pub struct Renderer {

  /// wgpu device
  pub device: wgpu::Device,

  /// preferred texutre format of surface
  pub texture_format: wgpu::TextureFormat,

  /// wgpu queue
  queue: wgpu::Queue,

  /// wgpu surface
  surface: wgpu::Surface,

  /// wgpu surfaceconfiguration
  surface_config: wgpu::SurfaceConfiguration
}

impl Renderer {
  pub async fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W, (width, height): (u32, u32)) -> Self {
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let surface;
    unsafe { surface = instance.create_surface(&window); };

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      force_fallback_adapter: false,
      compatible_surface: Some(&surface)
    })
      .await
      .expect("Adapter not created.");

    info!("adapter: {:?}", &adapter);

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
      label: None,
      features: wgpu::Features::default(),
      limits: wgpu::Limits::default()
    }, None)
      .await
      .expect("Device can't be created.");

    info!("device: {:?}", device);

    let texture_format = surface.get_preferred_format(&adapter)
      .expect("Can't get texture format for surface.");

    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: texture_format,
      width,
      height,
      present_mode: wgpu::PresentMode::Fifo
    };

    surface.configure(&device, &surface_config);

    Self {
      device,
      texture_format,
      surface,
      queue,
      surface_config
    }
  }

  pub fn test_draw(&self) {
    let mut command_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: None
    });

    let surface_texture = self.surface.get_current_texture()
      .expect("Can't get current texture");

    let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
    {
      command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
            store: true
          }
        }],
        depth_stencil_attachment: None
      });
    }

    self.queue.submit(command_encoder.finish().try_into());
    surface_texture.present();
  }
}
