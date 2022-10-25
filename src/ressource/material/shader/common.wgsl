struct Tile {
  model_view_matrix: mat4x4<f32>,
  clipping_rect: vec4<f32>,
}

struct View {
  view_matrix: mat4x4<f32>,
  width: u32,
  height: u32
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

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var<uniform> style: Style;

@group(2) @binding(0)
var<uniform> tile: Tile;

@vertex
fn vs_fill(
  @location(0) pos: vec2<f32>
) -> @builtin(position) vec4<f32> {
  return tile.model_view_matrix * vec4<f32>(pos, 0.0, 1.0);
}

@vertex
fn vs_stroke(vertex: VertexInput) -> FragmentInput {
  var delta = vec2<f32>(vertex.normal * style.stroke_width);
  var position = tile.model_view_matrix * vec4<f32>(vertex.position + delta, 0.0, 1.0);
  return FragmentInput(position, vertex.normal);
}

fn clipping_and_premul_alpha(position: vec4<f32>, input_color: vec4<f32>) -> FragmentOutput {
  var color = input_color.a * vec4<f32>(input_color.rgb, 1.0); // pre-multiplied alpha
  var fragment_output = FragmentOutput(color, 0xFFFFFFFFu);

  if (
    position.x < tile.clipping_rect[0] ||
    position.y < tile.clipping_rect[1] ||
    position.x > tile.clipping_rect[2] ||
    position.y > tile.clipping_rect[3]
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