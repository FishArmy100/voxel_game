pub mod octree;
pub mod terrain;
pub mod world_gen;
pub mod brick_map;

use std::sync::Arc;
use crate::colors::Color;
use crate::math::{Vec3, Point3D};
use crate::rendering::voxel_render_stage::{VoxelFaceData, VoxelRenderData};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelData
{
    color: Color,
}

impl VoxelData
{
    pub fn new(color: Color) -> Self
    {
        Self { color }
    }

    pub fn get_render_data(&self) -> VoxelRenderData
    {
        VoxelRenderData::new(self.color)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voxel 
{
    id: u16
}

impl Voxel
{
    pub fn new(id: u16) -> Self
    {
        Self { id }
    }
}