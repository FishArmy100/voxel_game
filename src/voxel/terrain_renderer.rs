use std::{sync::Arc, cell::RefCell};

use std::sync::{Mutex, MutexGuard};

use cgmath::Array;

use crate::{math::Vec3, rendering::{construct_render_pipeline, RenderPipelineInfo, DrawCall, RenderStage}, camera::{Camera, CameraUniform}, colors::Color};
use crate::gpu_utils::{Storage, BindGroup, Uniform, VertexBuffer, VertexData, GPUVec3, Entry, GBuffer, IndexBuffer};
use crate::voxel::voxel_rendering::*;

use super::{terrain::{VoxelTerrain, Chunk}, VoxelStorage, Voxel};

pub struct ChunkRenderData
{
    face_instance_buffer: VertexBuffer<VoxelFace>
}

impl ChunkRenderData
{
    pub fn face_instance_buffer(&self) -> &VertexBuffer<VoxelFace> { &self.face_instance_buffer }

    pub fn new(mesh: &VoxelMesh, device: &wgpu::Device) -> Self
    {
        Self 
        {
            face_instance_buffer: mesh.create_buffers(device)
        }
    }
}

pub struct TerrainRenderStage<TStorage> where TStorage : VoxelStorage<Voxel>
{
    device: Arc<wgpu::Device>,
    
    camera: Camera,
    camera_uniform: RefCell<Uniform<CameraUniform>>,
    _voxel_size_uniform: Uniform<f32>,
    _voxel_color_storage: Storage<Color>,
    chunk_position_uniform: RefCell<Uniform<GPUVec3<i32>>>,

    vertex_buffer: VertexBuffer<VoxelVertex>,
    index_buffer: IndexBuffer,

    terrain: Arc<Mutex<VoxelTerrain<TStorage>>>,
    terrain_bind_group: BindGroup,

    render_pipeline: wgpu::RenderPipeline,
}

impl<TStorage> TerrainRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    pub fn new(terrain: Arc<Mutex<VoxelTerrain<TStorage>>>, camera: Camera, device: Arc<wgpu::Device>, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let terrain_mutex = terrain.lock().unwrap();

        let mut camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);

        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, &device);
        let voxel_size_uniform = Uniform::new(terrain_mutex.info().voxel_size, wgpu::ShaderStages::VERTEX, &device);

        let chunk_position_uniform = Uniform::new(GPUVec3::new(0, 0, 0), wgpu::ShaderStages::VERTEX, &device);

        let voxel_colors: Vec<Color> = terrain_mutex
            .info().voxel_types
            .iter()
            .map(|v| v.color.into())
            .collect();

        let voxel_color_storage = Storage::new(&voxel_colors, wgpu::ShaderStages::VERTEX, &device);

        let vertex_buffer = VertexBuffer::new(&VOXEL_FACE_VERTICES, &device, Some("Voxel Vertex Buffer"));
        let index_buffer = IndexBuffer::new(&VOXEL_FACE_TRIANGLES, &device, Some("Voxel Index Buffer"));

        let terrain_bind_group = BindGroup::new(&[&camera_uniform, &voxel_size_uniform, &chunk_position_uniform, &voxel_color_storage], &device);

        let render_pipeline = construct_render_pipeline(&device, config, &RenderPipelineInfo {
            shader_source: include_str!("../shaders/voxel_terrain_shader.wgsl"),
            shader_name: Some("Voxel Shader"),
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[&VoxelFace::desc(), &VoxelVertex::desc()],
            bind_groups: &[terrain_bind_group.layout()],
            label: Some("Voxel Render Pipeline")
        });

        drop(terrain_mutex);

        Self 
        { 
            device, 
            camera, 
            camera_uniform: RefCell::new(camera_uniform), 
            _voxel_size_uniform: voxel_size_uniform, 
            _voxel_color_storage: voxel_color_storage, 
            chunk_position_uniform: RefCell::new(chunk_position_uniform),
            vertex_buffer,
            index_buffer,
            terrain_bind_group, 
            terrain, 
            render_pipeline 
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }
}

impl<TStorage> RenderStage for TerrainRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>> 
    {
        let mut draw_calls: Vec<Box<(dyn DrawCall + 's)>> = vec![];
        let terrain = Arc::new(self.terrain.lock().unwrap());

        let face_count: u64 = terrain.chunks().iter().map(|c| c.render_data().map_or(0, |c| c.face_instance_buffer().length())).sum();
        println!("Rendered {} faces.", face_count);

        for chunk_index in 0..terrain.chunks().len()
        {
            if terrain.chunks()[chunk_index].render_data().is_none()
            {
                continue;
            }

            let draw_call = TerrainDrawCall
            {
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                terrain_bind_group: &self.terrain_bind_group,
                camera: self.camera.clone(),
                camera_uniform: &self.camera_uniform,
                chunk_position_uniform: &self.chunk_position_uniform,
                chunk_index,
                terrain: terrain.clone()
            };

            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct TerrainDrawCall<'a, TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    vertex_buffer: &'a VertexBuffer<VoxelVertex>,
    index_buffer: &'a IndexBuffer,
    terrain_bind_group: &'a BindGroup,

    camera: Camera,
    camera_uniform: &'a RefCell<Uniform<CameraUniform>>,
    chunk_position_uniform: &'a RefCell<Uniform<GPUVec3<i32>>>,

    terrain: Arc<MutexGuard<'a, VoxelTerrain<TStorage>>>,
    chunk_index: usize
}

impl<'a, TStorage> DrawCall for TerrainDrawCall<'a, TStorage> 
    where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([&self.terrain_bind_group])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut data = CameraUniform::new();
        data.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_write(data, queue);

        let chunk_index: Vec3<i32> = self.terrain.chunks()[self.chunk_index].index().cast().unwrap();
        let chunk_position = chunk_index * self.terrain.info().chunk_length() as i32;
        self.chunk_position_uniform.borrow_mut().enqueue_write(chunk_position.into(), queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        let chunk_render_data = self.terrain.chunks()[self.chunk_index].render_data();
        render_pass.set_vertex_buffer(0, chunk_render_data.unwrap().face_instance_buffer().slice_all());
        render_pass.set_vertex_buffer(1, self.vertex_buffer.slice_all());
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..(chunk_render_data.unwrap().face_instance_buffer().length() as u32));
    }
}