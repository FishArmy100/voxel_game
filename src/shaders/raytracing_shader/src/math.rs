use spirv_std::glam::{Vec3A, Vec3};

const PI: f32 = 3.14159265358979323846;

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