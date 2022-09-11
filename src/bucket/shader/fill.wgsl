struct Transforms {
  view_matrix: mat4x4<f32>,
  model_matrix: mat4x4<f32>,
  model_view_matrix: mat4x4<f32>,
  clipping_rect: vec4<f32>
};

struct Style {
  fill_color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> transforms: Transforms;

@group(0) @binding(1)
var<uniform> style: Style;

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
  var color = style.fill_color.a * vec4<f32>(style.fill_color.rgb, 1.0); // pre-multiplied alpha
  var fragment_output = FragmentOutput(color, 0xFFFFFFFFu);

  if (
    position.x < transforms.clipping_rect[0] ||
    position.y < transforms.clipping_rect[1] ||
    position.x > transforms.clipping_rect[2] ||
    position.y > transforms.clipping_rect[3]
  ) {
    fragment_output.mask_out = 0u;
  }
  return fragment_output;
}