struct Transforms {
  view_matrix: mat4x4<f32>,
  model_matrix: mat4x4<f32>,
  model_view_matrix: mat4x4<f32>,
  clipping_rect: vec4<f32>,
}

struct Style {
  fill_color: vec4<f32>,
  stroke_color: vec4<f32>,
  stroke_width: f32,
}

struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) normal: vec2<f32>,
}

struct FragmentOutput {
  @location(0) color: vec4<f32>,
  @builtin(sample_mask) mask_out: u32,
}

struct FragmentInput {
  @builtin(position) position: vec4<f32>,
  @location(0) @interpolate(linear, center) normal: vec2<f32>,
}

// use different bind groups for different scopes
// e.g. bind_group 0 for world metadata and bind_group 1 for object metadata
@group(0) @binding(0)
var<uniform> transforms: Transforms;

@group(0) @binding(1)
var<uniform> style: Style;

@vertex
fn vs_fill(
  @location(0) pos: vec2<f32>
) -> @builtin(position) vec4<f32> {
  return transforms.model_view_matrix * vec4<f32>(pos, 0.0, 1.0);
}

@vertex
fn vs_stroke(vertex: VertexInput) -> FragmentInput {
  var delta = vec2<f32>(vertex.normal * style.stroke_width);
  var position = transforms.model_view_matrix * vec4<f32>(vertex.position + delta, 0.0, 1.0);
  return FragmentInput(position, vertex.normal);
}

fn clipping_and_premul_alpha(position: vec4<f32>, input_color: vec4<f32>) -> FragmentOutput {
  var color = input_color.a * vec4<f32>(input_color.rgb, 1.0); // pre-multiplied alpha
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

@fragment
fn fs_fill(@builtin(position) position: vec4<f32>) -> FragmentOutput {
  return clipping_and_premul_alpha(position, style.fill_color);
}

@fragment
fn fs_stroke(input: FragmentInput) -> FragmentOutput {
  // TODO: implement feather lines
  return clipping_and_premul_alpha(input.position, style.stroke_color);
}