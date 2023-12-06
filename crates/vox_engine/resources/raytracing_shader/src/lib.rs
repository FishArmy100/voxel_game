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

const TEST_SPHERE: Sphere = Sphere {
    center: Vec3::ZERO,
    radius: 2.0
};

pub struct Sphere
{
    pub center: Vec3,
    pub radius: f32
}

impl Intersectable for Sphere
{
    fn intersect(&self, ray: &Ray) -> bool 
    {
        const T_MIN: f32 = 1.0;
        const T_MAX: f32 = 1000.0;

        let oc = ray.origin - self.center;
        let a = ray.dir.length_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;

        let disc = half_b * half_b - a * c;
        if disc < 0.0 {
            return false;
        }
        let sqrtd = disc.sqrt();

        let mut root = (-half_b - sqrtd) / a;

        if root < T_MIN || T_MAX < root {
            root = (-half_b + sqrtd) / a;
            if root < T_MIN || T_MAX < root {
                return false;
            }
        }

        true
    }
}

const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);

fn get_voxel(pos: Vec3) -> bool
{
    let pos = pos.floor();
    pos.x == 0.0 && pos.y == 0.0 && pos.z == 0.0
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

    let mut found = false;
    let mut i = 0;
    loop
    {
        if i == MAX_RAY_STEPS { break; }
        
        i += 1;

        found = found | get_voxel(map_pos);

        let mask_x = if side_dist.x < side_dist.y.min(side_dist.z) { 1.0 } else { 0.0 };
        let mask_y = if side_dist.y < side_dist.z.min(side_dist.x) { 1.0 } else { 0.0 };
        let mask_z = if side_dist.z < side_dist.x.min(side_dist.y) { 1.0 } else { 0.0 };
        let mask = Vec3::new(mask_x, mask_y, mask_z);

        side_dist += mask * delta_dist;
        map_pos += mask * ray_step;
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

    // let color = if TEST_SPHERE.intersect(&ray)
    // {
    //     Vec4::new(0.1, 0.2, 0.3, 1.0)
    // }
    // else 
    // {
    //     BACKGROUND_COLOR
    // };

    unsafe 
    { 
        texture.write(id.truncate(), color)
    }
}
