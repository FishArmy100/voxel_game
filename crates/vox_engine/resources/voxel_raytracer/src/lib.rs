#![no_std]
use vox_core::{Ray, Intersectable, camera::RTCameraInfo, AABB, glam::vec4, HitInfo, VoxelModelInstance, VoxelModelHit};

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3, Vec2, BVec3, IVec3, uvec3},
    num_traits::Float,
    spirv, Image, image::Image2d, Sampler, arch::IndexUnchecked,
};

const BACKGROUND_COLOR: Vec4 = vec4(0.0, 0.0, 0.0, 1.0);

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
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] voxel_colors: &[Vec4],
    output: &mut Vec4,
) 
{
    let x = (uv.x * camera.width as f32) as u32;
    let y = (uv.y * camera.height as f32) as u32;

    let ray = camera.get_ray(x, y);

    let mut closest = VoxelModelHit {
        hit: false,
        value: 0,
        distance: f32::infinity(),
        normal: Vec3A::ZERO
    };

    for i in 0..instances.len()
    {
        let hit = instances[i].intersect(&ray, voxels);
        if hit.hit && hit.distance < closest.distance
        {
            closest = hit;
        }
    }

    if closest.hit
    {
        let mut modifier = 1.0;
        if closest.normal.x != 0.0 {
            modifier = 0.5;
        }
        if closest.normal.y != 0.0 {
            modifier = 1.0;
        }
        if closest.normal.z != 0.0 {
            modifier = 0.75;
        }

        let mut color = voxel_colors[closest.value as usize];
        color *= modifier;
        *output = color;
    }
    else 
    {
        *output = BACKGROUND_COLOR;
    }
}
