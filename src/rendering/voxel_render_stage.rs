use std::sync::Arc;

use crate::camera::{Camera, CameraUniform};
use crate::math::{Vec3, Mat4x4, Point3D};
use crate::rendering::{ModelUniform, construct_render_pipeline};
use crate::voxel::terrain::VoxelTerrain;

use crate::colors::Color;
use super::{RenderStage, DrawCall, BindGroupData, VertexBuffer, VertexData, IndexBuffer};

pub const VOXEL_FACE_VERTICES: [VoxelVertex; 4] = [VoxelVertex::new(0, Color::WHITE), VoxelVertex::new(1, Color::RED), VoxelVertex::new(2, Color::GREEN), VoxelVertex::new(3, Color::BLUE)];
pub const VOXEL_FACE_TRIANGLES: [u16; 6] = [2, 1, 0, 2, 3, 1];

pub enum VoxelFace 
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl VoxelFace
{
    pub fn to_index(&self) -> u16
    {
        match self 
        {
            VoxelFace::Up => 0,
            VoxelFace::Down => 1,
            VoxelFace::North => 2,
            VoxelFace::South => 3,
            VoxelFace::East => 4,
            VoxelFace::West => 5,
        }
    }

    pub fn from_index(index: u16) -> Self
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
    pub index: u32,
    pub color: Color
}

impl VoxelVertex
{
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Uint32, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub const fn new(index: u32, color: Color) -> Self
    {
        Self { index, color }
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
pub struct VoxelFaceData
{
    pub position: Vec3<u32>,
    pub id: u32,
    pub face_index: u32
}

impl VoxelFaceData
{
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![2 => Uint32x3, 3 => Uint32, 4 => Uint32];

    pub fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }

    pub fn new(position: Vec3<u32>, id: u32, face_index: u32) -> Self
    {
        Self { position, id, face_index }
    }
}

unsafe impl bytemuck::Pod for VoxelFaceData {}
unsafe impl bytemuck::Zeroable for VoxelFaceData {}

impl VertexData for VoxelFaceData
{
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        Self::desc()
    }

    fn append_bytes(&self, bytes: &mut Vec<u8>) {
        bytes.extend(bytemuck::cast_slice(&[*self]).iter());
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
    pub data: Box<[VoxelRenderData]>
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

pub struct VoxelRenderStage
{
    terrain: Arc<VoxelTerrain>,
    bind_groups: [BindGroupData; 3],
    render_pipeline: wgpu::RenderPipeline,

    camera: Camera,

    faces_buffer: VertexBuffer<VoxelFaceData>,

    vertex_buffer: VertexBuffer<VoxelVertex>,
    index_buffer: IndexBuffer
}

impl VoxelRenderStage
{
    pub fn new(terrain: Arc<VoxelTerrain>, camera: Camera, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self
    {
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);
        let camera_bind_group = BindGroupData::uniform("camera_bind_group".into(), camera_uniform, wgpu::ShaderStages::VERTEX, device);

        let voxel_uniform = VoxelRenderDataUniform::new(terrain.voxel_types().iter().map(|v| v.get_render_data()).collect());
        let voxel_bind_group = BindGroupData::uniform_bytes("voxel_bind_group".into(), voxel_uniform.as_bytes(), wgpu::ShaderStages::VERTEX, device);

        let model_uniform = ModelUniform::from_position(terrain.position());
        let model_bind_group = BindGroupData::uniform("model_bind_group".into(), model_uniform, wgpu::ShaderStages::VERTEX, device);

        let vertex_buffer = VertexBuffer::new(&VOXEL_FACE_VERTICES, device, Some("Voxel vertex buffer"));
        let index_buffer = IndexBuffer::new(device, &VOXEL_FACE_TRIANGLES, Some("Voxel index buffer"));

        const FACE_BUFFER_CAPACITY: u64 = 65545;
        let faces_buffer = VertexBuffer::<VoxelFaceData>::new_empty(device, FACE_BUFFER_CAPACITY, Some("Faces instance buffer"));

        let render_pipeline = construct_render_pipeline(device, config, &crate::rendering::RenderPipelineInfo 
        { 
            shader_source: include_str!("../shaders/voxel_shader.wgsl"), 
            shader_name: Some("Voxel shader"), 
            vs_main: "vs_main", 
            fs_main: "fs_main", 
            vertex_buffers: &[&vertex_buffer, &faces_buffer], 
            bind_groups: &[&camera_bind_group, &model_bind_group, &voxel_bind_group], 
            label: Some("Voxel Render Pipeline")
        });

        Self 
        {
            terrain, 
            bind_groups: [camera_bind_group, model_bind_group, voxel_bind_group], 
            render_pipeline,
            camera,
            faces_buffer,
            vertex_buffer,
            index_buffer
        }
    }

    pub fn update(&mut self, camera: Camera)
    {
        self.camera = camera;
    }
}

impl RenderStage for VoxelRenderStage
{
    fn bind_groups(&self) -> &[BindGroupData]
    {
        &self.bind_groups
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline 
    {
        &self.render_pipeline
    }

    fn get_draw_calls<'s>(&'s self) -> Vec<Box<(dyn DrawCall + 's)>>
    {
        let faces_count = self.terrain.faces().len();
        let mut ranges = vec![self.faces_buffer.capacity(); faces_count / self.faces_buffer.capacity() as usize];
        let remainder = faces_count % self.faces_buffer.capacity() as usize;
        if remainder != 0 { ranges.push(remainder as u64); }

        let mut current_index: usize = 0;
        let mut draw_calls: Vec<Box<dyn DrawCall>> = vec![];
        for range in ranges
        {
            let old = current_index;
            current_index += range as usize;
            let slice = &self.terrain.faces()[old..current_index];
            let draw_call = VoxelDrawCall
            {
                voxels: slice,
                faces_buffer: &self.faces_buffer,
                vertex_buffer: &self.vertex_buffer,
                index_buffer: &self.index_buffer,
                faces_length: self.terrain.faces().len() as u64,
                camera: self.camera.clone(),
                position: self.terrain.position(),
                camera_bind_group: &self.bind_groups[0],
                model_bind_group: &self.bind_groups[1]
            };

            draw_calls.push(Box::new(draw_call));
        }

        draw_calls
    }
}

pub struct VoxelDrawCall<'vox, 'buffer, 'bind_group>
{
    voxels: &'vox [VoxelFaceData],
    faces_buffer: &'buffer VertexBuffer<VoxelFaceData>,
    vertex_buffer: &'buffer VertexBuffer<VoxelVertex>,
    index_buffer: &'buffer IndexBuffer,
    faces_length: u64,

    camera: Camera,
    position: Point3D<f32>,

    camera_bind_group: &'bind_group BindGroupData,
    model_bind_group: &'bind_group BindGroupData
}

impl<'vox, 'buffer, 'bind_group> DrawCall for VoxelDrawCall<'vox, 'buffer, 'bind_group>
{
    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        self.faces_buffer.enqueue_set_data(queue, self.voxels);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&self.camera);
        self.camera_bind_group.enqueue_set_data(queue, camera_uniform);

        let model_uniform = ModelUniform::from_position(self.position);
        self.model_bind_group.enqueue_set_data(queue, model_uniform);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>)
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.set_vertex_buffer(1, self.faces_buffer.slice(0, (self.faces_length as usize * std::mem::size_of::<VoxelFaceData>()) as u64));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..6, 0, 0..(self.faces_length as u32));
    }
}