// Vertex shader

struct VertexInput {
    @location(0) compressed_location: u32,
    @location(1) voxel_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) voxel_id: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>
}

struct ChunkUniform {
    position: vec3<i32>,
    size: u32
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
var<uniform> chunk: ChunkUniform;

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

fn get_byte(number: u32, offset: u32) -> u32
{
    (number >> (offset * 8)) & 255
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, vertex: VertexInput) -> VertexOutput 
{
    var face_array: FaceArrayIndirect;
    face_array.arr = voxel_face_array;

    var out: VertexOutput;
    out.voxel_id = vertex.voxel_id;

    let loc = vertex.compressed_location;

    let x = f32(i32(get_byte(loc, u32(0))) + chunk_uniform.position.x * i32(chunk_uniform.size));
    let y = f32(i32(get_byte(loc, u32(1))) + chunk_uniform.position.y * i32(chunk_uniform.size));
    let z = f32(i32(get_byte(loc, u32(2))) + chunk_uniform.position.z * i32(chunk_uniform.size));

    let face_index = get_byte(loc, u32(3));
    let vertex_index = in_vertex_index % u32(4);
    var vert_pos = face_array.arr[face_index][vertex_index];
    vert_pos += vec3f(x, y, z);
    vert_pos *= voxel_size_uniform.voxel_size;

    out.clip_position = camera.view_proj * vec4<f32>(vert_pos, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> 
{
    return render_data.data[in.voxel_id].color;
}
 
