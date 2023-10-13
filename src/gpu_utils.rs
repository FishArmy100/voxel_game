pub mod bind_group;
pub mod buffer;
pub mod texture;

use std::borrow::Cow;
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

pub enum ShaderSource<'a>
{
    WGSL(&'a str),
    SpirV(&'a [u8])
}

pub struct ShaderInfo<'a>
{
    pub entry_point: &'a str,
    pub source: ShaderSource<'a>
}

impl<'a> ShaderInfo<'a>
{
    pub fn generate_shader(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::ShaderModule
    {
        match &self.source
        {
            ShaderSource::WGSL(src) => 
            {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(src)),
                })
            },
            ShaderSource::SpirV(bytes) => 
            {
                let spirv = wgpu::util::make_spirv_raw(bytes);

                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label,
                    source: wgpu::ShaderSource::SpirV(spirv),
                })
            },
        }

        
    }
}