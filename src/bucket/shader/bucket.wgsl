struct Transforms {
  view_matrix: mat4x4<f32>,
  model_matrix: mat4x4<f32>,
  model_view_matrix: mat4x4<f32>,
  clipping_rect: vec4<f32>
};

@group(0) @binding(0)
var<uniform> transforms: Transforms;

@group(0) @binding(1)
var<uniform> extent: vec4<f32>;

@vertex
fn vs_main(
  @location(0) pos: vec2<f32>
) -> @builtin(position) vec4<f32> {
  return transforms.model_view_matrix * vec4<f32>(pos, 0.0, 1.0);
}

struct FragmentOutput {
  @location(0) color: vec4<f32>,
  @builtin(sample_mask) mask_out: u32
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> FragmentOutput {
  var alpha = 0.5;

  var out: FragmentOutput;
  out.color = alpha * vec4<f32>(1.0, 0.0, 0.0, 1.0); // pre-multiplied alpha

  if (
    position.x < transforms.clipping_rect[0] ||
    position.y < transforms.clipping_rect[1] ||
    position.x > transforms.clipping_rect[2] ||
    position.y > transforms.clipping_rect[3]
  ) {
    out.mask_out = 0x0u;
  } else {
    out.mask_out = 0xFFFFFFFFu;
  }
  return out;
}