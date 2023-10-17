#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{Vec3, vec3, Vec4, vec4, Mat4, mat4, UVec3, uvec3, IVec3, IVec2, IVec4};

const SOUTH_FACE: [Vec3; 4] = [   
    vec3(0.0, 1.0, 1.0),
    vec3(1.0, 1.0, 1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 1.0),
];

const NORTH_FACE: [Vec3; 4] = [
    vec3(0.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 1.0, 0.0),
];

const UP_FACE: [Vec3; 4] = [
    vec3(0.0, 1.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0),
    vec3(1.0, 1.0, 1.0),
];

const DOWN_FACE: [Vec3; 4] = [
    vec3(0.0, 0.0, 1.0),
    vec3(1.0, 0.0, 1.0),
    vec3(0.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0),
];

const EAST_FACE: [Vec3; 4] = [
    vec3(1.0, 1.0, 1.0),
    vec3(1.0, 1.0, 0.0),
    vec3(1.0, 0.0, 1.0),
    vec3(1.0, 0.0, 0.0),
];

const WEST_FACE: [Vec3; 4] = [
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 1.0, 1.0),
    vec3(0.0, 0.0, 0.0),
    vec3(0.0, 0.0, 1.0),
];

const VOXEL_FACE_ARRAY: [[Vec3; 4]; 6] = [
    UP_FACE,
    DOWN_FACE,
    NORTH_FACE,
    SOUTH_FACE,
    EAST_FACE,
    WEST_FACE
];

#[spirv(vertex)]
pub fn vs_main(
    // vertex
    index: u32,
    _color: Vec4,

    // instance
    voxel_position: UVec3,
    voxel_id: u32,
    face_index: u32,
    
    #[spirv(position)] clip_position: &mut Vec4,

    #[spirv(uniform, descriptor_set = 0, binding = 0)] view_proj: &Mat4,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] voxel_size: &f32,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] chunk_position: &IVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] voxel_colors: &[Vec4; 4],


    color_out: &mut Vec4
) 
{
    *color_out = voxel_colors[voxel_id as usize];
    
    let mut vert_pos = VOXEL_FACE_ARRAY[face_index as usize][index as usize];
    vert_pos += voxel_position.as_vec3() + chunk_position.as_vec3();
    vert_pos *= *voxel_size;

    *clip_position = *view_proj * vec4(vert_pos.x, vert_pos.y, vert_pos.z, 1.0);
}

#[spirv(fragment)]
pub fn fs_main(color: Vec4, output: &mut Vec4)
{
    *output = color;
}
