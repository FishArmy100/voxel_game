use std::sync::{Arc, Mutex, MutexGuard};

use cgmath::{Array, Zero};

use crate::camera::{Camera, CameraUniform};
use crate::math::{Vec3, Mat4x4, Point3D};
use crate::rendering::{ModelUniform, construct_render_pipeline};
use crate::voxel::{VoxelStorage, Voxel, VoxelVertex};
use crate::voxel::terrain::VoxelTerrain;

use crate::colors::Color;
use super::{RenderStage, DrawCall, BindGroupData, VertexBuffer, VertexData, IndexBuffer};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelRenderData 
{
    pub color: Color
}

impl VoxelRenderData 
{
    pub fn new(color: Color) -> Self 
    {
        Self { color }
    }
}

unsafe impl bytemuck::Pod for VoxelRenderData {}
unsafe impl bytemuck::Zeroable for VoxelRenderData {}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct VoxelRenderDataUniform
{
    pub data: Box<[VoxelRenderData]>,
}

impl VoxelRenderDataUniform
{
    pub fn new(data: Box<[VoxelRenderData]>) -> Self
    {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8]
    {
        bytemuck::cast_slice(&self.data)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VoxelSizeUniform
{
    pub voxel_size: f32
}

unsafe impl bytemuck::Pod for VoxelSizeUniform {}
unsafe impl bytemuck::Zeroable for VoxelSizeUniform {}

impl VoxelSizeUniform
{
    fn new(voxel_size: f32) -> Self
    {
        Self { voxel_size }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ChunkUniform
{
    pub position: Vec3<i32>,
    pub chunk_size: u32
}

unsafe impl bytemuck::Pod for ChunkUniform {}
unsafe impl bytemuck::Zeroable for ChunkUniform {}

pub struct VoxelRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send
{
    terrain: Arc<Mutex<VoxelTerrain<TStorage>>>,
    
    camera_bind_group: BindGroupData,
    chunk_bind_group: BindGroupData,
    voxel_bind_group: BindGroupData,
    voxel_size_bind_group: BindGroupData,

    render_pipeline: wgpu::RenderPipeline,

    camera: Camera
}

impl<TStorage> VoxelRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    pub fn new(terrain: Arc<Mutex<VoxelTerrain<TStorage>>>, camera: Camera, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let terrain_mutex = terrain.lock().unwrap();
        let voxel_uniform = VoxelRenderDataUniform::new(terrain_mutex.voxel_types().iter().map(|v| v.get_render_data()).collect());
        let voxel_bind_group = BindGroupData::uniform_bytes("voxel_bind_group".into(), voxel_uniform.as_bytes(), wgpu::ShaderStages::FRAGMENT, device);

        let chunk_uniform = ChunkUniform {
            position: Vec3::zero(),
            chunk_size: 0,
        };
        
        let chunk_bind_group = BindGroupData::uniform("chunk_bind_group".into(), chunk_uniform, wgpu::ShaderStages::VERTEX, device);

        let voxel_size_uniform = VoxelSizeUniform::new(terrain_mutex.info().voxel_size);
        let voxel_size_bind_group = BindGroupData::uniform("voxel_size_bind_group".into(), voxel_size_uniform, wgpu::ShaderStages::VERTEX, device);
        drop(terrain_mutex);

        let render_pipeline = construct_render_pipeline(device, config, &crate::rendering::RenderPipelineInfo 
        { 
            shader_source: include_str!("../shaders/voxel_shader.wgsl"), 
            shader_name: Some("Voxel shader"), 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffer_layouts: &[&VoxelVertex::desc()],
            bind_groups: &[&camera_bind_group, &chunk_bind_group, &voxel_bind_group, &voxel_size_bind_group], 
            label: Some("Voxel Render Pipeline")
        });

        Self 
        {
            terrain, 
            camera_bind_group,
            chunk_bind_group,
            voxel_bind_group, 
            voxel_size_bind_group,
            render_pipeline,
            camera,
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }
}

impl<TStorage> RenderStage for VoxelRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn bind_groups(&self) -> Box<[&BindGroupData]>
    {
        Box::new([&self.camera_bind_group, &self.chunk_bind_group, &self.voxel_bind_group, &self.voxel_size_bind_group])
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>
    {
        let mut draw_calls: Vec<Box<dyn DrawCall>> = vec![];
        let terrain = Arc::new(self.terrain.lock().unwrap());
        for chunk_index in 0..terrain.chunks().len()
        {
            if terrain.chunks()[chunk_index].storage().is_empty()
            {
                continue;
            }

            let draw_call = VoxelDrawCall
            {
                chunk_index,
                camera: self.camera.clone(),
                position: Point3D::from_value(0.0),
                camera_bind_group: &self.camera_bind_group,
                chunk_bind_group: &self.chunk_bind_group,
                terrain: terrain.clone()
            };

            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct VoxelDrawCall<'a, TStorage> where TStorage : VoxelStorage<Voxel> 
{
    chunk_index: usize,

    camera: Camera,
    position: Point3D<f32>,

    camera_bind_group: &'a BindGroupData,
    chunk_bind_group: &'a BindGroupData,

    terrain: Arc<MutexGuard<'a, VoxelTerrain<TStorage>>>
}

impl<'a, TStorage> DrawCall for VoxelDrawCall<'a, TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_bind_group.enqueue_set_data(queue, camera_uniform);

        let model_uniform = ChunkUniform {
            position: self.terrain.chunks()[self.chunk_index].chunk_index().cast().unwrap(),
            chunk_size: self.terrain.chunk_size() as u32
        };

        self.chunk_bind_group.enqueue_set_data(queue, model_uniform);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>)
    {
        let chunk = &self.terrain.chunks()[self.chunk_index];

        if let Some((vertex_buffer, index_buffer)) = &chunk.get_buffers()
        {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice_all());
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.draw_indexed(0..index_buffer.capacity() as u32, 0, 0..1);
        }
    }
}