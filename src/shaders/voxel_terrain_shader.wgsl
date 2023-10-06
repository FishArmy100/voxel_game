struct VoxelVertex
{
    @location(0) face_index: u32,
}

struct OutVertex
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color_index: u32
}

struct VoxelFace
{
    pos_x: u32,
    pos_y: u32,
    pos_z: u32,
    direction: u32,
    voxel_id: u32,
}

struct CameraUniform 
{
    view_proj: mat4x4<f32>
}

struct FaceArrayIndirect 
{
    arr: array<array<vec3<f32>, 4>, 6>
}

struct IndexArrayWrapper
{
    arr: array<i32, 6>
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> voxel_size: f32;

@group(0) @binding(2)
var<uniform> chunk_position: vec3<i32>;

@group(0) @binding(3)
var<storage> voxel_colors: array<vec4<f32>>;

@group(1) @binding(0)
var<storage> faces: array<VoxelFace>;

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

const vertex_index_array = array<i32, 6>(
    2,
    1,
    0,
    2,
    3,
    1,
);

@vertex
fn vs_main(@builtin(vertex_index) index: u32, vertex: VoxelVertex) -> OutVertex
{
    var face_array: FaceArrayIndirect;
    face_array.arr = voxel_face_array;

    var index_array: IndexArrayWrapper;
    index_array.arr = vertex_index_array;

    let vertex_index = u32(index_array.arr[index % u32(6)]);
    let face = faces[vertex.face_index];

    
    var vertex_pos = face_array.arr[face.direction][vertex_index] + 
                     vec3<f32>(f32(face.pos_x), f32(face.pos_y), f32(face.pos_z)) + 
                     vec3<f32>(chunk_position);
                     
    vertex_pos *= voxel_size;

    var out: OutVertex;
    out.clip_position = camera.view_proj * vec4<f32>(vertex_pos, 1.0);
    out.color_index = face.voxel_id;
    return out;
}

@fragment
fn fs_main(vertex: OutVertex) -> @location(0) vec4<f32>
{
    return voxel_colors[vertex.color_index];
}
