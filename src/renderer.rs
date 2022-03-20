use std::fmt::Debug;

use log::info;

use crate::{bucket::Bucket, view::View};

pub struct Renderer {
  /// wgpu device
  pub device: wgpu::Device,

  /// preferred texutre format of surface
  pub texture_format: wgpu::TextureFormat,

  /// used view
  pub view: View,

  /// wgpu queue
  queue: wgpu::Queue,

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

    let texture_format = surface
      .get_preferred_format(&adapter)
      .expect("Can't get texture format for surface.");

    let surface_config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: texture_format,
      width,
      height,
      present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &surface_config);

    Self {
      device,
      texture_format,
      view: View::new((width, height)),
      surface,
      queue,
      surface_config,
    }
  }

  pub fn create_bucket<T>(&self) -> Bucket<T> {
    Bucket::new(&self.device, &self.texture_format, &self.view)
  }

  pub fn render<T: Debug>(&self, buckets: Vec<Bucket<T>>) {
    let mut command_encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let surface_texture = self
      .surface
      .get_current_texture()
      .expect("Can't get current texture");

    let view = surface_texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: None,
      color_attachments: &[wgpu::RenderPassColorAttachment {
        view: &view,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 0.4,
          }),
          store: true,
        },
      }],
      depth_stencil_attachment: None,
    });

    for bucket in buckets.iter() {
      let mut pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: true,
          },
        }],
        depth_stencil_attachment: None,
      });

      bucket.render(&mut pass, &self.queue, &self.view);
    }

    self.queue.submit(command_encoder.finish().try_into());
    surface_texture.present();
  }
}
