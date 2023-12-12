/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

use vox_core::{Ray, Intersectable, camera::RTCameraInfo, AABB, glam::vec4, HitInfo};

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3, Vec2, BVec3, IVec3, uvec3},
    num_traits::Float,
    spirv, Image, image::Image2d, Sampler,
};

const TEST_SPHERE: Sphere = Sphere {
    center: Vec3A::ZERO,
    radius: 2.0
};

pub struct Sphere
{
    pub center: Vec3A,
    pub radius: f32
}

impl Intersectable for Sphere
{
    fn intersect(&self, ray: &Ray) -> HitInfo 
    {
        const T_MIN: f32 = 1.0;
        const T_MAX: f32 = 1000.0;
        let oc = ray.origin - self.center;
        let a = ray.dir.length_squared();
        let half_b = oc.dot(ray.dir);
        let c = oc.length_squared() - self.radius * self.radius;
        let disc = half_b * half_b - a * c;
        if disc < 0.0 {
            return HitInfo {
                hit: false,
                hit_pos: Vec3A::ZERO
            };
        }
        let sqrtd = disc.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < T_MIN || T_MAX < root {
            root = (-half_b + sqrtd) / a;
            if root < T_MIN || T_MAX < root {
                return HitInfo {
                    hit: false,
                    hit_pos: Vec3A::ZERO
                };
            }
        }

        HitInfo {
            hit: true,
            hit_pos: Vec3A::ZERO
        }
    }
}

const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);

fn get_voxel(pos: Vec3A) -> bool
{
    let pos = pos.floor();
    (pos.x == 0.0) & (pos.y == 0.0) & (pos.z == 0.0)
}

fn intersect_voxel(ray: Ray) -> Vec4
{
    const MAX_RAY_STEPS: u32 = 256;
    const VOXEL_COLOR: Vec4 = Vec4::new(0.1, 0.2, 0.3, 1.0);

    let mut map_pos = ray.origin.floor();
    let delta_dist = (ray.dir.length() / ray.dir).abs();

    
    let ray_step = ray.dir.signum();

    let mut side_dist = (ray_step * (map_pos - ray.origin) + (ray_step * 0.5) + 0.5) * delta_dist;

    let mut found = false;
    let mut i = 0;
    loop
    {
        if (i == MAX_RAY_STEPS) | found { break; }
        
        i += 1;

        found = found | get_voxel(map_pos);

        let mask_x = if side_dist.x < side_dist.y.min(side_dist.z) { 1.0 } else { 0.0 };
        let mask_y = if side_dist.y < side_dist.z.min(side_dist.x) { 1.0 } else { 0.0 };
        let mask_z = if side_dist.z < side_dist.x.min(side_dist.y) { 1.0 } else { 0.0 };
        let mask = Vec3A::new(mask_x, mask_y, mask_z);

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
    output: &mut Vec4,
) 
{
    let x = (uv.x * camera.width as f32) as u32;
    let y = (uv.y * camera.height as f32) as u32;

    let ray = camera.get_ray(x, y);

    let b = AABB::from_extents(Vec3A::ZERO, Vec3A::ONE * 2.0);
    if b.intersect(&ray).hit
    {
        *output = vec4(0.1, 0.2, 0.3, 1.0);
    }
    else 
    {
        *output = vec4(0.5, 0.5, 0.5, 1.0);
    }

    // *output = intersect_voxel(ray);

    // if TEST_SPHERE.intersect(&ray).hit
    // {
    //     *output = vec4(0.1, 0.2, 0.3, 1.0);
    // }
    // else 
    // {
    //     *output = vec4(0.5, 0.5, 0.5, 1.0);
    // }
}
