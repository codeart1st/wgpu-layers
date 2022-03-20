struct World {
  view_matrix: mat3x3<f32>;
};

[[group(0), binding(0)]]
var<uniform> world: World;

[[stage(vertex)]]
fn vs_main(
  [[location(0)]] pos: vec2<f32>
) -> [[builtin(position)]] vec4<f32> {
  return vec4<f32>((world.view_matrix * vec3<f32>(pos, 1.0)), 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
  return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}