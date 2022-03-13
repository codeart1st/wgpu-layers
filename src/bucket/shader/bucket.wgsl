[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec2<f32>
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(pos, 1.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}