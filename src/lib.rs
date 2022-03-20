use geo_types::polygon;
use log::info;

mod bucket;
pub mod renderer;
mod view;

#[cfg(target_arch = "wasm32")]
mod wasm {
  use wasm_bindgen::prelude::*;

  pub use wasm_bindgen_rayon::init_thread_pool;

  impl super::renderer::ToSurface for web_sys::OffscreenCanvas {
    unsafe fn create_surface(&self, instance: &wgpu::Instance) -> wgpu::Surface {
      instance.create_surface_from_offscreen_canvas(self)
    }
  }

  #[wasm_bindgen]
  pub async fn start(canvas: web_sys::OffscreenCanvas) -> JsValue {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    Closure::wrap(
      Box::new(super::init(&canvas, (canvas.width(), canvas.height())).await)
        as Box<dyn FnMut(Vec<f32>)>,
    )
    .into_js_value()
  }
}

pub async fn init<W: renderer::ToSurface>(window: &W, size: (u32, u32)) -> impl FnMut(Vec<f32>) {
  let mut renderer = renderer::Renderer::new(window, size).await;

  info!("renderer initialized");

  move |view_matrix: Vec<f32>| {
    info!("{:?}", view_matrix);
    renderer.view.view_matrix = view_matrix.try_into().expect("View matrix is wrong");

    let mut bucket = renderer.create_bucket();

    let test_geometry: geo_types::Geometry<f32> = (polygon!(
      exterior: [
        (x: -3862117.868494708, y: 9809176.416636087),
        (x: 8579526.353004107, y: 9915819.08139179),
        (x: 901254.4905934092, y: 1597691.2304468695),
        (x: -3862117.868494708, y: 9809176.416636087)
      ],
      interiors: []
    ))
    .try_into()
    .expect("Can't convert polygon to geometry");

    let test_feature = bucket::feature::Feature {
      geometry: test_geometry,
      properties: None,
    };

    bucket.add_features(vec![test_feature]);

    renderer.render(vec![bucket]);
  }
}
