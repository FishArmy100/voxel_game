pub mod bind_group;
pub mod buffer;
pub mod texture;
use crate::math::Vec4;
use crate::{utils::Byteable, math::Vec3};

pub use self::bind_group::*;
pub use self::buffer::*;
pub use self::texture::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPUVec3<T> where T : Byteable
{
    pub x: T,
    pub y: T,
    pub z: T
}

unsafe impl<T> bytemuck::Pod for GPUVec3<T> where T : Byteable {}
unsafe impl<T> bytemuck::Zeroable for GPUVec3<T> where T : Byteable {}

impl<T> GPUVec3<T> where T : Byteable 
{
    pub fn new(x: T, y: T, z: T) -> Self
    {
        Self 
        { 
            x, 
            y, 
            z 
        }
    }
}

impl<T> From<Vec3<T>> for GPUVec3<T> where T : Byteable
{
    fn from(value: Vec3<T>) -> Self 
    {
        GPUVec3::new(value.x, value.y, value.z)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GPUVec4<T> where T : Byteable
{
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T
}

unsafe impl<T> bytemuck::Pod for GPUVec4<T> where T : Byteable {}
unsafe impl<T> bytemuck::Zeroable for GPUVec4<T> where T : Byteable {}

impl<T> GPUVec4<T> where T : Byteable 
{
    pub fn new(x: T, y: T, z: T, w: T) -> Self
    {
        Self 
        { 
            x, 
            y, 
            z,
            w
        }
    }
}

impl<T> From<Vec4<T>> for GPUVec4<T> where T : Byteable
{
    fn from(value: Vec4<T>) -> Self 
    {
        GPUVec4::new(value.x, value.y, value.z, value.w)
    }
}