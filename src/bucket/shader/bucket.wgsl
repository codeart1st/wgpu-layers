struct World {
  view_matrix: mat3x3<f32>,
};

@group(0) @binding(0)
var<uniform> world: World;

@group(0) @binding(1)
var<uniform> extent: vec4<f32>;

@vertex
fn vs_main(
  @location(0) pos: vec2<f32>
) -> @builtin(position) vec4<f32> {
  var tile_size = 4096.0;
  var tile_transform = mat3x3<f32>(
    (extent[2] - extent[0]) / tile_size, 0.0, 0.0,
    0.0, (extent[2] - extent[0]) / tile_size, 0.0,
    extent[0], extent[1], 1.0
  );
  var flip_tile_transform = mat3x3<f32>(
    1.0, 0.0, 0.0,
    0.0, -1.0, 0.0,
    0.0, tile_size, 1.0
  );
  var model_matrix = tile_transform * flip_tile_transform;
  var model_view_matrix = world.view_matrix * model_matrix;

  return vec4<f32>((model_view_matrix * vec3<f32>(pos, 1.0)), 1.0);
}

struct FragmentOutput {
  @location(0) color: vec4<f32>,
  @builtin(sample_mask) mask_out: u32
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> FragmentOutput {
  var alpha = 0.5;
  var extent = vec4<f32>(512.0, 512.0, 1024.0, 1024.0);

  var out: FragmentOutput;
  out.color = alpha * vec4<f32>(1.0, 0.0, 0.0, 1.0);

  // vector tile buffer clipping
  if (
    position.x < extent[0] ||
    position.x > extent[2] ||
    position.y < extent[1] ||
    position.y > extent[3]
  ) {
    out.mask_out = 0u;
  } else {
    out.mask_out = 0xFFFFFFFFu;
  }
  return out;
}