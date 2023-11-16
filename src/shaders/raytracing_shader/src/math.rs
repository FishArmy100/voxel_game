use spirv_std::glam::Vec3A;

const PI: f32 = 3.14159265358979323846;

#[derive(Clone, Copy)]
pub struct Ray 
{
    pub origin: Vec3A,
    pub dir: Vec3A
}

impl Ray 
{
    pub fn new(origin: Vec3A, dir: Vec3A) -> Self 
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