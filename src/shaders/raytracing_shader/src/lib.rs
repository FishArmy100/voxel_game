/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

mod math;

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3},
    num_traits::Float,
    spirv, Image,
};

use math::{Ray, Intersectable};

#[derive(Clone, Copy)]
struct Camera
{
    eye: Vec3,
    lower_left: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

struct Sphere
{
    radius: f32,
    center: Vec3,
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

const SPHERE: Sphere = Sphere { radius: 1.0, center: Vec3::new(0.0, 0.0, -5.0), color: Vec4::new(0.4, 1.0, 0.4, 1.0) };
const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);
const FOV: f32 = 60.0;

fn create_ray(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray 
{
    let u = x as f32 / (width as f32 - 1.0);
    let v = y as f32 / (height as f32 - 1.0);

    let origin = camera.eye;
    let dir = (camera.lower_left + (camera.horizontal * u) + (camera.vertical * v) - origin).normalize();

    Ray 
    { 
        origin, 
        dir
    }
}

fn create_working_ray(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray
{
    let fov_adjustment = (FOV.to_radians() / 2.0).tan();
    let aspect_ratio = (width as f32) / (height as f32);
    let sensor_x = ((((x as f32 + 0.5) / width as f32) * 2.0 - 1.0) * aspect_ratio) * fov_adjustment;
    let sensor_y = (1.0 - ((y as f32 + 0.5) / height as f32) * 2.0) * fov_adjustment;

    let dir = Vec3::new(sensor_x, sensor_y, -1.0).normalize();
    Ray 
    {
        origin: Vec3::ZERO,
        dir
    }
}

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(descriptor_set = 0, binding = 0)] texture: &Image!(2D, format = rgba8, sampled = false),
    #[spirv(uniform, descriptor_set = 0, binding = 1)] width_buffer: &u32,
    #[spirv(uniform, descriptor_set = 0, binding = 2)] height_buffer: &u32,
    #[spirv(uniform, descriptor_set = 0, binding = 3)] camera_eye: &Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 4)] camera_lower_left: &Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 5)] camera_horizontal: &Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 6)] camera_vertical: &Vec4,
) 
{
    let camera = Camera {
        eye: camera_eye.truncate(),
        lower_left: camera_lower_left.truncate(),
        horizontal: camera_horizontal.truncate(),
        vertical: camera_vertical.truncate(),
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
