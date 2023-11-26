
pub type Vec2<T> = cgmath::Vector2<T>;
pub type Vec3<T> = cgmath::Vector3<T>;
pub type Vec4<T> = cgmath::Vector4<T>;

pub type Point3D<T> = cgmath::Point3<T>;
pub type Point2D<T> = cgmath::Point2<T>;

pub type Mat4x4<T> = cgmath::Matrix4<T>;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Color 
{
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

unsafe impl bytemuck::Pod for Color {}
unsafe impl bytemuck::Zeroable for Color {}

impl Color
{
    pub const RED: Color = Color   { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color  { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
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

    pub fn to_wgpu(self) -> wgpu::Color
    {
        wgpu::Color { r: self.r as f64, g: self.g as f64, b: self.b as f64, a: self.a as f64 }
    }
}
