/*
 * Copyright (c) 2023, SkillerRaptor
 *
 * SPDX-License-Identifier: MIT
*/

#![no_std]

mod math;

use spirv_std::{
    glam::{UVec3, Vec3A, Vec4, Mat4, Vec3, Vec2},
    num_traits::Float,
    spirv, Image,
};

use math::{Ray, Intersectable};

#[derive(Clone, Copy)]
struct Camera
{
    eye: Vec3,
    target: Vec3,
    fov: f32,
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
        let a = ray.dir.dot(ray.dir);
        let b = 2.0 * ray.origin.dot(ray.dir);
        let c = ray.origin.dot(ray.origin) - self.radius * self.radius;

        let d = b * b - 4.0 * a * c;
        if d >= 0.0
        {
            let sqrt_d = d.sqrt();
            let t_plus = (-b + sqrt_d) / (2.0 * a);
            let t_minus = (-b - sqrt_d) / (2.0 * a);

            (t_plus >= 0.0) | (t_minus >= 0.0)
        }
        else 
        {
            false    
        }
    }
}

const SPHERE: Sphere = Sphere { radius: 1.0, center: Vec3::new(0.0, 0.0, -5.0), color: Vec4::new(1.0, 0.0, 0.0, 1.0) };

const BACKGROUND_COLOR: Vec4 = Vec4::new(0.5, 0.5, 0.5, 1.0);

fn create_ray(x: u32, y: u32, width: u32, height: u32, camera: Camera) -> Ray 
{
    let aspect = width as f32 / height as f32;
    let theta = camera.fov.to_radians();
    let half_height = (theta / 2.0).tan();
    let half_width = aspect * half_height;

    let w = (camera.eye - camera.target).normalize();
    let u = Vec3::Y.cross(w).normalize();
    let v = w.cross(u);

    let origin = camera.eye;
    let lower_left_corner = origin - (u * half_width) - (v * half_height) - w;
    let horizontal = u * 2.0 * half_width;
    let vertical = v * 2.0 * half_height;

    let x = x as f32 / width as f32;
    let y = y as f32 / height as f32;
    let dir = (lower_left_corner + (horizontal * x) + (vertical * y) - origin).normalize();

    Ray 
    { 
        origin: camera.eye, 
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
    #[spirv(uniform, descriptor_set = 0, binding = 4)] camera_target: &Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 5)] camera_fov: &f32,
) 
{
    let camera = Camera {
        eye: camera_eye.truncate(),
        target: camera_target.truncate(),
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
