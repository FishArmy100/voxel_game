#![no_std]

pub use glam;
use glam::f32::Vec3;

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
}

pub trait Intersectable 
{
    fn intersect(&self, ray: &Ray) -> bool;
}