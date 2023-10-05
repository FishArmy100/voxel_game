use std::{sync::Arc, cell::RefCell};

use crate::{math::Vec3, rendering::{VertexData, VertexBuffer, construct_render_pipeline, RenderPipelineInfo, DrawCall, IVertexBuffer, RenderStage}, gpu_utils::{Storage, BindGroup, Uniform}, camera::{Camera, CameraUniform}};

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
    faces: Vec<VoxelFace>
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
            faces: vec![] 
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
}

pub struct VoxelRenderStage
{
    device: Arc<wgpu::Device>,
    
    camera: Camera,
    camera_uniform: RefCell<Uniform<CameraUniform>>,

    face_storage: Storage<VoxelFace>,
    vertex_buffer: VertexBuffer<VoxelVertex>,

    bind_group: BindGroup,

    render_pipeline: wgpu::RenderPipeline,
}

impl VoxelRenderStage
{
    pub fn new(mesh: VoxelMesh, camera: Camera, device: Arc<wgpu::Device>, config: &wgpu::SurfaceConfiguration) -> Self 
    {
        let mut camera_uniform_data = CameraUniform::new();
        camera_uniform_data.update_view_proj(&camera);
        let camera_uniform = Uniform::new(camera_uniform_data, wgpu::ShaderStages::VERTEX, &device);

        let face_storage = Storage::new(mesh.faces(), wgpu::ShaderStages::VERTEX, &device);
        let bind_group = BindGroup::new(&[&camera_uniform, &face_storage], &device);

        let vertex_buffer = VertexBuffer::new(mesh.vertices(), &device, Some("Voxel Vertex Buffer"));

        let render_pipeline = construct_render_pipeline(&device, config, &RenderPipelineInfo {
            shader_source: include_str!("../shaders/voxel_shader.wgsl"),
            shader_name: Some("Voxel Shader"),
            vs_main: "vs_main",
            fs_main: "fs_main",
            vertex_buffers: &[vertex_buffer.layout()],
            bind_groups: &[bind_group.layout()],
            label: Some("Voxel Render Pipeline")
        });

        Self 
        { 
            device, 
            camera, 
            camera_uniform: RefCell::new(camera_uniform), 
            face_storage, 
            vertex_buffer, 
            bind_group,
            render_pipeline, 
        }
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
        let draw_call = VoxelDrawCall
        {
            bind_group: &self.bind_group,
            camera: self.camera.clone(),
            camera_uniform: &self.camera_uniform,
            vertex_buffer: &self.vertex_buffer
        };

        vec![Box::new(draw_call)]
    }
}

pub struct VoxelDrawCall<'a>
{
    bind_group: &'a BindGroup,
    camera: Camera,
    camera_uniform: &'a RefCell<Uniform<CameraUniform>>,
    vertex_buffer: &'a VertexBuffer<VoxelVertex>
}

impl<'a> DrawCall for VoxelDrawCall<'a>
{
    fn bind_groups(&self) -> Box<[&BindGroup]> 
    {
        Box::new([self.bind_group])
    }

    fn on_pre_draw(&self, queue: &wgpu::Queue) 
    {
        let mut data = CameraUniform::new();
        data.update_view_proj(&self.camera);
        self.camera_uniform.borrow_mut().enqueue_write(data, queue);
    }

    fn on_draw<'pass, 's: 'pass>(&'s self, render_pass: &mut wgpu::RenderPass<'pass>) 
    {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice_all());
        render_pass.draw(0..(self.vertex_buffer.capacity() as u32), 0..1);
    }
}