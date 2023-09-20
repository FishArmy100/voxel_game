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
    @location(2) position: vec3<i32>,
    @location(3) id: u32,
    @location(4) face_index: u32,
    @location(5) scale: u32
};

struct CameraUniform {
    view_proj: mat4x4<f32>
}

struct ModelUniform {
    model: mat4x4<f32>
}

struct VoxelRenderData {
    color: vec4<f32>
}

struct VoxelRenderDataUniform {
    data: array<VoxelRenderData, 4>
}

struct VoxelSizeUniform {
    voxel_size: f32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@group(2) @binding(0)
var<uniform> render_data: VoxelRenderDataUniform;

@group(3) @binding(0)
var<uniform> voxel_size_uniform: VoxelSizeUniform;

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
    out.color = render_data.data[instance.id].color;

    var vert_pos = face_array.arr[instance.face_index][vertex.index];
    vert_pos *= f32(instance.scale);
    vert_pos.x += f32(instance.position.x);
    vert_pos.y += f32(instance.position.y);
    vert_pos.z += f32(instance.position.z);
    vert_pos *= voxel_size_uniform.voxel_size;

    out.clip_position = camera.view_proj * model.model * vec4<f32>(vert_pos, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
 