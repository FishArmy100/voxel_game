// Vertex shader

struct VertexInput {
    @location(0) index: u32,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec4<f32>,
};

struct InstanceInput {
    @location(2) position: vec3<u32>,
    @location(3) voxel_id: u32,
    @location(4) face_index: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> voxel_size: f32;

@group(0) @binding(2) 
var<uniform> chunk_position: vec3<i32>;

@group(0) @binding(3)
var<storage> voxel_colors: array<vec4<f32>>;

const voxel_south_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 1.0),
);

const voxel_north_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(0.0, 0.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(1.0, 1.0, 0.0),
);

const voxel_up_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0),
);

const voxel_down_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0),
);

const voxel_east_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(1.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 0.0),
);

const voxel_west_face_position_array = array<vec3<f32>, 4>(
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(0.0, 0.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
);

const voxel_face_array = array<array<vec3<f32>, 4>, 6>(
    voxel_up_face_position_array,
    voxel_down_face_position_array,
    voxel_north_face_position_array,
    voxel_south_face_position_array,
    voxel_east_face_position_array,
    voxel_west_face_position_array
);

struct FaceArrayIndirect {
    arr: array<array<vec3<f32>, 4>, 6>
}

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var face_array: FaceArrayIndirect;
    face_array.arr = voxel_face_array;

    var out: VertexOutput;
    out.color = voxel_colors[instance.voxel_id];

    var vert_pos = face_array.arr[instance.face_index][vertex.index];
    vert_pos += vec3<f32>(instance.position) + vec3<f32>(chunk_position);
    vert_pos *= voxel_size;

    out.clip_position = camera.view_proj * vec4<f32>(vert_pos, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
 