use std::{fmt::Debug, sync::Arc};

use log::{info, warn};

use crate::{
  bucket::{line_tessellation::LineTessellation, Bucket},
  view::View,
};

pub struct Renderer {
  /// wgpu device queue pair
  pub device_queue: (Arc<wgpu::Device>, Arc<wgpu::Queue>),

  /// preferred texutre format of surface
  pub texture_format: wgpu::TextureFormat,

  /// used view
  pub view: View,

  /// line tessellation
  line_tessellation: LineTessellation,

  /// wgpu surface
  surface: wgpu::Surface,

  /// wgpu surfaceconfiguration
  surface_config: wgpu::SurfaceConfiguration,
}

pub trait ToSurface {
  unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface;
}

impl Renderer {
  pub async fn new<W: ToSurface>(window: &W, (width, height): (u32, u32)) -> Self {
    let instance = wgpu::Instance::new(wgpu::Backends::all());

    let surface;
    unsafe {
      surface = window.create_surface(&instance);
    };

    info!("surface: {:?}", &surface);

    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .expect("Adapter not created.");

    info!("adapter: {:?}", &adapter);

    let (device, queue) = adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          features: wgpu::Features::default(),
          limits: wgpu::Limits::default(),
        },
        None,
      )
      .await
      .expect("Device can't be created.");

    info!("device: {:?}", device);

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let texture_format = surface
      .get_supported_formats(&adapter)
      .first()
      .expect("Can't get texture format for surface.")
      .to_owned();

    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: texture_format,
      width,
      height,
      present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &surface_config);

    let line_tessellation = LineTessellation::new((device.clone(), queue.clone()));

    Self {
      device_queue: (device, queue),
      texture_format,
      view: View::new((width, height)),
      line_tessellation,
      surface,
      surface_config,
    }
  }

  pub fn create_bucket<T>(&self) -> Bucket<T> {
    let (device, _) = &self.device_queue;
    Bucket::new(device.clone(), &self.texture_format, &self.view)
  }

  pub fn set_size(&mut self, (width, height): (u32, u32)) {
    let (device, _) = &self.device_queue;

    self.surface_config.width = width;
    self.surface_config.height = height;
    self.surface.configure(device, &self.surface_config);
    self.view.set_size((width, height));
  }

  pub async fn compute(&mut self) {
    let vertices = [0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0];
    let indices = [0, 1, 2, 3];

    self
      .line_tessellation
      .tessellate((&vertices, &indices))
      .await;
  }

  pub fn render<T: Debug>(&self, buckets: &[Bucket<T>]) {
    let (device, queue) = &self.device_queue;
    let mut command_encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let surface_texture = self
      .surface
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
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });
    } // out of scope

    for bucket in buckets.iter() {
      let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: true,
          },
        })],
        depth_stencil_attachment: None,
      });

      bucket.render(&mut pass, queue, &self.view);
    }

    queue.submit(Some(command_encoder.finish()));
    surface_texture.present();
  }
}
