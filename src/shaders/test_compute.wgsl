@group(0) @binding(0)
var<storage, read_write> v_indices: array<f32>; // this is used as both input and output for convenience

fn rand(n: f32) -> f32 { return fract(sin(n) * 43758.5453123); }

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    v_indices[global_id.x] = rand(f32(global_id.x));
}