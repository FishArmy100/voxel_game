/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

use vox_core::{Ray, Intersectable, RTCameraInfo};

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3, Vec2, BVec3, IVec3, uvec3},
    num_traits::Float,
    spirv, Image,
};

const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);

fn get_voxel(pos: Vec3) -> bool
{
    let pos = pos.floor();
    pos.y == 0.0
}

fn intersect_voxel(ray: Ray) -> Vec4
{
    const MAX_RAY_STEPS: u32 = 64;
    const VOXEL_COLOR: Vec4 = Vec4::new(0.1, 0.2, 0.3, 1.0);

    let mut map_pos = ray.origin.floor();
    let delta_dist = (ray.dir.length() / ray.dir).abs();

    
    let ray_step: Vec3 = {
        let v = ray.dir;
        let v: IVec3 = (uvec3(v.x.to_bits(), v.y.to_bits(), v.z.to_bits()).as_ivec3() >> 31) | 1;
        v.as_vec3()
    };

    let mut side_dist = (ray_step * (map_pos - ray.origin) + (ray_step * 0.5) + 0.5) * delta_dist;

    let mut mask = BVec3::FALSE;
    let mut found = false;
    for _ in 0..MAX_RAY_STEPS
    {
        found |= get_voxel(map_pos);
        
        let yzx = Vec3::new(side_dist.y, side_dist.z, side_dist.x);
        let zxy = Vec3::new(side_dist.z, side_dist.x, side_dist.y);
        mask = side_dist.cmple(yzx.min(zxy));

        let v_mask = Vec3::new(mask.x as u32 as f32, mask.y as u32 as f32, mask.z as u32 as f32);
        side_dist += v_mask * delta_dist;
        map_pos += v_mask * ray_step;
    }

    if found
    {
        VOXEL_COLOR
    }
    else 
    {
        BACKGROUND_COLOR    
    }
}

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, format = rgba8, sampled = false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] camera: &RTCameraInfo,
) 
{
    let ray = camera.get_ray(id.x, id.y);
    let color = intersect_voxel(ray);

    unsafe 
    { 
        texture.write(id.truncate(), color)
    }
}
