use log::info;
use geo_types::polygon;
use rayon::prelude::*;

pub mod renderer;
mod bucket;

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
  pub async fn start(canvas: web_sys::OffscreenCanvas) {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    #[cfg(feature = "console_log")]
    console_log::init_with_level(log::Level::Info).expect("error initializing logger");

    super::init(&canvas, (canvas.width(), canvas.height())).await;
  }
}

pub async fn init<W: renderer::ToSurface>(window: &W, size: (u32, u32)) {
  let renderer = renderer::Renderer::new(window, size).await;

  info!("renderer initialized");

  let buckets: Vec<bucket::Bucket<bucket::feature::Feature<geo_types::Geometry<f32>>>> = vec![];
  renderer.render(buckets);

  (0..1).into_par_iter().for_each(|x| {
    let mut bucket = renderer.create_bucket();

    // EPSG:3857
    let test_geometry: geo_types::Geometry<f32> = (polygon!(
      exterior: [
        /*(x: 1458675.916789971, y: 6911404.021700942),
        (x: 1527996.1263083573, y: 6910479.752240697),
        (x: 1487328.2700575707, y: 6858720.662466968),
        (x: 1458675.916789971, y: 6911404.021700942)*/
        (x: -0.5, y: 0.5),
        (x: -0.5, y: -0.5),
        (x: 0.5, y: -0.5),
        (x: 0.5, y: 0.5),
        (x: -0.5, y: 0.5)
      ],
      interiors: [
        [
          (x: 0.25, y: 0.25),
          (x: 0.25, y: -0.25),
          (x: -0.25, y: -0.25),
          (x: -0.25, y: 0.25),
          (x: 0.25, y: 0.25)
        ]
      ]
    )).try_into().expect("Can't convert polygon to geometry");
    let test_feature = bucket::feature::Feature {
      geometry: test_geometry,
      properties: None
    };

    bucket.add_features(vec![test_feature]);

    renderer.render(vec![bucket]);
  });
}