/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

mod math;

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4},
    num_traits::Float,
    spirv, Image,
};

use math::{Ray, Intersectable};

#[derive(Clone, Copy)]
struct Camera
{
    eye: Vec3A,
    lower_left: Vec3A,
    horizontal: Vec3A,
    vertical: Vec3A,
    fov: f32,
}

struct Sphere
{
    radius: f32,
    center: Vec3A,
    color: Vec4
}

impl Intersectable for Sphere
{
    fn intersect(&self, ray: &Ray) -> bool 
    {
        let line = self.center - ray.origin;
        let adj2 = line.dot(ray.dir);
        let d2 = line.dot(line) - (adj2 * adj2);
        d2 < (self.radius * self.radius)
    }
}

const SPHERE: Sphere = Sphere { radius: 1.0, center: Vec3A::new(0.0, 0.0, -5.0), color: Vec4::new(0.4, 1.0, 0.4, 1.0) };
const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);

fn create_ray(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray 
{
    let u = x as f32 / width as f32;
    let v = y as f32 / height as f32;

    let origin = camera.eye;
    let dir = camera.lower_left + (camera.horizontal * u) + (camera.vertical * v) - origin;

    Ray 
    { 
        origin, 
        dir
    }
}

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, format = rgba8, sampled = false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] width_buffer: &u32,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] height_buffer: &u32,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] camera_eye: &Vec3A,
    #[spirv(uniform, descriptor_set = 0, binding = 4)] camera_lower_left: &Vec3A,
    #[spirv(uniform, descriptor_set = 0, binding = 5)] camera_horizontal: &Vec3A,
    #[spirv(uniform, descriptor_set = 0, binding = 6)] camera_vertical: &Vec3A,
    #[spirv(uniform, descriptor_set = 0, binding = 7)] camera_fov: &f32,
) 
{
    let camera = Camera {
        eye: *camera_eye,
        lower_left: *camera_lower_left,
        horizontal: *camera_horizontal,
        vertical: *camera_vertical,
        fov: *camera_fov
    };

    let ray = create_ray(id.x, id.y, *width_buffer, *height_buffer, camera);
    let color = if SPHERE.intersect(&ray)
    { 
        SPHERE.color 
    }
    else 
    {
        BACKGROUND_COLOR
    };

    unsafe 
    { 
        texture.write(id.truncate(), color)
    }
}
