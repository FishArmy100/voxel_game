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
    pub eye: Vec3,
    pub target: Vec3,
    pub fov: f32,
    pub aspect: f32
}

pub struct RTCameraInfo
{
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub width: u32,
    pub height: u32,
}