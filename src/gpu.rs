use crate::{utils::Byteable, math::Vec3};


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

impl<T> Into<GPUVec3<T>> for Vec3<T> where T : Byteable
{
    fn into(self) -> GPUVec3<T> 
    {
        GPUVec3::new(self.x, self.y, self.z)
    }
}