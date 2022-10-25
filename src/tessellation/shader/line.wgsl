struct OutputVertex {
  position: vec2<f32>,
  normal: vec2<f32>,
}

@group(0) @binding(0)
var<storage, read> vertices : array<vec2<f32>>;

@group(0) @binding(1)
var<storage, read> indices : array<u32>;

@group(0) @binding(2)
var<storage, read_write> line_vertices : array<OutputVertex>;

@group(0) @binding(3)
var<storage, read_write> line_indices : array<u32>;

@compute @workgroup_size(256, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  if (global_id.x >= arrayLength(&indices) - 1u) {
    return;
  }

  let i1 = indices[global_id.x];
  let i2 = indices[global_id.x + 1u];

  let v1 = vertices[i1];
  let v2 = vertices[i2];

  let dx = v2.x - v1.x;
  let dy = v2.y - v1.y;
  let n1 = normalize(vec2<f32>(-dy, dx));
  let n2 = normalize(vec2<f32>(dy, -dx));

  let ii1 = global_id.x * 4u;
  let ii2 = ii1 + 1u;
  let ii3 = ii1 + 2u;
  let ii4 = ii1 + 3u;

  line_vertices[ii1] = OutputVertex(v1, n1);
  line_vertices[ii2] = OutputVertex(v1, n2);
  line_vertices[ii3] = OutputVertex(v2, n1);
  line_vertices[ii4] = OutputVertex(v2, n2);

  line_indices[ii1] = ii1;
  line_indices[ii2] = ii2;
  line_indices[ii3] = ii3;
  line_indices[ii4] = ii4;
}