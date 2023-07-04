use crate::math::*;

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
    pub const RED: Color = Color   { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color  { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const CLEAR: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.0 };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color
    {
        Color { r, g, b, a }
    }

    pub fn rgb(self) -> Vec3<f32>
    {
        Vec3::new(self.r, self.g, self.b)
    }

    pub fn rgba(self) -> Vec4<f32>
    {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}