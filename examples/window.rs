#[cfg(not(target_arch = "wasm32"))]
mod example {
  use log::info;
  use std::sync::Arc;

  struct MapWindow {
    window: Arc<winit::window::Window>,
  }

  impl wgpu_layers::renderer::ToSurface for MapWindow {
    fn create_surface(
      &self,
      instance: &wgpu::Instance,
    ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
      instance.create_surface(wgpu::SurfaceTarget::Window(Box::new(self.window.clone())))
    }
  }

  fn create_map_window() -> (winit::event_loop::EventLoop<()>, MapWindow) {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let size = winit::dpi::PhysicalSize::new(512, 512);
    let window_builder = winit::window::WindowBuilder::new()
      .with_title("wgpu-map")
      //.with_transparent(true)
      .with_resizable(false)
      .with_min_inner_size(size)
      .with_max_inner_size(size)
      .with_inner_size(size);

    let window = window_builder
      .build(&event_loop)
      .expect("Window can't be created");

    (
      event_loop,
      MapWindow {
        window: Arc::new(window),
      },
    )
  }

  async fn start(
    event_loop: winit::event_loop::EventLoop<()>,
    window: MapWindow,
    size: winit::dpi::PhysicalSize<u32>,
  ) {
    wgpu_layers::init(&window, (size.width, size.height)).await;

    #[rustfmt::skip]
    let view_matrix = vec![
      1.15142285e-7, -0.0, 0.0, 0.0,
      0.0, 1.15142285e-7, 0.0, 0.0,
      -0.27666306, -0.7963807, 1.0, 0.0,
      0.0, 0.0, 0.0, 1.0,
    ];
    wgpu_layers::render(view_matrix, vec![size.width, size.height]);

    info!("renderer init");

    let _ = event_loop.run(move |event, elwt| {
      if let winit::event::Event::WindowEvent { event, .. } = event {
        match event {
          winit::event::WindowEvent::CloseRequested => elwt.exit(),
          winit::event::WindowEvent::KeyboardInput {
            event:
              winit::event::KeyEvent {
                state: winit::event::ElementState::Pressed,
                logical_key,
                ..
              },
            ..
          } => match logical_key {
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => elwt.exit(),
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {}
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {}
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {}
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {}
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp) => {}
            winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown) => {}
            _ => (),
          },
          winit::event::WindowEvent::Resized(_) => {}
          winit::event::WindowEvent::RedrawRequested => {
            #[rustfmt::skip]
            let view_matrix = vec![
              1.15142285e-7, -0.0, 0.0, 0.0,
              0.0, 1.15142285e-7, 0.0, 0.0,
              -0.27666306, -0.7963807, 1.0, 0.0,
              0.0, 0.0, 0.0, 1.0,
            ];
            wgpu_layers::render(view_matrix, vec![size.width, size.height]);
          }
          _ => (),
        }
      }
    });
  }

  pub fn main() {
    env_logger::init_from_env(
      env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let (event_loop, map_window) = create_map_window();
    let size = map_window.window.inner_size();

    pollster::block_on(start(event_loop, map_window, size));
  }
}

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  example::main();
}
