use std::{sync::Arc, cell::RefCell};

use crate::{math::Vec3, rendering::{VertexData, VertexBuffer, construct_render_pipeline, RenderPipelineInfo, DrawCall, IVertexBuffer, RenderStage}, camera::{Camera, CameraUniform}};
use crate::gpu_utils::{Storage, BindGroup, Uniform, GPUVec3, Entry};

pub enum FaceDir
{
    Up,
    Down,
    North,
    South,
    East,
    West
}

impl FaceDir
{
    pub fn to_index(&self) -> u32
    {
        match self 
        {
            FaceDir::Up =>      0,
            FaceDir::Down =>    1,
            FaceDir::North =>   2,
            FaceDir::South =>   3,
            FaceDir::East =>    4,
            FaceDir::West =>    5,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelFace
{
    pos_x: u32,
    pos_y: u32,
    pos_z: u32,
    direction: u32,
    voxel_id: u32,
}

unsafe impl bytemuck::Pod for VoxelFace {}
unsafe impl bytemuck::Zeroable for VoxelFace {}

impl VoxelFace
{
    pub fn new(location: Vec3<u32>, direction: FaceDir, voxel_id: u16) -> Self 
    {
        Self 
        { 
            pos_x: location.x, 
            pos_y: location.y, 
            pos_z: location.z, 
            direction: direction.to_index(), 
            voxel_id: voxel_id as u32 
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex
{
    pub face_index: u32,
}

unsafe impl bytemuck::Pod for VoxelVertex {}
unsafe impl bytemuck::Zeroable for VoxelVertex {}

impl VoxelVertex
{
    pub fn new(face_index: u32) -> Self 
    {
        Self 
        { 
            face_index, 
        }
    }
}

impl VertexData for VoxelVertex
{
    fn desc() -> wgpu::VertexBufferLayout<'static>
    {
        const ATTRIBUTES: [wgpu::VertexAttribute; 1] =
            wgpu::vertex_attr_array![0 => Uint32];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }

    fn append_bytes(&self, bytes: &mut Vec<u8>) 
    {
        bytes.extend(bytemuck::cast_slice(&[*self]));
    }
}

pub struct VoxelMesh
{
    vertices: Vec<VoxelVertex>,
    faces: Vec<VoxelFace>,
}

impl VoxelMesh
{
    pub fn faces(&self) -> &Vec<VoxelFace> { &self.faces }
    pub fn vertices(&self) -> &Vec<VoxelVertex> { &self.vertices }

    pub fn new() -> Self 
    {
        Self 
        { 
            vertices: vec![], 
            faces: vec![],
        }
    }

    pub fn add_face(&mut self, location: Vec3<u32>, direction: FaceDir, voxel_id: u16)
    {
        let face_count = self.faces.len() as u32;
        self.vertices.extend(
        [
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count),
            VoxelVertex::new(face_count)
        ]);

        self.faces.push(VoxelFace::new(location, direction, voxel_id))
    }

    pub fn to_render_data(&self, voxel_size: f32, position: Vec3<u32>, device: &wgpu::Device) -> VoxelRenderData
    {
        let vertex_buffer = VertexBuffer::new(&self.vertices, device, Some("Voxel Vertex Buffer"));
        let face_storage = Storage::new(&self.faces, wgpu::ShaderStages::VERTEX, device);
        let voxel_size_uniform = Uniform::new(voxel_size, wgpu::ShaderStages::VERTEX, device);
        let position_uniform = Uniform::new(position.into(), wgpu::ShaderStages::VERTEX, device);

        VoxelRenderData::new(vertex_buffer, face_storage, voxel_size_uniform, position_uniform, device)
    }
}

pub struct VoxelRenderData
{
    vertex_buffer: VertexBuffer<VoxelVertex>,
    face_storage: Storage<VoxelFace>,
    voxel_size_uniform: Uniform<f32>,
    position_uniform: Uniform<GPUVec3<u32>>,
    
    bind_group: BindGroup
}

impl VoxelRenderData
{
    pub fn new(vertex_buffer: VertexBuffer<VoxelVertex>, face_storage: Storage<VoxelFace>, voxel_size_uniform: Uniform<f32>, position_uniform: Uniform<GPUVec3<u32>>, device: &wgpu::Device) -> Self
    {
        let bind_group = BindGroup::new(&[&face_storage, &voxel_size_uniform, &position_uniform], device);

        Self 
        { 
            vertex_buffer, 
            face_storage,
            voxel_size_uniform,
            position_uniform,
            bind_group
        }
    }

    pub fn construct_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout
    {
        let face_storage_layout = Storage::<VoxelFace>::get_layout_static(wgpu::ShaderStages::VERTEX, 0);
        let voxel_size_layout = Uniform::<f32>::get_layout_static(wgpu::ShaderStages::VERTEX, 1);
        let position_layout = Uniform::<GPUVec3<u32>>::get_layout_static(wgpu::ShaderStages::VERTEX, 2);
        BindGroup::construct_layout_from_entries(&[face_storage_layout, voxel_size_layout, position_layout], device)
    }
}

pub struct VoxelRenderStage
{
    device: Arc<wgpu::Device>,
    
    camera: Camera,
    camera_uniform: RefCell<Uniform<CameraUniform>>,
    camera_bind_group: BindGroup,

    voxel_meshes: Vec<VoxelRenderData>,

    render_pipeline: wgpu::RenderPipeline,
}

impl VoxelRenderStage
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

impl RenderStage for VoxelRenderStage
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
        Box::new([&self.camera_bind_group, &self.render_data.bind_group])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut data = CameraUniform::new();
        data.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_write(data, queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.render_data.vertex_buffer.slice_all());
        render_pass.draw(0..(self.render_data.vertex_buffer.capacity() as u32), 0..1);
    }
}