#![no_std]

pub use glam;
pub use num_traits::Float;
use glam::{f32::Vec3, Vec3A};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ray 
{
    pub origin: Vec3,
    pub dir: Vec3
}

impl Ray 
{
    pub fn new(origin: Vec3, dir: Vec3) -> Self 
    {
        Self 
        { 
            origin, 
            dir
        }
    }

    pub fn from_points<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3> + Copy
    {
        Self 
        {
            origin: origin.into(),
            dir: origin.into() - destination.into()
        }
    }

    pub fn from_points_normalized<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3> + Copy
    {
        let dir: Vec3 = origin.into() - destination.into();
        Self 
        {
            origin: origin.into(),
            dir: dir.normalize()
        }
    }
}

pub trait Intersectable 
{
    fn intersect(&self, ray: &Ray) -> bool;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Camera
{
    pub eye: Vec3A,
    pub target: Vec3A,
    pub fov: f32,
}

impl Camera
{
    pub fn get_rt_info(&self, width: u32, height: u32) -> RTCameraInfo
    {
        let aspect = width as f32 / height as f32;
        let theta = self.fov.to_radians();
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;

        let w = (self.eye - self.target).normalize();
        let u = Vec3A::Y.cross(w).normalize();
        let v = w.cross(u);

        let origin = self.eye;
        let lower_left_corner = origin - (u * half_width) - (v * half_height) - w;
        let horizontal = u * 2.0 * half_width;
        let vertical = v * 2.0 * half_height;

        RTCameraInfo 
        { 
            eye: self.eye, 
            target: self.target, 
            horizontal, 
            vertical, 
            width, 
            height,
            lower_left_corner
        }
    }
}

#[derive(Copy, Clone)]
pub struct RTCameraInfo
{
    pub eye: Vec3A,
    pub target: Vec3A,
    pub horizontal: Vec3A,
    pub vertical: Vec3A,
    pub width: u32,
    pub height: u32,
    pub lower_left_corner: Vec3A,
}

impl RTCameraInfo
{
    pub fn get_ray(&self, x: u32, y: u32) -> Ray
    {
        let x = x as f32 / self.width as f32;
        let y = y as f32 / self.height as f32;
        let dir = (self.lower_left_corner + (self.horizontal * x) + (self.vertical * y) - self.eye).normalize();

        Ray 
        { 
            origin: self.eye.into(), 
            dir: dir.into()
        }
    }
}