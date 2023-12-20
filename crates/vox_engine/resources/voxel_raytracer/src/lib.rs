/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]
use vox_core::{Ray, Intersectable, camera::RTCameraInfo, AABB, glam::vec4, HitInfo, VoxelModelInstance};

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3, Vec2, BVec3, IVec3, uvec3},
    num_traits::Float,
    spirv, Image, image::Image2d, Sampler,
};

const VOXEL_COLORS: [Vec4; 4] = 
[
    vec4(1.0, 0.0, 0.0, 1.0), 
    vec4(0.0, 1.0, 0.0, 1.0), 
    vec4(0.0, 0.0, 1.0, 1.0),
    vec4(1.0, 0.0, 1.0, 1.0) // ERROR
];

const BACKGROUND_COLOR: Vec4 = vec4(0.5, 0.5, 0.5, 1.0);

#[spirv(vertex)]
pub fn vs_main(
    out_uv: &mut Vec2,
    #[spirv(vertex_index)] vertex_id: i32,
    #[spirv(position)] position: &mut Vec4,
) 
{
    let x = (((vertex_id as u32 + 2) / 3) % 2) as f32;
    let y = (((vertex_id as u32 + 1) / 3) % 2) as f32;

    *out_uv = Vec2::new(x, y);
    *position = Vec4::new(-1.0 + x * 2.0, -1.0 + y * 2.0, 0.0, 1.0);
}

#[spirv(fragment)]
pub fn fs_main(
    uv: Vec2,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] camera: &RTCameraInfo,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] instances: &[VoxelModelInstance],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] voxels: &[u32],
    output: &mut Vec4,
) 
{
    let x = (uv.x * camera.width as f32) as u32;
    let y = (uv.y * camera.height as f32) as u32;

    let ray = camera.get_ray(x, y);

    let mut closest = (false, f32::infinity(), 0);
    for i in 0..instances.len()
    {
        let hit = instances[i].intersect(&ray, voxels);
        if hit.hit && hit.distance < closest.1
        {
            closest = (true, hit.distance, hit.value)
        }
    }

    if closest.0
    {
        *output = VOXEL_COLORS[closest.2 as usize];
    }
    else 
    {
        *output = BACKGROUND_COLOR;
    }
}
