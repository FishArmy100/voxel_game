// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>
}

struct ModelUniform {
    model: mat4x4<f32>
}

@group(0) @binding(0) // 1.
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, obj: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = obj.color;
    out.clip_position = camera.view_proj * model.model * vec4<f32>(obj.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color);
}
 