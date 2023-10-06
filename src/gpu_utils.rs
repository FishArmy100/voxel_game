pub mod bind_group;
pub mod buffer;

use std::borrow::Cow;
use crate::colors::Color;
use crate::{utils::Byteable, math::Vec3};

pub use self::bind_group::*;
pub use self::buffer::*;

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
#[derive(Debug, Clone, Copy)]
pub struct GPUColor
{
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

unsafe impl bytemuck::Pod for GPUColor {}
unsafe impl bytemuck::Zeroable for GPUColor {}

impl GPUColor
{
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self
    {
        Self 
        { 
            r, 
            g, 
            b, 
            a 
        }
    }
}

impl From<Color> for GPUColor
{
    fn from(value: Color) -> Self
    {
        Self 
        { 
            r: value.r, 
            g: value.g, 
            b: value.b, 
            a: value.a 
        }
    }
}

pub struct ShaderInfo<'a>
{
    pub entry_point: &'a str,
    pub source: &'a str
}

impl<'a> ShaderInfo<'a>
{
    pub fn generate_shader(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::ShaderModule
    {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(self.source)),
        })
    }
}