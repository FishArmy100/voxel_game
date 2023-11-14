use spirv_std::glam::Vec3A;

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