#[cfg(not(target_arch = "wasm32"))]
mod example {
  use log::info;

  struct MapWindow {
    window: winit::window::Window,
  }

  impl wgpu_layers::renderer::ToSurface for MapWindow {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface(&self.window)
    }
  }

  fn create_map_window() -> (winit::event_loop::EventLoop<()>, MapWindow) {
    let event_loop = winit::event_loop::EventLoop::new();
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

    (event_loop, MapWindow { window })
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

    event_loop.run(move |event, _, control_flow| match event {
      winit::event::Event::WindowEvent {
        event: winit::event::WindowEvent::CloseRequested,
        ..
      } => *control_flow = winit::event_loop::ControlFlow::Exit,
      winit::event::Event::WindowEvent {
        event:
          winit::event::WindowEvent::KeyboardInput {
            input:
              winit::event::KeyboardInput {
                state: winit::event::ElementState::Pressed,
                virtual_keycode,
                ..
              },
            ..
          },
        ..
      } => match virtual_keycode {
        Some(winit::event::VirtualKeyCode::Escape) => {
          *control_flow = winit::event_loop::ControlFlow::Exit
        }
        Some(winit::event::VirtualKeyCode::Up) => {}
        Some(winit::event::VirtualKeyCode::Down) => {}
        Some(winit::event::VirtualKeyCode::Left) => {}
        Some(winit::event::VirtualKeyCode::Right) => {}
        Some(winit::event::VirtualKeyCode::PageUp) => {}
        Some(winit::event::VirtualKeyCode::PageDown) => {}
        Some(winit::event::VirtualKeyCode::A) => {}
        Some(winit::event::VirtualKeyCode::D) => {}
        _ => (),
      },
      winit::event::Event::WindowEvent {
        event: winit::event::WindowEvent::Resized(_),
        ..
      } => {}
      winit::event::Event::MainEventsCleared => {
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
