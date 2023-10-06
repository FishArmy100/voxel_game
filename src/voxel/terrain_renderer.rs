use std::{sync::Arc, cell::RefCell};

use crate::{math::Vec3, rendering::{VertexData, VertexBuffer, construct_render_pipeline, RenderPipelineInfo, DrawCall, IVertexBuffer, RenderStage}, camera::{Camera, CameraUniform}};
use crate::gpu_utils::{Storage, BindGroup, Uniform, GPUVec3, Entry};
use crate::voxel::voxel_rendering::*;

pub struct VoxelTerrainRenderStage
{
    device: Arc<wgpu::Device>,
    
    camera: Camera,
    camera_uniform: RefCell<Uniform<CameraUniform>>,
    camera_bind_group: BindGroup,

    voxel_meshes: Vec<VoxelRenderData>,

    render_pipeline: wgpu::RenderPipeline,
}

impl VoxelTerrainRenderStage
{
    pub fn new(camera: Camera, device: Arc<wgpu::Device>, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let mut camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);
        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, &device);

        let camera_bind_group = BindGroup::new(&[&camera_uniform], &device);
        let mesh_bind_group_layout = VoxelRenderData::construct_layout(&device);

        let render_pipeline = construct_render_pipeline(&device, config, &RenderPipelineInfo {
            shader_source: include_str!("../shaders/voxel_terrain_shader.wgsl"),
            shader_name: Some("Voxel Shader"),
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[&VoxelVertex::desc()],
            bind_groups: &[camera_bind_group.layout(), &mesh_bind_group_layout], // need to have bind group for thing
            label: Some("Voxel Render Pipeline")
        });

        Self 
        { 
            device, 
            camera, 
            camera_uniform: RefCell::new(camera_uniform), 
            camera_bind_group, 
            voxel_meshes: vec![], 
            render_pipeline 
        }
    }

    pub fn add_mesh(&mut self, mesh: &VoxelMesh, voxel_size: f32, position: Vec3<u32>)
    {
        let render_data = mesh.to_render_data(voxel_size, position, &self.device);
        self.voxel_meshes.push(render_data);
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }
}

impl RenderStage for VoxelTerrainRenderStage
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        let mut draw_calls: Vec<Box<(dyn DrawCall + 's)>> = vec![];
        for render_data in &self.voxel_meshes
        {
            let draw_call = VoxelDrawCall
            {
                camera_bind_group: &self.camera_bind_group,
                camera: self.camera.clone(),
                camera_uniform: &self.camera_uniform,
                render_data,
            };

            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct VoxelDrawCall<'a>
{
    camera_bind_group: &'a BindGroup,

    camera: Camera,
    camera_uniform: &'a RefCell<Uniform<CameraUniform>>,

    render_data: &'a VoxelRenderData
}

impl<'a> DrawCall for VoxelDrawCall<'a>
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([&self.camera_bind_group, &self.render_data.bind_group()])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut data = CameraUniform::new();
        data.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_write(data, queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.render_data.vertex_buffer().slice_all());
        render_pass.draw(0..(self.render_data.vertex_buffer().capacity() as u32), 0..1);
    }
}