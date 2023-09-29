use std::sync::{Arc, Mutex, MutexGuard};

use cgmath::Array;

use crate::camera::{Camera, CameraUniform};
use crate::math::{Vec3, Point3D};
use crate::rendering::construct_render_pipeline;
use crate::voxel::{VoxelStorage, Voxel};
use crate::voxel::terrain::VoxelTerrain;

use crate::colors::Color;
use super::{RenderStage, DrawCall, BindGroupData, VertexData, RenderPipelineInfo};

pub enum VoxelFaceOrientation 
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl VoxelFaceOrientation
{
    pub fn to_index(&self) -> u32
    {
        match self 
        {
            VoxelFaceOrientation::Up => 0,
            VoxelFaceOrientation::Down => 1,
            VoxelFaceOrientation::North => 2,
            VoxelFaceOrientation::South => 3,
            VoxelFaceOrientation::East => 4,
            VoxelFaceOrientation::West => 5,
        }
    }

    pub fn from_index(index: u32) -> Self
    {
        match index 
        {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::North,
            3 => Self::South,
            4 => Self::East,
            5 => Self::West,
            _ => panic!("Invalid index {}", index)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex 
{
    pub face_id: u32
}

impl VoxelVertex
{
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] =
            wgpu::vertex_attr_array![0 => Uint32];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub const fn new(face_id: u32) -> Self
    {
        Self { face_id }
    }
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        Self::desc()
    }

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]).iter())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelFace
{
    pub position_x: u32,
    pub position_y: u32,
    pub position_z: u32,
    pub orientation: u32,
    pub voxel_id: u32,
}

unsafe impl bytemuck::Pod for VoxelFace {}
unsafe impl bytemuck::Zeroable for VoxelFace {}

impl VoxelFace
{
    pub fn new(position: Vec3<u32>, orientation: VoxelFaceOrientation, voxel_id: u32) -> Self
    {
        Self 
        {
            position_x: position.x,
            position_y: position.y,
            position_z: position.z,
            orientation: orientation.to_index(),
            voxel_id,
        }
    }
}

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
struct ChunkUniform
{
    pos_x: i32,
    pos_y: i32,
    pos_z: i32,
    size: u32,
    voxel_size: f32,
}

unsafe impl bytemuck::Zeroable for ChunkUniform {}
unsafe impl bytemuck::Pod for ChunkUniform {}

impl ChunkUniform
{
    pub fn new(position: Vec3<i32>, size: u32, voxel_size: f32) -> Self
    {
        Self 
        { 
            pos_x: position.x,
            pos_y: position.y,
            pos_z: position.z,
            size,
            voxel_size
        }
    }
}

pub struct VoxelRenderStage<TStorage> where TStorage : VoxelStorage<Voxel> + Send
{
    terrain: Arc<Mutex<VoxelTerrain<TStorage>>>,
    
    camera_bind_group: BindGroupData,
    chunk_bind_group: BindGroupData,
    voxel_bind_group: BindGroupData,

    render_pipeline: wgpu::RenderPipeline,

    camera: Camera,
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

        let chunk_uniform = ChunkUniform::new(Vec3::new(0, 0, 0), 0, 0.0);
        let chunk_bind_group = BindGroupData::uniform("voxel_size_bind_group".into(), chunk_uniform, wgpu::ShaderStages::VERTEX, device);
        drop(terrain_mutex);

        let face_storage_layout = BindGroupData::get_storage_layout(wgpu::ShaderStages::VERTEX, device);

        let render_pipeline = construct_render_pipeline(device, config, &RenderPipelineInfo 
        { 
            shader_source: include_str!("../shaders/voxel_shader.wgsl"), 
            shader_name: Some("Voxel shader"), 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[&VoxelVertex::desc()], 
            bind_groups: &[camera_bind_group.layout(), chunk_bind_group.layout(), voxel_bind_group.layout(), &face_storage_layout], 
            label: Some("Voxel Render Pipeline")
        });

        Self 
        {
            terrain, 
            camera_bind_group,
            voxel_bind_group, 
            chunk_bind_group,
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
            if terrain.chunks()[chunk_index].mesh_data().is_none()
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
                voxel_bind_group: &self.voxel_bind_group,
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
    voxel_bind_group: &'a BindGroupData,

    terrain: Arc<MutexGuard<'a, VoxelTerrain<TStorage>>>
}

impl<'a, TStorage> DrawCall for VoxelDrawCall<'a, TStorage> where TStorage : VoxelStorage<Voxel> + Send + 'static
{
    fn bind_groups(&self) -> Box<[&BindGroupData]> 
    {
        let face_storage = self.terrain.chunks()[self.chunk_index].mesh_data().as_ref().unwrap().faces_bind_group();
        Box::new([&self.camera_bind_group, &self.chunk_bind_group, &self.voxel_bind_group, face_storage])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_bind_group.enqueue_set_data(queue, camera_uniform);

        let chunk = &self.terrain.chunks()[self.chunk_index];

        let chunk_uniform = ChunkUniform::new(chunk.index().cast().unwrap(), chunk.size() as u32, self.terrain.info().voxel_size);
        
        let face_storage = self.terrain.chunks()[self.chunk_index].mesh_data().as_ref().unwrap().faces_bind_group();
        face_storage.enqueue_set_data(queue, chunk_uniform);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>)
    {
        let chunk = &self.terrain.chunks()[self.chunk_index];

        if let Some(chunk_mesh) = &chunk.mesh_data()
        {
            render_pass.set_vertex_buffer(0, chunk_mesh.vertex_buffer().slice_all());
            render_pass.set_index_buffer(chunk_mesh.index_buffer().slice(..), wgpu::IndexFormat::Uint32);

            let indices_count = chunk_mesh.index_buffer().capacity() as u32;
            render_pass.draw_indexed(0..indices_count, 0, 0..1);
        }
    }
}