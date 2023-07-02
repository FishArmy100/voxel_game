use crate::math::vectors::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color 
{
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color
{
    pub fn rgb_vec(&self) -> Vec3<f32>
    {
        Vec3::new(self.r, self.g, self.b)
    }

    pub fn rgb_arr(&self) -> [f32; 3]
    {
        [self.r, self.g, self.b]
    }

    pub fn rgba_arr(&self) -> [f32; 4]
    {
        [self.r, self.g, self.b, self.a]
    }
}