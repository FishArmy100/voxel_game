struct VertexInput {
    @location(0) face_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) voxel_id: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>
}

struct ChunkUniform {
    position: vec3<i32>,
    size: u32,
    voxel_size: f32
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

struct FaceData {
    position_x: u32,
    position_y: u32,
    position_z: u32,
    orientation: u32,
    voxel_id: u32
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> chunk: ChunkUniform;

@group(2) @binding(0)
var<uniform> render_data: VoxelRenderDataUniform;

@group(3) @binding(0)
var<storage, read> faces: array<FaceData>;

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
fn vs_main(@builtin(vertex_index) index: u32, vertex: VertexInput) -> VertexOutput {
    var face_array: FaceArrayIndirect;
    face_array.arr = voxel_face_array;

    let face_vertex_index = index % u32(4);
    let face_data = faces[vertex.face_id];

    var vert_pos = face_array.arr[face_data.orientation][index];

    let face_position = vec3f(f32(face_data.position_x), f32(face_data.position_y), f32(face_data.position_z));
    vert_pos += face_position; // + vec3f(chunk.position) * f32(chunk.size);

    vert_pos += vec3f(f32(index / u32(4)), 0.0, 0.0);

    vert_pos *= 1.0;

    
    var out: VertexOutput;
    out.voxel_id = face_data.voxel_id;
    out.clip_position = camera.view_proj * vec4<f32>(vert_pos, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return render_data.data[in.voxel_id].color;
}
 