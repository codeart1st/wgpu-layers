#[cfg(not(target_arch = "wasm32"))]
mod example {
  use log::info;
  use pollster::FutureExt;
  use std::sync::Arc;

  struct Application {
    window: Option<Arc<winit::window::Window>>,
    size: winit::dpi::PhysicalSize<u32>,
  }

  impl Application {
    fn new() -> Self {
      Self {
        window: None,
        size: winit::dpi::PhysicalSize::new(512, 512),
      }
    }
    fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
      let window_attributes = winit::window::Window::default_attributes()
        .with_title("wgpu-map")
        //.with_transparent(true)
        .with_resizable(false)
        .with_min_inner_size(self.size)
        .with_max_inner_size(self.size)
        .with_inner_size(self.size);

      let window = event_loop
        .create_window(window_attributes)
        .expect("Window can't be created");

      self.window = Some(Arc::new(window));
    }
  }

  impl winit::application::ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
      info!("Resumed");

      self.create_window(event_loop);

      wgpu_layers::init(self, (self.size.width, self.size.height)).block_on();
    }

    fn window_event(
      &mut self,
      event_loop: &winit::event_loop::ActiveEventLoop,
      _window_id: winit::window::WindowId,
      event: winit::event::WindowEvent,
    ) {
      match event {
        winit::event::WindowEvent::CloseRequested => event_loop.exit(),
        winit::event::WindowEvent::KeyboardInput {
          event:
            winit::event::KeyEvent {
              state: winit::event::ElementState::Pressed,
              logical_key,
              ..
            },
          ..
        } => match logical_key {
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) => event_loop.exit(),
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {}
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown) => {}
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {}
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {}
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageUp) => {}
          winit::keyboard::Key::Named(winit::keyboard::NamedKey::PageDown) => {}
          _ => (),
        },
        winit::event::WindowEvent::Resized(size) => {
          self.size = size;
        }
        winit::event::WindowEvent::RedrawRequested => {
          #[rustfmt::skip]
            let view_matrix = vec![
              1.15142285e-7, -0.0, 0.0, 0.0,
              0.0, 1.15142285e-7, 0.0, 0.0,
              -0.27666306, -0.7963807, 1.0, 0.0,
              0.0, 0.0, 0.0, 1.0,
            ];
          wgpu_layers::render(view_matrix, vec![self.size.width, self.size.height]);
        }
        _ => (),
      }
    }
  }

  impl wgpu_layers::renderer::ToSurface for Application {
    fn create_surface(
      &self,
      instance: &wgpu::Instance,
    ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
      instance.create_surface(wgpu::SurfaceTarget::Window(Box::new(
        <std::option::Option<std::sync::Arc<winit::window::Window>> as Clone>::clone(&self.window)
          .unwrap()
          .clone(),
      )))
    }
  }

  pub fn main() {
    env_logger::init_from_env(
      env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let mut application = Application::new();

    let _ = event_loop.run_app(&mut application);
  }
}

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  example::main();
}
