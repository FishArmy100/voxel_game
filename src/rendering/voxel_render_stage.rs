use wgpu::ShaderStages;

use crate::camera::{Camera, CameraUniform};
use crate::voxel::{VoxelData, VoxelTerrain};

use super::VoxelFaceData;
use super::renderer::{RenderStage, DrawCall, BindGroupData};


pub struct VoxelRenderStage<'terrain, const S: usize, const N: usize>
{
    terrain: &'terrain VoxelTerrain<S, N>,

    camera_bind_group: BindGroupData,
    voxel_bind_group: BindGroupData,
    model_bind_group: BindGroupData
}

impl<'terrain, const S: usize, const N: usize> VoxelRenderStage<'terrain, S, N>
{
    pub fn new<'device>(terrain: &'terrain VoxelTerrain<S, N>, camera: Camera, device: &'device wgpu::Device) -> Self
    {
        let camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);
    }
}

impl<'terrain, const S: usize, const N: usize> RenderStage for VoxelRenderStage<'terrain, S, N>
{
    fn bind_groups(&self) -> &[super::renderer::BindGroupData] {
        todo!()
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        todo!()
    }

    fn get_draw_calls(&self) -> &[&dyn DrawCall] {
        todo!()
    }
}

pub struct VoxelDrawCall
{

}