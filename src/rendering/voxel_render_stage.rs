use crate::camera::{Camera, CameraUniform};
use crate::voxel::{VoxelData, VoxelTerrain};

use super::{VoxelFaceData, VoxelRenderDataUniform, ModelUniform};
use super::renderer::{RenderStage, DrawCall, BindGroupData};


pub struct VoxelRenderStage<'terrain, const S: usize, const N: usize>
{
    terrain: &'terrain VoxelTerrain<S, N>,
    bind_groups: [BindGroupData; 3]
}

impl<'terrain, const S: usize, const N: usize> VoxelRenderStage<'terrain, S, N>
{
    pub fn new<'device>(terrain: &'terrain VoxelTerrain<S, N>, camera: Camera, device: &'device wgpu::Device) -> Self
    {
        let camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let voxel_uniform = VoxelRenderDataUniform::new(terrain.voxel_types().map(|v| v.get_render_data()).clone());
        let voxel_bind_group = BindGroupData::uniform("voxel_bind_group".into(), voxel_uniform, wgpu::ShaderStages::VERTEX, device);

        let model_uniform = ModelUniform::from_position(terrain.position());
        let model_bind_group = BindGroupData::uniform("model_bind_group".into(), model_uniform, wgpu::ShaderStages::VERTEX, device);
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