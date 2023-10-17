#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{Vec3, vec3, Vec4, vec4, Mat4, mat4, UVec3, uvec3};

pub struct CameraUniform
{
    view_proj: Mat4
}

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
pub fn vs_main() 
{
    
}

#[spirv(fragment)]
pub fn fs_main()
{

}
