#![no_std]

pub mod camera;

pub use glam;
pub use num_traits::Float;
use glam::f32::Vec3A;


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ray 
{
    pub origin: Vec3A,
    pub dir: Vec3A
}

impl Ray 
{
    pub fn new<T>(origin: T, dir: T) -> Self 
        where T : Into<Vec3A>
    {
        Self 
        { 
            origin: origin.into(), 
            dir: dir.into(),
        }
    }

    pub fn from_points<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        Self 
        {
            origin: origin.into(),
            dir: origin.into() - destination.into()
        }
    }

    pub fn from_points_normalized<T>(origin: T, destination: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        let dir: Vec3A = origin.into() - destination.into();
        Self 
        {
            origin: origin.into(),
            dir: dir.normalize()
        }
    }
}

#[derive(Clone, Copy)]
pub struct HitInfo 
{
    pub hit: bool,
    pub hit_pos: Vec3A
}

pub trait Intersectable 
{
    fn intersect(&self, ray: &Ray) -> HitInfo;
}

#[derive(Clone, Copy)]
pub struct AABB
{
    pub min: Vec3A,
    pub max: Vec3A,
}

impl AABB 
{
    pub fn new<T>(min: T, max: T) -> Self
        where T : Into<Vec3A>
    {
        Self
        {
            min: min.into(),
            max: max.into(),
        }
    }

    pub fn from_extents<T>(pos: T, extents: T) -> Self
        where T : Into<Vec3A> + Copy
    {
        let min = pos.into() - extents.into();
        let max = pos.into() + extents.into();

        Self 
        {
            min,
            max,
        }
    }
}

impl Intersectable for AABB
{
    fn intersect(&self, ray: &Ray) -> HitInfo 
    {
        let t_min = (self.min - ray.origin) / ray.dir;
        let t_max = (self.max - ray.origin) / ray.dir;

        let t1 = t_min.min(t_max);
        let t2 = t_min.max(t_max);
        let near = t1.max_element();
        let far = t2.min_element();

        HitInfo 
        { 
            hit: !(near > far) && far >= 0.0, 
            hit_pos: Vec3A::ZERO 
        }
    }
}
